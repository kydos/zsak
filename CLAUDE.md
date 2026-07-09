# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

**zsak** (Zenoh Swiss Army Knife) is a Rust CLI tool that compiles to a binary named `zenoh`. It provides a unified command-line interface for experimenting with the [Zenoh](https://zenoh.io) pub/sub/query protocol: publishing, subscribing, querying, queryables, scouting, liveliness tokens, in-memory storage, and routing graph inspection.

## Build commands

```bash
# Standard build
cargo build

# Release build
cargo build --release

# Build with optional video streaming feature (requires OpenCV)
cargo build --features video

# Run directly
cargo run -- <subcommand> [options]

# Run with logging
RUST_LOG=debug cargo run -- <subcommand>
```

There are no tests in this project currently.

## Code structure

The project has three source files:

- **`src/parser.rs`** — All CLI argument definitions (via `clap`) and argument resolution helpers (`resolve_argument`, `resolve_optional_argument`, `resolve_bool_argument`). Adding a new subcommand means extending `arg_parser()` here.
- **`src/action.rs`** — All action implementations (`do_scout`, `do_publish`, `do_subscribe`, `do_query`, `do_queryable`, `do_delete`, `do_declare_liveliness_token`, etc.). Each subcommand maps to a `do_*` async function.
- **`src/main.rs`** — Entry point: parses args, opens a Zenoh session, dispatches to `action.rs` functions, and handles the `wait_for_ctrl_c` pattern. Also contains `set_required_options` (always-on config) and `parse_top_level_args` (CLI-driven config overrides).

## Key design patterns

**Session lifecycle**: A single `zenoh::Session` is opened once in `main` and passed by reference to all action functions. The session is configured before opening via `set_required_options` (hardcoded defaults) and `parse_top_level_args` (user flags).

**`wait_for_ctrl_c`**: Subcommands return `bool` from the match arm — `true` means the process should block until Ctrl-C (used for long-running commands like `storage` and `liveliness --declare`). Most subcommands return `false` and exit after completion.

**Liveliness token ownership**: The `_token: Option<LivelinessToken>` in `main` must remain alive for the duration of the session. Dropping it revokes the liveliness declaration.

**Python embedding**: The `queryable --script` mode uses pyo3 to execute a Python script per incoming query. The script receives `key_expr` and `payload` as locals and must set `result`. See `script/dictionary.py` for an example.

**`{N}` macro**: In `publish`, the literal `{N}` in the value string is replaced with the current publication count (1-based).

## Subcommands

| Subcommand | Alias | Description |
|---|---|---|
| `doctor` | — | Checks `ZSAK_HOME` env var and `zenohd` in PATH |
| `scout <seconds>` | — | Scouts for Zenoh runtimes for N seconds |
| `list` | — | Lists discovered runtimes (`-r`/`-p`/`-c` to filter) |
| `publish <key> <value>` | `put` | Publishes; supports `--count`, `--period`, `--file` |
| `delete <key>` | — | Deletes a key |
| `subscribe <key>` | `sub` | Subscribes until Ctrl-C |
| `query <key>` | `get` | Issues a get/query |
| `queryable <key> <reply>` | — | Declares a queryable; `--script` runs reply as Python |
| `storage <key>` | — | Starts an in-memory storage via `zenohd` (requires `ZSAK_HOME`) |
| `liveliness` | — | Declare (`-d`), subscribe (`-s`), or query (`-q`) liveliness tokens |
| `graph` | — | Fetches the routing graph in DOT format from a router |
| `stream` | — | Video streaming (requires `--features video` and OpenCV) |

## Environment variables

- **`ZSAK_HOME`**: Required for the `storage` subcommand. Must point to the zsak installation directory containing `config/config.json5` with a `$STORAGE` placeholder that gets replaced at runtime.
- **`RUST_LOG`**: Standard `env_logger` log level control (e.g., `debug`, `info`).

## Configuration

- Custom Zenoh config files are JSON5 format, passed via `-c <file>`.
- `config/default.json5` is a reference config (peer mode, multicast scouting enabled).
- The binary always enables plugins loading, storage manager plugin, timestamping, and admin space metadata via `set_required_options`.
- Top-level flags (`-m`, `-e`, `-l`, `-r`, `--admin`, `-n`, `--no-multicast-scouting`) override config after loading.

## Useful one-liners

```bash
# Visualize the routing graph
cargo run -- graph | dot -Tpng -o graph.png

# Run a Python-powered queryable
cargo run -- queryable my/key/expr script.py --script

# Publish 10 messages every second
cargo run -- publish --count 10 --period 1000 my/key "Message {N}"
```
