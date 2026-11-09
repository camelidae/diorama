# WASI 0.3 runtime

The development images bundle a separate `containerd-shim-wasmtime-v1` executable.
Diorama continues to use containerd and forwards a Pod's RuntimeClass handler to
CRI. Pods without a RuntimeClass continue to use the normal native runtime.
Wasmtime is not linked into Diorama.

Managed containerd configuration enables the handler only when the executable is
installed. Diorama also registers the `wasmtime` RuntimeClass when a connected
CRI runtime advertises that handler. Use `runtimeClassName: wasmtime` for WASI 0.3 command components.
Core modules (including P1) and P2 components are rejected. This runtime hosts
`wasi:cli/run@0.3.0` commands, including programs that open TCP sockets; it does
not host `wasi:http` incoming-handler components.

## Build

Use the existing `container/dev-image.sh` or `make image-source` workflow.
The first build compiles the shim in a Linux build stage; subsequent Diorama
edits reuse that stage. No host Wasmtime or zwasm installation is needed.

`build.sh` pins runwasi to `f1401719a3e981956ced9cd685abb2a49190bd1c` and applies
`runwasi.patch`, including its Cargo.lock. The patch upgrades Wasmtime to 50.0.1
and replaces the P1/P2/HTTP adapter with the P3 command adapter. The development
shim uses a stripped debug build to keep cold build costs lower. This is a local
patch, not functionality available in the unmodified upstream shim release.

Native Linux offline releases build the same static executable for amd64 and
arm64, include it in the checksummed installer bundle and link it into the runtime
search path. The containerd executable, runc shim and ctr are built alongside it
using `container/wasip3/Dockerfile`. Release bundles include the source build
scripts and patches as provenance; `release-inputs.json` describes downloaded
assets, excluding these locally built executables.

The macOS BusyBox guest is maintained in the sibling oci-lib repository. Its
`cargo xtask guest oci-cri` build includes `/usr/local/bin/oci-wasip3` and
`/etc/oci-cri/config.d/wasmtime.toml`. This separate static P3 runner executes
inside oci-cri's existing libcontainer isolation; the guest does not need
containerd or Youki. See `crates/oci-wasip3/README.md` in oci-lib for installation
and live CRI tests. Use the rebuilt oci-cri guest release, not an older guest
archive. The guest currently targets Apple Silicon (Linux arm64).

The two adapters use the same pinned Wasmtime version and P3 command interface,
but their hosting code is separate. No Wasmtime code is linked into Diorama or
oci-cri. The Runwasi shim itself uses Youki's libcontainer internally; that
library is not used by the standalone oci-cri runner.

## Smoke test

With a development container named `diorama` running and `KUBECONFIG` pointing at
its API server, build a real P3 component and import a `wasip3/wasm32` OCI image:

```sh
rustup target add --toolchain nightly-2026-10-12 wasm32-wasip3
rustc +nightly-2026-10-12 --edition=2024 --target wasm32-wasip3 -O \
  container/wasip3/probe.rs -o container/wasip3/probe.wasm
python3 container/wasip3/image.py container/wasip3/probe.wasm container/wasip3/probe.tar
podman cp container/wasip3/probe.tar diorama:/tmp/probe.tar
podman exec diorama ctr --address /var/lib/diorama/containerd/state/containerd.sock \
  --namespace k8s.io images import --all-platforms --no-unpack /tmp/probe.tar
kubectl apply -f container/wasip3/probe.yaml
kubectl wait --for=jsonpath='{.status.phase}'=Succeeded pod/p3-probe --timeout=120s
kubectl logs p3-probe
```

Expected output includes `hello from WASI 0.3`, `arg=from-pod`, `env=from-env`,
and `p3 stderr`. The container exit code is 0. Change `args` to `[exit]` in a new
pod to verify exit code 7. The fixture also accepts `file` (reads `/data/message`),
`http` (serves `wasip3!` on TCP port 8080), and `wait`.

