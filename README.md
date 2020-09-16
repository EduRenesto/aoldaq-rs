# Rust Backend for the AOLDAQ Acquisition Stack

This is the code that runs under the hood on the new acquisition stack for the
Silver Lab AOL microscope.

This code was previously written in C, but I had some problems and tried to
fix them by, well, rewriting in Rust (like any crustacean would do). Turns out
the experiment ended up working better than the year-mature C code, so this is
what I'll be shipping.

# Building

If you want to build it by source, you will need a working Rust toolchain. The
easiest way to do so is by installing the [rustup.rs](rustup.rs) toolchain
manager. A stable toolchain is enough, since I'm not using any nightly-only
features.

After you have cloned the repository and installed the toolchain, open a
terminal in this folder and run

```sh
cargo build --release
```

This will build the release mode `aoldaq.dll` in the `target` folder. The
Matlab code (at the time of writing) expects a `libaoldaq.dll` file, so copy
it to the `microscope_controller/nifpga/aoldaq` folder and rename it
accordingly.

If you do not want to go through the build process, I (am trying to) keep an
up-to-date debug dll in the root of this repo. You can just download it and
put in the afforementioned folder.

# Documentation

One of the most beautiful features of Rust is the `rustdoc` system. If you run
`cargo doc`, you will have the documentation of this API rendered in very
readable HTML form in the `target/doc` folder.
