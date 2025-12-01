# Contributing to TUI Hacker News App

We welcome contributions! Please follow these guidelines to ensure a smooth process.

## Development Setup

1. **Install Rust**: Ensure you have the latest stable Rust toolchain installed.
2. **Clone the repo**: `git clone https://github.com/chunghha/tui_hn_app.git`

## Running Tests

We use standard `cargo test` along with property-based testing.

```bash
# Run all tests
cargo test

# Run property tests specifically
cargo test --test property_tests
```

## Benchmarking

Performance is critical. If you make changes to rendering or parsing logic, please run benchmarks:

```bash
cargo bench
```

We use `criterion` for benchmarking. Results will be saved in `target/criterion/`.

## Code Style

- Run `cargo fmt` before committing.
- Run `cargo clippy` to catch common mistakes.

## Pull Requests

- Create a new branch for your feature/fix.
- Ensure all tests pass.
- Add new tests if applicable.
- Open a PR against `main`.
