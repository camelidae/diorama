# Contributing Guide

Welcome! We are glad that you want to contribute to diorama.

If setup steps, docs, or automation do not work as described, please
[open an issue](https://github.com/geoffsee/diorama/issues) and tell us what you expected.

## Ways to Contribute

We welcome:

* Implementation and bug fixes in Rust crates
* Compatibility fixtures and qualification evidence
* Documentation and README improvements
* CI, release, and repository tooling (`diorama-dev`, `.github/workflows`)

Not everything goes through a pull request. Use [GitHub issues](https://github.com/geoffsee/diorama/issues)
to discuss design, scope, and acceptance criteria before large changes.

## Find an Issue

Browse [open issues](https://github.com/geoffsee/diorama/issues). Describe the behavior
you intend to change and how you will validate it. Comment on an issue when you want to
take it so work is not duplicated.

## Development Environment Setup

Clone the repository and install the toolchain from [rust-toolchain.toml](rust-toolchain.toml).

```sh
git clone https://github.com/geoffsee/diorama.git
cd diorama
make build
```

Repository layout:

| Path | Role |
| --- | --- |
| `crates/diorama-kube` | Node executable (`diorama` package) |
| `crates/dioramactl` | Management CLI |
| `crates/diorama-*` | Component libraries |
| `crates/diorama-dev` | Maintenance binaries (policy, bundles, security) |
| `container/` | Runtime images and scripts |

Maintenance binaries live in `diorama-dev` and stay outside the node runtime dependency graph.
Preserve existing fixtures and independent evidence when changing behavior.

## Pull Request Checklist

Before you open or update a pull request, run local checks when you can:

```sh
make check
```

`make check` runs formatting, Clippy, the nextest CI profile, doctests, language policy,
and `cargo-deny`. Equivalent commands:

```sh
cargo fmt-check
cargo lint
cargo test-ci
cargo test --locked --workspace --doc --all-features
cargo run --locked -p diorama-dev --bin diorama-language-policy
cargo deny check
```

[CI](.github/workflows/ci.yml) on the default branch also runs release-profile builds of
`diorama` and `dioramactl`. [Security](.github/workflows/security.yml) and
[code quality](.github/workflows/code_quality.yml) workflows add further gates.

In your pull request, include:

* What behavior changed and why
* Links to related issues
* Commands or evidence used for validation
* Known gaps or follow-up work

Use the [pull request template](.github/pull_request_template.md) when it helps.

## Pull Request Lifecycle

Open a pull request against `main`. Address review feedback and keep CI green. Maintainers
merge when the change matches agreed scope and checks pass.

Some integration work needs Linux, Docker, or pinned tools described in crate or fixture
READMEs; note your environment in the PR when qualification is partial.
