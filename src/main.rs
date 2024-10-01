use std::{fs::Permissions, io::Write, os::unix::fs::PermissionsExt, process::Stdio};

fn emplace_rlink(command: &str, rlink_path: &str) -> Option<String> {
    let (b, suffix) = command.split_once(".rs ")?;
    let (prefix, _) = b.rsplit_once(" ")?;
    Some(format!("{prefix} '{rlink_path}' {suffix}"))
}

// We utilize cargo to discover:
// - The `rustc` invocation to build the crate
// - The `linker` invocation to link the final binary.

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let lib = args.next().expect("dep crate arg");
    let bin = args.next().expect("bin crate arg");
    let value = std::process::Command::new("cargo")
        .arg("-vv")
        .arg("build")
        .stderr(Stdio::piped())
        .spawn()
        .expect("cargo to run");
    use std::io::BufRead;
    let mut lib_command: Option<String> = None;
    let mut bin_command: Option<String> = None;
    let se = std::io::BufReader::new(value.stderr.unwrap());
    // Cargo probably doesn't have a stable output format
    for line in se.lines() {
        let line = line.unwrap();
        if let Some(initial) = line.strip_prefix("     Running `") {
            let Some((inner, _)) = initial.rsplit_once('`') else {
                continue;
            };
            if let Some((_, rest)) = inner.split_once("--crate-name ") {
                if let Some((crate_name, _)) = rest.split_once(" ") {
                    if crate_name == lib {
                        lib_command = Some(inner.to_string());
                    } else if crate_name == bin {
                        bin_command = Some(inner.to_string());
                    }
                }
            }
        } else {
            println!("{}", line);
        }
    }
    let Some(bin_command) = bin_command else {
        panic!("missing bin command")
    };

    // We split up the link and build step using `no-link and `link-only`
    // so we can avoid rebuilding the final crate.

    let output = std::process::Command::new("/bin/sh")
        .env("RUSTC_BOOTSTRAP", "1")
        .arg("-c")
        .arg(format!("{bin_command} -Z no-link"))
        .output()
        .unwrap();
    let content = std::str::from_utf8(&output.stderr).expect("to be utf8");
    let artifact = jsony::drill(&content)["artifact"].parse::<&str>().unwrap();
    let rlink = format!("{}.rlink", artifact.strip_suffix(".d").unwrap());

    let rustc_link_command = emplace_rlink(&bin_command, &rlink).unwrap();
    let fin = std::process::Command::new("/bin/sh")
        .env("RUSTC_BOOTSTRAP", "1")
        .arg("-c")
        .arg(format!(
            "{rustc_link_command} -Z link-only -C save-temps --print link-args"
        ))
        .output()
        .unwrap();

    let content = std::str::from_utf8(&fin.stderr).unwrap();
    let run_location = jsony::drill(&content)["artifact"].parse::<&str>().unwrap();

    let link_command = std::str::from_utf8(&fin.stdout).expect("Link command to be utf8");

    let lib_command = lib_command.expect("lib command");
    let mut f = std::fs::File::create("./spooky_run.sh").unwrap();
    let _ = f.write_all(b"#!/bin/sh\n");
    let _ = f.write_all(b"set -e\n echo BUILDING LIB\n");
    let _ = f.write_all(b"start=`date +%s.%N`\n");
    let _ = f.write_all(lib_command.as_bytes());
    let _ = f.write_all(b"\n echo LINKING BINARY\n");
    let _ = f.write_all(link_command.as_bytes());
    let _ = f.write_all(b"end=`date +%s.%N`\n");
    let _ = f.write_all(b"runtime=$( echo \"$end - $start\" | bc -l )\n");
    let _ = f.write_all(b"echo \"BUILD completed in $runtime seconds\"");
    let _ = f.write_all(b"\n");
    let _ = f.write_all(run_location.as_bytes());
    drop(f);
    let _ = std::fs::set_permissions("./spooky_run.sh", Permissions::from_mode(0o755));
}
