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
builds and tests; running a node requires a Linux environment.

After the node has created its credentials, export the kubeconfig and inspect it
with an installed `kubectl`:

```sh
umask 077
make kubeconfig
kubectl --kubeconfig=kubeconfig.yaml get nodes
```

`make kubeconfig` writes `kubeconfig.yaml` and sets the API endpoint to
`https://127.0.0.1:6443`. To stop and remove the container:

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