Registry pulls support single manifests and OCI indices, including mixed native
Linux and `wasip3/wasm32` entries. `build-containerd.sh` pins upstream commit
`43aa4ea134f3e50791b8537c48a18303f0fcc775`, which includes runtime-aware pull
selection (upstream PR #14314). The checked-in `containerd.patch` additionally
preserves the selected platform through container creation/unpacking and rejects
wrong-platform cache hits. This is a temporary downstream patch: the inspected
stable releases did not provide the full behavior. Remove it only after the
mixed-index tests pass against a supported upstream release.

Diorama forwards the runtime handler on image requests and uses the immutable
image ID returned by CRI during creation, so pulling the other platform under
the same tag cannot change the selected workload. Native pods retain crun and
the native platform; WASI images are never relabeled as Linux.

## Wasm OCI registry

`container/wasip3/oci-registry` is an OCI Distribution `/v2/` server as a
`wasi:cli/run` component (TCP + filesystem; no WIT, no custom host). Deploy with
`runtimeClassName: wasmtime`. See `oci-registry/README.md`.

```sh
cd container/wasip3/oci-registry
cargo run --release -- --bind 127.0.0.1:5000 --data /tmp/oci-reg
# Wasm: cargo build --release --target wasm32-wasip3
```

For a disposable HTTP registry, run the in-tree server (same binary CI uses in place
of `registry:2`) and publish fixtures with `registry.py`:

```sh
cargo run --release --manifest-path container/wasip3/oci-registry/Cargo.toml \
  -- --bind 127.0.0.1:15000 --data /tmp/oci-reg
python3 container/wasip3/registry.py probe.tar native.tar http://127.0.0.1:15000
```

Configure that registry in containerd's `registry/HOST:PORT/hosts.toml`, then run:

```sh
python3 container/wasip3/smoke.py HOST:PORT/p3-probe
```

This tests single and indexed WASI tags with `Always`, then alternates WASI,
native and WASI under the same mixed tag, including `IfNotPresent` cache checks.

The smoke fixture uses a fractional CPU limit because the existing whole-core
CPU allocation status can contain an invalid Kubernetes quantity, which causes
`kubectl logs` to reject the Pod response before requesting its logs.

The shim tests included in the patch check P1/P2 rejection and a real P3 fixture
(including exit 7). In a Linux runwasi build tree, set `WASIP3_TEST_COMPONENT` to
an absolute fixture path and run:

```sh
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER=env \
WASIP3_TEST_COMPONENT=/tmp/probe.wasm \
  cargo test -p containerd-shim-wasmtime --lib -- --include-ignored
```

Use the corresponding `X86_64` runner variable on x86-64. The runner override
bypasses upstream's sudo wrapper when already running as root in a build container.

## Verified locally

Linux arm64 inside Podman: a fresh development image builds and starts with the
shim installed and creates its RuntimeClass. The fresh image passes single,
indexed and mixed-image tests, including alternating native and WASI with
`IfNotPresent`. Earlier execution tests cover
stdout/stderr, args, environment, exit statuses 0 and 7, a hostPath file mount,
TCP traffic to the pod IP, a 128 MiB memory cgroup and ordinary native containers.
Targeted shim, CRI, kubelet, containerd-image and packaging tests pass.

The standalone oci-cri path passes a live Linux test for single, indexed and
mixed WASI pulls, native execution from the same mixed tag, and switching back
to WASI. The arm64 BusyBox guest archive builds with the static runner and TOML, boots
under Apple Virtualization on macOS, advertises the handler through CRI Status,
and passes the same live image-selection and native-container tests.
Both Linux release architectures are configured and covered by installer layout
tests.

### x86-64 (this review)

| Check | arm64 | amd64 |
| --- | --- | --- |
| Static `oci-wasip3` build (`scripts/build-wasip3.sh` / CI) | verified | verified via `cargo zigbuild --target x86_64-unknown-linux-musl` on the macOS build host (no dynamic `INTERP`) |
| P3 probe stdout + exit 7 | verified (live CRI + k8s) | verified under `qemu-x86_64-static` in a Debian arm64 test container (not bare-metal amd64) |
| P1 core module rejection (`oci-wasip3`) | verified | verified (same qemu setup) |
| `containerd-shim-wasmtime-v1` P1/P2 unit tests | covered in patched Runwasi sources | same sources; not re-run on macOS (shim depends on Linux-only crates) |
| Full Diorama dev image + `smoke.py` | verified earlier | not re-run locally (Podman storage full pulling `linux/amd64` build images) |

Offline release installer layout and checksum verification use fixtures; a full
`diorama-release-bundle` install against downloaded production bytes is still
worth running in CI or on a machine with spare Podman disk.

## Release coupling (unpublished)

**Published** `geoffsee/oci-lib@v0.1.15` includes the WASI 0.3 runner and `wasmtime` handler.
The guest runner, `wasmtime.toml`, and CRI handler plumbing live only on branch
`feat/wasip3-runtime-plugin` in the oci-lib worktree until that branch is tagged.

| Artifact / path | Needs WASI changes from oci-lib? | Notes |
| --- | --- | --- |
| Diorama `diorama-oci` git pin (`crates/diorama-oci/Cargo.toml`) | done on bump branch | `v0.1.15` / `9e2140c330e58d4249321b41ad2b2e98ebbc83e9` |
| npm `OCI_LIB_VERSION` / publisher downloads | done on bump branch | `v0.1.15` tarballs ship `oci-wasip3` and `wasmtime.toml` |
| macOS BusyBox guest (`cargo xtask guest oci-cri`) | yes | Rebuild after oci-lib release; do not ship an older guest archive |
| Linux npm bundles (`@diorama/*`) | yes for optional bundled `oci-cri` | Same new oci-lib release |
| Diorama offline Linux bundles | no oci-lib crate dependency | `containerd-shim-wasmtime-v1`, `ctr`, patched containerd built from this repo’s `container/wasip3/` scripts |
| Runwasi / Wasmtime shim | self-contained | Pin + patch in `build.sh`; not a crates.io dependency |

Do not add absolute-path `path =` dependencies in Cargo.toml for release; publish
oci-lib first, then bump the git rev and `OCI_LIB_VERSION` together.

## Wasmtime hosting (Runwasi vs `oci-wasip3`)

Both use Wasmtime 50.0.1 and `wasi:cli/run@0.3.x`, but the Runwasi shim and the
standalone runner keep separate hosting code (~80 lines each, different I/O and
lifecycle boundaries). A shared library would cross-link the patched Runwasi tree
with oci-lib and coupling releases without removing much duplication; keep them
separate unless one side grows substantially.

## Limitations

- WASI 0.3 **command** components only; P1 modules and P2 worlds are rejected.
- No `wasi:http` incoming-handler hosting.
- Containerd stays on commit `43aa4ea…` plus `containerd.patch`; stable 2.2.x
  includes partial runtime-handler pull support (#12721) but not the full
  platform-preserving create/unpack/status behavior exercised by mixed-index
  tests. Do not drop the patch or relabel WASI images as `linux` to “use upstream.”
- macOS guest targets Linux **arm64** only today.
- Mixed OCI indices require `runtime_handler` on pull **and** create; clients must
  use the immutable image id returned by CRI (Diorama kubelet does).
