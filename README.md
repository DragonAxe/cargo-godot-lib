# cargo-godot-lib

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/cargo-godot-lib.svg)](https://crates.io/crates/cargo-godot-lib)
[![.github/workflows/ci.yml](https://github.com/DragonAxe/cargo-godot-lib/actions/workflows/ci.yml/badge.svg)](https://github.com/DragonAxe/cargo-godot-lib/actions/workflows/ci.yml)

A Rust library for launching Godot from a Cargo run script, specifically designed for GDExtension development.

`cargo-godot-lib` provides similar functionality to the [`cargo-godot`](https://github.com/godot-rust/cargo-godot) executable but as a library. This allows you to include it as a dependency in your project, ensuring all developers have access to the same runner without requiring a separate global installation via `cargo install`.

## Features

- **`.gdextension` Generation**: Supports customized Cargo target directory (e.g. `target-dir = ".cache/cargo/target"`).
- **Godot Project Import**: Automatically runs `godot --import --headless` if the `.godot` folder is missing, eliminating the need to manually open the editor on a fresh clone.
- **Godot Binary Discovery**: Intelligently locates the Godot binary via environment variables (`godot` or `GODOT`), the system `PATH`, or common installation paths.
- **Developer Friendly**: Launches Godot with the `--debug` flag by default for better output in your terminal.
- **Configurable**: Convenient builder pattern allows customization of run parameters (See `GodotRunner` for details).

## Example Usage

Example project structure:

```
project/
├── Cargo.toml (workspace)
├── godot/
│   └── project.godot
└── rust/
    ├── Cargo.toml (package: example)
    ├── run_godot.rs
    └── src/
        └── main.rs
```

The main interface of `cargo-godot-lib` is the `GodotRunner` struct.
This struct follows a builder pattern and can be configured to suit your needs.
See the following `project/rust/run_godot.rs` example:

```rust
fn main() {
    let runner = cargo_godot_lib::GodotRunner::create(
        env!("CARGO_PKG_NAME"),
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../godot"),
    );
    if let Err(e) = runner.execute() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
```

Add `cargo-godot-lib` to your dependencies and define the binary in `project/rust/Cargo.toml`:

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "run-godot"
path = "run_godot.rs"

[dependencies]
# ... your other dependencies (e.g. godot-rust)
cargo-godot-lib = "<latest_version_goes_here>"
```

Now any developer can launch the Godot project and build the extension in one command:

```bash
cargo run --package example
```

## License

This project is licensed under the MIT License.
