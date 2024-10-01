# Spooky Rust Compilation

An extremely quick prototype to demonstrate the possible optimization of avoiding unnecessary
rebuilds of dependent crates.

Notably this brings incremental compile time down from `26.51s` to `2.69s`, on my 5950x linux workstation when developing on zed.


**WARNING**: The current implementation does no checking whatsoever on whether the optimization
is safe to perform. Subtle changes can cause a rebuild to be necessary, so expect to see segfaults or worse.
Using this is almost certainly going cause incremental compliation bugs and all sorts of pain but compile savings may be worth is.

Tested on Rust 1.81 on Linux.

Example Usage:

Say you're working on `zed`, and anticipate working on the internals of `gpui`.

Modify something in the internals of `gpui` (to ensure `gpui` gets rebuilt and collects
the `rustc` invocations in the next step), then run:

```sh
spooky-rust-build gpui zed
```

Afterward, you can now run `./spooky_run.sh`, which will rebuild only gpui, relink the final binary, and then run the zed binary.

## Limitations

Currently, this only supports working on one crate at a time. Support could be expanded to more usecases that would only make
it more dangerous this really just demo. Ideally, native & checked support lands in rustc itself.

See:
- [Compiler Major Change Request](https://github.com/rust-lang/compiler-team/issues/790)
- [Zulip Thread](https://rust-lang.zulipchat.com/#narrow/stream/233931-t-compiler.2Fmajor-changes/topic/Relink.2C.20don't.20rebuild.20compiler-team.23790)