# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` starts the MCP server; `src/lib.rs` wires modules for library use.
- Core behavior: `src/server.rs` (service setup), `src/executor.rs` (command runner), `src/resources.rs` (documentation resources).
- Tools live under `src/tools/` (build, call, convert, install, up) with shared helpers in `src/tools/common.rs`.
- Integration tests mirror tool layout in `tests/tools/` and are gated by the `pop-e2e` feature because they call Pop CLI and start ink! nodes.
- Build artifacts land in `target/`; CI configuration is in `Dockerfile.ci`.

## Build, Test, and Development Commands
- `cargo +nightly fmt` – Format all Rust sources.
- `cargo clippy --all-features --all-targets` – Lint and fail on warnings; run before PRs.
- `cargo build` / `cargo build --release` – Compile the server (release binary at `target/release/pop-mcp-server`).
- `cargo run` – Run the MCP server from source.
- `cargo test --features pop-e2e` – Integration tests (`/tests`) that invoke Pop CLI using `pop`, use `pop --help` to get a feel for it.

## Coding Style & Naming Conventions
- Rust 2021 edition; use rustfmt defaults (4-space indent, trailing commas).
- Modules and files use snake_case; types and traits use PascalCase; functions and variables use snake_case.
- Add `anyhow::Context` on fallible paths; return `Result` from async entrypoints.
- Keep tool output deterministic and human-readable; avoid panics in tool code paths.

## Testing Guidelines
- Default to `cargo test` and individual tests within the `/tests` folder. Only at the end you can test all integration tests `cargo test --features pop-e2e`.
- Add fast unit tests inline with `#[cfg(test)]` when Pop CLI is not required; keep `cargo test` without features quick.
- Use descriptive test names (e.g., `build_contract_rejects_missing_path`).
- Use simple asserts without custom failure messages; let test names and assertion expressions speak for themselves.
- Make sure each tool has an integration test using the Pop CLI.

## Commit & Pull Request Guidelines
- Follow the existing conventional-style prefixes (e.g., `fix: …`, `refactor: …`); keep subjects imperative and ≤72 characters.
- PRs should include a short summary, linked issues, test commands/results, and notes about Pop CLI or network prerequisites.
