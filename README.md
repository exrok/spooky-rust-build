# Spooky Rust Compilation

An extremely quick prototype to demonstrate the possible optimization of avoiding unnecessary
rebuilds of dependent crates.

**WARNING**: This technique does no checking whatsoever on whether the optimization
is safe to perform. Subtle changes can cause a rebuild to be necessary, so expect to see segfaults or worse.

Tested on Rust 1.81 on Linux.

Example Usage:

Say you're working on `zed`, and anticipate working on the internals of `gpui`.

Modify something in the internals of `gpui` (to ensure `gpui` gets rebuilt and collects
the `rustc` invocations in the next step), then run:

```sh
spooky-rust-build gpui zed
```

Afterward, you can now run `./spooky_run.sh`, which will rebuild only gpui, relink the final binary, and then run the zed binary.

