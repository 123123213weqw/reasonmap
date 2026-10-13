# Development policy

- Do not compile Rust on the local workstation. Use `WZU_Server` for all Cargo build/check/test/clippy/package verification commands.
- Before running, rsync the current checkout including uncommitted changes to `~/codex-build/reasonmap/`, excluding `.git/`, `target/`, `node_modules/`, `.env`, `.env.*`, keys, and other secrets.
- Return full compiler/test logs. If the server is unavailable, report the block; do not fall back locally. Local `cargo fmt` is allowed.
- Never commit API keys, provider response bodies, or private application settings. Live tests are opt-in and must preserve per-case evidence grades.
