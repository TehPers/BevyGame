# WasmGame

Work in progress experiment using WASM modules to add components to a game.

## Building and running

To build this project, you need to install the latest version of the Rust nightly compiler from [rustup.rs](https://rustup.rs/).
Ensure that you have the correct version installed by running these commands:

```sh
rustup self update
rustup override set nightly
rustup update
```

Additionally, you may need to install the following packages on your computer:

- clang (Linux)
- lld (Linux) / zld (Mac OSX)

Finally, compile and run the game via the following commands:

```sh
cd engine
cargo run # Add flags here
```

Possible flags:

- `--features trace`: Add tracing output usable by chrome://tracing
- `--release`: Takes longer to compile, but optimizes the build output
   - Certain CPU-specific optimizations may be performed by release builds, so ensure you build on the computer you intend to run it from
