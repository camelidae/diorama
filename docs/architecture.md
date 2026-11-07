# Architecture

diorama runs cluster services and workloads under a dependency-aware supervisor.
The `diorama` executable starts the node; `dioramactl` handles installation,
bundles, upgrades, and administrative commands.

## Cluster services

| Component | Role |
| --- | --- |
| API server | Kubernetes API requests, discovery, authentication, authorization, and admission |
| Controllers | Reconcile workload and cluster resources; use leader election when running together |
| Datastore | Store cluster objects with revisions, watches, write-ahead logging, and snapshots; replicate state between peers using Raft over QUIC |
| PKI | Create and manage cluster credentials and join material |
| Supervisor | Order component startup, track readiness, and coordinate shutdown |

These services are implemented in Rust. Component documentation is linked from
[Resources](resources.md).

## Workload execution

The kubelet reconciles Pods assigned to its node and communicates with a container
runtime through CRI. diorama can manage containerd or connect to a host-managed
CRI endpoint. Networking connects Pods and implements Service routing; DNS and
local-path storage provide name resolution and persistent volumes.

In single-host mode, NodeSetter admission assigns workloads to the local node.
The project does not include a general-purpose scheduler. Adding nodes alone
does not provide automatic workload placement across them.

## Controller and worker roles

A single-host deployment combines cluster services and workload execution. The
multi-node runtime has separate join paths:

- **Controller:** joins the datastore peer group and runs API and controller
  services. A controller can also run the workload components.
- **Worker:** connects to the remote datastore and runs workload components
  without starting its own API server or controller manager.

Peer communication uses QUIC. Cross-node routing allows Pod logs and exec requests
to reach the node running the workload. See [Multi-node development](multi-node.md)
for the two-controller environment and current limits.

## Configuration and state

The node reads `diorama.io/v1alpha1` configuration and combines defaults, file
settings, environment variables, and command-line flags. Configuration controls
the runtime, networking, storage, and optional components. See
[configuration inspection](development.md#inspect-configuration) for precedence
and the default file location.

A container deployment keeps node state under `/var/lib/diorama` in its data
volume. This includes cluster credentials and persistent state. The container
image and node data have separate lifecycles: stopping a container retains its
volume.

[Back to documentation](../README.md#documentation)
