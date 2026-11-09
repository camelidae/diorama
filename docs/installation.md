# Installation

Install prebuilt binaries and offline bundles from
[GitHub Releases](https://github.com/geoffsee/diorama/releases). Git and Rust are
not required. This guide uses [v0.1.3](https://github.com/geoffsee/diorama/releases/tag/v0.1.3);
keep the CLI, bundles, and checksums on the same release.

## Choose a platform

| Machine | v0.1.3 download | Where the node runs |
| --- | --- | --- |
| Linux x86_64 | `amd64` | Linux host or privileged container |
| Linux aarch64 / arm64 | `arm64` | Linux host or privileged container |
| macOS | No native release binary | Linux VM, using the VM's architecture |

On macOS, run the Linux installation steps inside a Linux VM. The published
Linux binaries do not run directly on macOS. Source development is covered in
[Development](development.md); other platform targets are listed in
[Project scope](scope.md#platforms).

## Install a container node

Use a Linux machine with a running Docker Engine, the Docker CLI, `curl`, `tar`,
and `sha256sum`. Run the installer on that machine. These commands use `sudo` for
access to the system Docker daemon; omit it if your user already has that access.

The node runs as a privileged container named `diorama`. Its persistent volume,
`diorama-data`, contains cluster credentials and workload data. Stopping the
container retains the volume.

Download and install in a directory for this release. Change `arch` to `arm64`
for an ARM64 machine:

```sh
(
  set -eu
  version=v0.1.3
  arch=amd64
  base="https://github.com/geoffsee/diorama/releases/download/$version"
  cli="dioramactl-linux-$arch"
  bundle="diorama-$version-linux-$arch-container-offline.tar.gz"

  for file in "$cli" "$bundle" SHA256SUMS; do
    curl --fail --location "$base/$file" --output "$file"
  done
  sha256sum --check --ignore-missing SHA256SUMS
  chmod +x "$cli"
  sudo "./$cli" install --run-mode=container --offline-install="./$bundle"
)
```

The block stops before installation if a download or checksum check fails.
`--ignore-missing` skips assets you did not download. If a checksum differs,
remove that download and retrieve it and `SHA256SUMS` again from the same release.
Report a persistent mismatch through [GitHub issues](https://github.com/geoffsee/diorama/issues).

The installer imports the bundled image, creates the container, and starts it.
The Kubernetes API is published on `127.0.0.1` with a dynamically allocated host
port. Inspect startup and the assigned port:

```sh
sudo docker logs diorama
sudo docker port diorama 6443/tcp
```

## Connect to the cluster

Install `kubectl` on the Linux machine. Once startup has created the node's
credentials, copy the kubeconfig and set its server address to the published port:

```sh
(
  set -eu
  umask 077
  api_endpoint=$(sudo docker port diorama 6443/tcp)
  sudo docker exec diorama cat /var/lib/diorama/pki/admin.kubeconfig > kubeconfig.yaml
  kubectl --kubeconfig ./kubeconfig.yaml config set-cluster diorama \
    --server="https://$api_endpoint"
  kubectl --kubeconfig ./kubeconfig.yaml get nodes
)
```

The kubeconfig contains administrator credentials; keep it private. If the file
has not been created yet or the API is unavailable, inspect `docker logs diorama`
and retry after startup completes. `get nodes` confirms API access and shows node
status; a running container alone does not establish node readiness.

## Extract a host bundle

Download the matching `dioramactl` binary and
`diorama-<version>-linux-<arch>-host-offline.tar.gz`, and verify them against the
release's `SHA256SUMS`. The installer checks the host architecture and libc.
For the v0.1.3 amd64 assets, after making the CLI executable:

```sh
./dioramactl-linux-amd64 install \
  --offline-install=./diorama-v0.1.3-linux-amd64-host-offline.tar.gz --path=./node
```

This extracts files into `./node` with the invoking user's ownership. It does
**not** register or start a system service. A protected destination directory
requires appropriate filesystem privileges.

## Add-ons and upgrades

Portainer and D2K are separate add-on packs. Load the pack, provide the required
credentials, and apply its installation Job as described in the
[v0.1.3 add-on guide](https://github.com/geoffsee/diorama/blob/v0.1.3/crates/dioramactl/README.md#installing-add-ons).
Portainer requires credentials for your Portainer server.

For upgrade commands, bundle formats, and data handling, use the
[release management guide](https://github.com/geoffsee/diorama/blob/v0.1.3/crates/dioramactl/README.md).
When choosing another release, read the guide at that tag.

[Back to documentation](../README.md#documentation)
