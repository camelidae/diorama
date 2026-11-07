# Project scope

diorama develops a Kubernetes runtime for single-host deployments and multi-node
clusters. Its scope includes cluster APIs and controllers, durable state,
workload execution, networking, storage, and node lifecycle management.

## Deployment limits

| Area | Current boundary |
| --- | --- |
| Single-host placement | NodeSetter admission assigns workloads locally. |
| Multi-node clusters | Controller/worker joins, replication, leader election, and cross-node access are under development. Production HA and end-to-end recovery are not yet qualified. |
| Host installation | The documented v0.1.3 installer extracts files; it does not register or start a system service. |
| Kubernetes conformance | The [Kubernetes 1.35 checklist](conformance.md) lists the official requirements. Individual results have not yet been recorded there. |

Use [Installation](installation.md) for release-specific behavior and
[Multi-node development](multi-node.md) for the development environment.

## Platforms

The published-assets column refers to
[v0.1.3](https://github.com/geoffsee/diorama/releases/tag/v0.1.3).
Select binaries from the release you intend to install.

| Platform | Published in v0.1.3 | Broader target |
| --- | --- | --- |
| Linux amd64 and arm64 | Node and management binaries, host/container bundles, add-on packs | Linux nodes and management CLIs |
| Linux ARMv7 hard-float and riscv64 | No | Linux nodes |
| macOS amd64 and arm64 | No | Management CLIs and source development |
| Native Windows | No | Outside current scope |

The broader Linux target matrix includes glibc and musl. Targets describe intended
coverage, not available artifacts. The [asset catalog](../crates/diorama-assets/CATALOG.md)
records the detailed matrix and packaging requirements.

## Out of scope

- An in-tree general-purpose Kubernetes scheduler
- GPU support in the baseline distribution
- Native Windows node or management binaries
- Reimplementing containerd or other external container engines

[Back to documentation](../README.md#documentation)
