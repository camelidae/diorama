# `@diorama/cli`

Single-node Kubernetes distribution daemon (`diorama`) and operator CLI (`dioramactl`), packaged as statically linked Linux executables (`x86_64` / `amd64` and `arm64` / `aarch64`) with the embedded `oci-cri` container runtime compiled in.

## Overview

`diorama` is a single-node Kubernetes distribution written in Rust. It ships two executables:

- **`diorama`**: The single-node Kubernetes daemon and startup adapter. It validates node configuration (`diorama.io/v1alpha1`), prepares the Linux host and network environment, and supervises the embedded control plane and node components:
  - Embedded MVCC/WAL datastore
  - Kubernetes API server (`v1.37.1` discovery/protobuf schema)
  - Controller manager (`Deployment`, `ReplicaSet`, `StatefulSet`, `DaemonSet`, `Job`, `Endpoints`, `EndpointSlice`, garbage collection, namespace lifecycle)
  - Kubelet (`dioramalet`), `kube-proxy`, CoreDNS, and `local-path` storage provisioner
  - Built-in **`oci-cri`** container runtime (compiled via `--features diorama/oci-cri` and enabled by default in `@diorama/cli`), plus support for managed `containerd`/`crun` or an external CRI runtime socket
- **`dioramactl`**: The operator CLI for host readiness checks (`check`), prerequisite preparation, offline bundle installation (`install`), add-on loading (`addon load`), transactional upgrades (`upgrade`), and cleanup (`reset` / `uninstall`).

## Installation

Install `@diorama/cli` globally with npm, Bun, or pnpm on Linux (`x86_64` or `arm64` / `aarch64`):

```sh
npm install -g @diorama/cli
# or
bun add -g @diorama/cli
# or
pnpm add -g @diorama/cli
```

Your package manager automatically installs the matching platform binary package via `optionalDependencies`:

| Package | OS | CPU Architecture | Contents |
| --- | --- | --- | --- |
| [`@diorama/amd64`](https://www.npmjs.com/package/@diorama/amd64) | `linux` | `x64` (`x86_64` / `amd64`) | Static `musl` `diorama` (`oci-cri`) + `dioramactl` |
| [`@diorama/arm64`](https://www.npmjs.com/package/@diorama/arm64) | `linux` | `arm64` (`aarch64`) | Static `musl` `diorama` (`oci-cri`) + `dioramactl` |
| [`@diorama/aarch64`](https://www.npmjs.com/package/@diorama/aarch64) | `linux` | `arm64` (`aarch64`) | Static `musl` `diorama` (`oci-cri`) + `dioramactl` |

All distributed binaries are 100% statically linked against `musl` and compressed with UPX (`--best --lzma`), so they run on any Linux distribution (Ubuntu, Debian, Alpine, Fedora, RHEL, Arch, NixOS) without `glibc` version constraints.

## Usage

### `diorama` (Node Daemon)

When invoked via `@diorama/cli`, `diorama` defaults `DIORAMA_CONTAINER_RUNTIME=oci-cri` so the built-in `oci-cri` runtime is used automatically unless you override `DIORAMA_CONTAINER_RUNTIME` or pass `--container-runtime`.

```sh
# Print version or CLI help without starting services
diorama --version
diorama --help

# Print the resolved effective configuration (YAML) without mutating host state
diorama --print-config

# Start the single-node Kubernetes daemon (requires root on Linux)
sudo diorama

# Start with a custom configuration file
sudo diorama --config /etc/diorama/config.yaml

# Override the container runtime mode explicitly (oci-cri, containerd, or external)
sudo DIORAMA_CONTAINER_RUNTIME=containerd diorama
```

Configuration precedence follows:

```text
defaults < config file < DIORAMA_* environment variables < explicit CLI flags
```

### `dioramactl` (Operator CLI)

```sh
# Inspect CLI commands and flags
dioramactl --help
dioramactl version

# Run non-mutating host readiness preflight probes
sudo dioramactl check

# Install from a verified offline host or container bundle
sudo dioramactl install --offline-install=./diorama-v0.1.0-linux-amd64-host-offline.tar.gz --path=/var/lib/diorama
dioramactl install --run-mode=container --offline-install=./diorama-v0.1.0-linux-amd64-container-offline.tar.gz

# Upgrade or clean up an installation
sudo dioramactl upgrade --version=v0.1.0 --offline-install=./diorama-v0.1.0-linux-amd64-host-offline.tar.gz
sudo dioramactl reset
sudo dioramactl uninstall
```

## Programmatic API

`@diorama/cli` exports helpers for resolving or spawning the platform binaries from Node.js or Bun:

```js
import {
  buildCommandEnv,
  resolveBinaryPath,
  resolvePlatformPackage,
  runBinary,
} from "@diorama/cli";

// Locate the absolute path to the native binary for the current host
const dioramaPath = resolveBinaryPath("diorama");
const dioramactlPath = resolveBinaryPath("dioramactl");

// Build environment variables with DIORAMA_CONTAINER_RUNTIME="oci-cri" default
const env = buildCommandEnv("diorama", process.env);

// Spawn `diorama --version` and inherit stdio
const exitCode = runBinary("diorama", ["--version"]);
```

## Building and Publishing (Maintainers)

From the `npm/` directory (or via `make -C npm <target>` from the repository root):

```sh
# Run TypeScript typechecks and unit tests
make -C npm check

# Stage binaries from a release artifact directory (compresses with UPX if available)
make -C npm stage FROM_DIR=../dist VERSION=0.1.0

# Cross-compile static musl binaries from Cargo and stage all platform packages
make -C npm build VERSION=0.1.0

# Dry-run or publish @diorama/amd64, @diorama/arm64, @diorama/aarch64, and @diorama/cli
make -C npm publish-dry-run
make -C npm publish
```

Tagged GitHub releases automatically build, verify, and publish all four packages with npm OIDC provenance (`npm publish --provenance`) via [`.github/workflows/release.yml`](https://github.com/geoffsee/diorama/blob/main/.github/workflows/release.yml).

## License

Apache-2.0
