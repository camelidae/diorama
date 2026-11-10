# Development

To use a published build, start with [Installation](installation.md).
The commands here run from a source checkout.

## Build from source

Install Git, Make, and the Rust toolchain specified by
[rust-toolchain.toml](../rust-toolchain.toml). Workspace settings and dependencies
are recorded in [Cargo.toml](../Cargo.toml) and [Cargo.lock](../Cargo.lock).

```sh
git clone https://github.com/geoffsee/diorama.git
cd diorama
make build
./target/debug/diorama --help
./target/debug/dioramactl --help
```

`make build` selects the LLVM backend for C dependency linking and builds both
CLIs. The executable paths above assume the default Cargo target directory.
The default workspace member is the `diorama` package; use `-p <package>` or
`--workspace` when running Cargo commands for other crates.

## Inspect configuration

```sh
./target/debug/diorama --print-config
```

This prints effective configuration without starting services. Settings take
precedence in this order, from lowest to highest:

1. Built-in defaults
2. Configuration file, defaulting to `/etc/diorama/config.yaml`
3. `DIORAMA_*` environment variables
4. Explicit command-line flags

A missing configuration file is allowed. Printed configuration may contain
credentials. See the [configuration reference](../crates/diorama-config/README.md)
and [startup behavior](../crates/diorama-kube/README.md) for details.

## Run a development image

The Makefile defaults to Podman. For Docker, set `CONTAINER_ENGINE=docker` on each
command, or export it for the shell session.

On Linux, build the image and start a node:

```sh
make image
make container-run
```

`make image` builds a Linux musl binary on the host. To compile inside a container
instead, use `make image-source`, which uses
[Dockerfile.source](../container/Dockerfile.source). macOS can be used for source
builds and tests. For a local macOS VM development build, see below; ClusterIP
Service routing still requires kube-proxy on a Linux node.

## macOS VM development build

On macOS, `make macos-release` builds `dist/macos/diorama` and its companion
`dist/macos/dioramactl`. The VM guest is embedded in `dioramactl`, and the two
executables must remain side by side. Start the node with no arguments and check
the VM prerequisites with:

```sh
dist/macos/diorama
dist/macos/dioramactl vm diagnose
```

The helper is supervised and shut down on a normal `diorama` exit. Force-killing
`diorama` can leave the helper and VM running; a later start may launch another
guest while the earlier helper remains alive. This development runtime hosts the
CRI guest, not the whole node. The guest configures bridge CNI in a quarter of
its NAT IPv4 subnet, excluding the quarter containing its own address, and
enables proxy ARP so the Mac can reach pod IPs without host routes. Exec, attach,
and other CRI streaming URLs bind the guest's NAT-facing address, so they are
reachable from the Mac without host-side rewriting; `kubectl exec` is verified.
`kubectl port-forward` remains unavailable because the API server currently
rejects the upgrade protocol used by the installed `kubectl`, before it reaches
the guest. This is a single-guest development setup: the selected pod range can
overlap addresses allocated by vmnet DHCP, and ClusterIP Services still need
kube-proxy inside Linux.

After the node has created its credentials, export the kubeconfig and inspect it
with an installed `kubectl`:

```sh
umask 077
make kubeconfig
kubectl --kubeconfig=kubeconfig.yaml get nodes
```

`make kubeconfig` writes `kubeconfig.yaml` and sets the API endpoint to
`https://127.0.0.1:6443`.

When the CRI advertises `wasmtime`, cluster start embeds and applies the
in-cluster OCI registry automatically. Confirm it after the node is Ready:

```sh
kubectl --kubeconfig=kubeconfig.yaml get pod,svc -n default oci-registry
```

`dioramactl wasm deploy` pushes WASI components to that registry as
`oci-registry:5000/...`. Rebuild and apply steps for standalone registry work
live under [container/wasip3/oci-registry](../container/wasip3/oci-registry/README.md).

To stop and remove the container:

```sh
make container-stop
```

The `diorama-data` volume is retained. See [Multi-node development](multi-node.md)
for the two-controller environment.

## Checks

The [Contributor Guide](../CONTRIBUTING.md#pull-request-checklist) describes
`make check` and its prerequisites. [Makefile](../Makefile) lists the individual
build, lint, test, and container targets.
For local probes and official Kubernetes runs, see
[Running conformance tests](conformance-tests.md).

[Back to documentation](../README.md#documentation)
