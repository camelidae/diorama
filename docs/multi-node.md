# Multi-node development

The runtime has controller and worker join paths. The repository's
[muxox.toml](../muxox.toml) defines a two-controller environment using Podman:

| Node | Role | API endpoint | Data volume |
| --- | --- | --- | --- |
| `diorama-1` | Seed controller | `https://127.0.0.1:6443` | `diorama-data-1` |
| `diorama-2` | Joined controller | `https://127.0.0.1:6444` | `diorama-data-2` |

## How the setup works

Use a Linux environment with the [source-build prerequisites](development.md),
Podman, and Muxox. The configuration runs from the repository root and builds the
node image with `make image` before starting the services.

Both containers use the `diorama-net` network. The second waits for the seed,
creates a controller token through `dioramactl`, and passes it to the joining node
through `DIORAMA_JOIN_TOKEN`. Each node keeps its state in its own volume. The
stop hooks remove the containers and retain those volumes.

The setup exercises controller membership and replication. The worker join path
runs workload components without local API or controller services; it is not
included in this two-controller configuration. See [Architecture](architecture.md#controller-and-worker-roles)
for the role split.

## Current limits

This is a development environment. It does not provide a general-purpose
scheduler or a production HA deployment recipe. Workload placement and failure
recovery need to be considered separately from joining nodes.

## Working on multi-node behavior

Relevant integration tests are grouped by component:

- [Datastore replication and joins](../crates/diorama-datastore/tests/raft_quic_ha.rs)
- [Controller leader election](../crates/diorama-controller/tests/controller_ha_leader_election.rs)
- [Cross-node Pod access](../crates/diorama-kubelet/tests/kubelet_quic_ha.rs)

[Back to documentation](../README.md#documentation)
