# `@diorama/cli`

Single-node Kubernetes distribution daemon (`diorama`), operator CLI (`dioramactl`), and bundled `oci-cri` container runtime, packaged as Linux executables (`x86_64` / `amd64` and `arm64` / `aarch64`) for zero-dependency execution.

## Overview

`diorama` is an ultra-lightweight (~8 MB) single-node Kubernetes distribution written in Rust. `@diorama/cli` ships three executables via platform-specific optional dependencies:

- **`diorama`**: The single-node Kubernetes daemon and startup adapter. It validates node configuration (`diorama.io/v1alpha1`), prepares the Linux host and network environment, and supervises the embedded control plane and node components:
  - Embedded MVCC/WAL datastore
  - Kubernetes API server (`v1.37.1` discovery/protobuf schema)
  - Controller manager (`Deployment`, `ReplicaSet`, `StatefulSet`, `DaemonSet`, `Job`, `Endpoints`, `EndpointSlice`, garbage collection, namespace lifecycle)
  - Kubelet (`dioramalet`), `kube-proxy`, CoreDNS, and `local-path` storage provisioner
- **`dioramactl`**: The operator CLI for host readiness checks (`check`), prerequisite preparation, offline bundle installation (`install`), add-on loading (`addon load`), transactional upgrades (`upgrade`), and cleanup (`reset` / `uninstall`).
- **`oci-cri`**: Prebuilt CRI v1 container runtime (`geoffsee/oci-lib`) automatically started by `@diorama/cli` on `/run/oci-cri/oci-cri.sock` when no external CRI runtime or override is specified.

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
| [`@diorama/amd64`](https://www.npmjs.com/package/@diorama/amd64) | `linux` | `x64` (`x86_64` / `amd64`) | `diorama` + `dioramactl` + `oci-cri` |
| [`@diorama/arm64`](https://www.npmjs.com/package/@diorama/arm64) | `linux` | `arm64` (`aarch64`) | `diorama` + `dioramactl` + `oci-cri` |
| [`@diorama/aarch64`](https://www.npmjs.com/package/@diorama/aarch64) | `linux` | `arm64` (`aarch64`) | `diorama` + `dioramactl` + `oci-cri` |

All distributed binaries are compressed with UPX (`--best --lzma`).

## Usage

### `diorama` (Node Daemon)

When invoked via `@diorama/cli`, `diorama` defaults `DIORAMA_CONTAINER_RUNTIME_ENDPOINT=unix:///run/oci-cri/oci-cri.sock` and starts the bundled `oci-cri` daemon automatically unless you pass read-only flags (`--version`, `--help`, `--print-config`) or override the runtime via `DIORAMA_CONTAINER_RUNTIME` / `--container-runtime-endpoint`.

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

# Override the container runtime mode explicitly (containerd or external CRI socket)
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
sudo dioramactl install --offline-install=./diorama-v0.1.1-linux-amd64-host-offline.tar.gz --path=/var/lib/diorama
dioramactl install --run-mode=container --offline-install=./diorama-v0.1.1-linux-amd64-container-offline.tar.gz

# Upgrade or clean up an installation
sudo dioramactl upgrade --version=v0.1.1 --offline-install=./diorama-v0.1.1-linux-amd64-host-offline.tar.gz
sudo dioramactl reset
sudo dioramactl uninstall
```

## Programmatic API

`@diorama/cli` exports helpers for resolving or spawning the platform binaries from Node.js or Bun:

```js
import {
  buildRuntimeEnv,
  resolveBinaryPath,
  resolvePlatformPackages,
  runBinary,
} from "@diorama/cli";

// Locate the absolute path to the native binaries for the current host
const dioramaPath = resolveBinaryPath("diorama");
const dioramactlPath = resolveBinaryPath("dioramactl");
const ociCriPath = resolveBinaryPath("oci-cri");

// Build environment variables with DIORAMA_CONTAINER_RUNTIME_ENDPOINT default
const env = buildRuntimeEnv("diorama", process.env);

// Spawn `diorama --version` and inherit stdio
runBinary("diorama", ["--version"]);
```

## Building and Publishing (Maintainers)

From the `npm/` directory (or via `make -C npm <target>` from the repository root):

```sh
# Run TypeScript typechecks and unit tests
make -C npm check

# Stage binaries from a release artifact directory (fetches pinned oci-cri and compresses with UPX)
make -C npm stage FROM_DIR=../release VERSION=0.1.1

# Dry-run or publish @diorama/amd64, @diorama/arm64, @diorama/aarch64, and @diorama/cli
make -C npm publish-dry-run
make -C npm publish
```

Tagged GitHub releases automatically assemble, verify, and publish all four packages with npm OIDC provenance (`npm publish --provenance`) via [`.github/workflows/release.yml`](https://github.com/geoffsee/diorama/blob/main/.github/workflows/release.yml).

## License

Apache-2.0
