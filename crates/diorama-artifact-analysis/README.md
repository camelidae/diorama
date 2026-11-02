# `diorama-artifact-analysis`

Development-time CLI binary that runs three Rust binary-size and unused-dependency analyzers (`cargo bloat`, `cargo udeps`, and `twiggy`) and combines their output into a single consolidated Markdown report.

## Tools Executed

1. **[`cargo-bloat`](https://github.com/RazrFalcon/cargo-bloat)**
   - `cargo bloat --release --crates -n 15` — Measures `.text` section size contribution per crate in the release binary.
   - `cargo bloat --release -n 15` — Lists the largest individual monomorphized functions in `.text`.
2. **[`cargo-udeps`](https://github.com/est31/cargo-udeps)**
   - `cargo udeps --all-targets` — Inspects Cargo compilation metadata across all targets to detect declared dependencies in `Cargo.toml` that are never referenced.
3. **[`twiggy`](https://github.com/rustwasm/twiggy)**
   - `twiggy top -n 15 <binary>` — Code-size profiler reporting shallow and retained byte sizes of symbols/items.
   - `twiggy dominators -r 15 <binary>` — Displays the call-graph dominator tree showing why large symbols are retained.

## Installation

Install the three external tools onto your `PATH` before running `diorama-artifact-analysis`:

```sh
cargo install cargo-bloat cargo-udeps twiggy
```

## Usage

Run from the root of the workspace (or any Rust package directory) with zero arguments:

```sh
CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo run --locked -p diorama-artifact-analysis
```

The binary prints the consolidated Markdown report to `stdout` and writes a copy to `./diorama-artifact-report.md`.

## Example Output

```markdown
# Diorama Artifact Analysis Report

- **Project Root**: `/Volumes/safe-vol/workspace/scratch/diorama`
- **Release Artifact**: `/Volumes/safe-vol/workspace/scratch/diorama/target/release/diorama` (~11 MiB)

## Executive Summary

| Check | Command | Status |
| :--- | :--- | :--- |
| 1a. Binary Size by Crate (`cargo bloat --crates`) | `cargo bloat --release --crates -n 15` | OK (exit 0) |
| 1b. Binary Size by Function (`cargo bloat`) | `cargo bloat --release -n 15` | OK (exit 0) |
| 2. Unused Dependencies (`cargo udeps`) | `cargo udeps --all-targets` | OK (exit 0) |
| 3a. Shallow & Retained Code Size (`twiggy top`) | `twiggy top -n 15 target/release/diorama` | OK (exit 0) |
| 3b. Call-Graph Dominators (`twiggy dominators`) | `twiggy dominators -r 15 target/release/diorama` | OK (exit 0) |

### Largest Crates (Top 5)
- `1.2%  18.4% 1.4MiB          diorama_apiserver`
- `0.9%  13.1% 998KiB          quiche`
- `0.7%  10.8% 820KiB          tokio`
```

## Limitations

- **External tools on `PATH`**: If any of `cargo-bloat`, `cargo-udeps`, or `twiggy` is not installed, `diorama-artifact-analysis` records the error in that tool's section and continues running the remaining tools.
- **Release build symbols**: `cargo bloat` and `twiggy` produce the most actionable results when `[profile.release]` retains symbol or DWARF line tables (`debug = "line-tables-only"`, which is configured in the workspace root `Cargo.toml`). Stripped binaries (`[profile.dist]`) collapse symbol names into `<unknown>`.
- **Nightly toolchain for `cargo udeps`**: `cargo udeps` requires a nightly Rust toolchain (already configured via `rust-toolchain.toml` in this repository).
