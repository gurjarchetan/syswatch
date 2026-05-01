# Contributing to SysWatch

Thank you for taking the time to contribute! Every bug report, feature idea, and pull request makes SysWatch better.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Report a Bug](#how-to-report-a-bug)
- [How to Request a Feature](#how-to-request-a-feature)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Development Setup](#development-setup)
- [Coding Conventions](#coding-conventions)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating you agree to abide by its terms.

---

## Getting Started

1. **Fork** the repository and create your branch from `main`.
2. Follow the [Development Setup](#development-setup) steps below.
3. Make your changes, ensuring all existing tests pass (when applicable).
4. Open a pull request using the provided template.

---

## How to Report a Bug

Use the **Bug Report** issue template. Include:

- SysWatch version (`syswatch --version`)
- OS and kernel (`uname -a`, `cat /etc/os-release`)
- Terminal emulator and size
- Steps to reproduce the issue
- What you expected to happen vs. what actually happened
- Relevant error output or screenshots

---

## How to Request a Feature

Use the **Feature Request** issue template. Describe:

- The problem you are trying to solve
- Your proposed solution
- Any alternatives you considered

---

## Submitting a Pull Request

1. Keep PRs focused — one feature or fix per PR.
2. Write a clear title and description (the PR template will guide you).
3. Reference any related issues (e.g., `Closes #42`).
4. Ensure `cargo build --release` compiles cleanly with no new errors.
5. Run `cargo clippy` and fix any warnings in code you touched.
6. Run `cargo fmt` before committing.

---

## Development Setup

**Prerequisites:** Rust 1.75+ (install via [rustup](https://rustup.rs))

```bash
git clone https://github.com/gurjarchetan/syswatch.git
cd syswatch

# Debug build (fast compile, useful during development)
cargo build

# Run directly
cargo run

# Release build
cargo build --release

# Lint
cargo clippy

# Format
cargo fmt
```

The project uses **tokio** for async I/O, **ratatui** for the TUI, and **sysinfo** + raw `/proc` reads for data collection. Familiarise yourself with `src/collector/` (data layer) and `src/ui/` (render layer) before diving in.

---

## Coding Conventions

| Rule | Detail |
|---|---|
| **Edition** | Rust 2021 |
| **Formatting** | `cargo fmt` (default style) |
| **Linting** | `cargo clippy -- -D warnings` on new code |
| **Layer separation** | `collector/` must not import from `ui/`; share state only via `Arc<RwLock<SystemState>>` |
| **No blocking calls** | All I/O must be async or spawned to a blocking thread pool |
| **Error handling** | Use `anyhow::Result` for collector code; propagate with `?` |

---

Questions? Open a [Discussion](https://github.com/gurjarchetan/syswatch/discussions) or email **chetan04.2014@gmail.com**.
