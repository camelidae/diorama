# Using the preview1 WASI handler

`oci-cri` can run a WASI preview1 module as a CRI container. The runtime name
stays `oci-cri`. The handler name is `zwasm`. The pod sandbox, pause
container, and CNI stay on the native engine. Only the workload container
takes the WASM path.

Diorama's default build does not include this handler. A managed
`diorama --container-runtime=oci-cri` process classifies a WASM workload and
then refuses it. Run a separately built `oci-cri` and point diorama at its
socket.

A Linux preview1 lifecycle has not been run in this repository. The steps
below are the contract the runtime implements. Treat a successful pod as
proof you still have to collect on the node you care about.

## Build the runtime

Zig 0.16 is required on `PATH` while compiling. The node that runs the binary
does not need Zig. `zwasm-sys` invokes the command named `zig` and has no
setting for another compiler path.

```sh
zig version   # 0.16.x
cargo build -p oci-cri --release --features zwasm
```

Build that from the [oci-lib](https://github.com/geoffsee/oci-lib) checkout,
not from the diorama workspace. Diorama pins `oci-cri` with
`default-features = false` and does not enable `zwasm`.

The guest joins the pause container's network namespace with `setns`, which
is Linux-only. A macOS release link shows that Zig and the SDK produce a
binary. It does not run a module inside a pod network.

Do not install `containerd-shim-zwasm`. This handler is inside `oci-cri`.

## Point diorama at that binary

Leave the managed backend alone. A managed `oci-cri` and `runtime.endpoint`
cannot be set together, and the managed binary is built without `zwasm`.

Start the feature-enabled daemon on Linux with its own socket, root, pause
image, and CNI directories. Then set diorama's `runtime.endpoint` to that
socket, for example `unix:///run/oci-cri/oci-cri.sock`. The advertised runtime
name remains `oci-cri`.

The podman-style engine is a different backend. It rejects any non-empty
runtime handler and does not create the pod.

## Select the handler

Create a RuntimeClass and name it from the pod. The kubelet reads
`node.k8s.io` / `runtimeclasses` and sends `.handler` on
`RunPodSandbox`. `system:node` is allowed to `get` that object. An empty
`.handler` fails sandbox creation. A pod with no `runtimeClassName` sends an
empty handler.

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: zwasm
handler: zwasm
---
apiVersion: v1
kind: Pod
metadata:
  name: hello-wasi
spec:
  runtimeClassName: zwasm
  containers:
    - name: app
      image: example/wasi-app:latest
      resources:
        limits:
          memory: 64Mi
```

Scheduling, overhead, and admission are unchanged. An unknown handler is
rejected. `zwasm` on a binary built without the feature is rejected with
`runtime handler "zwasm" requires oci-cri built with the zwasm feature`.

## When a container is WASM

The pause container is always native. A workload container is WASM when any
of these is true:

| Signal | Values |
| --- | --- |
| Sandbox handler | `zwasm` |
| Image config annotation `module.wasm.image/variant` | `compat` or `compat-smart` |
| Image OS | `wasip1`, `wasip2`, `wasip3`, `wasm`, or `wasi` |
| Image architecture | `wasm32` or `wasi` |
| Resolved argv[0] | ends in `.wasm` |

A later argument that happens to end in `.wasm` does not select the module.
`cp a.wasm /dst` stays native when argv[0] is `cp`.

The annotation is the image config annotation. Diorama does not copy pod
annotations onto the sandbox or the container.

Command precedence matches a native container:

- A non-empty `command` replaces the image entrypoint and command. `args` are
  appended.
- An empty `command` uses the image entrypoint, then `args` when those are
  set, otherwise the image command.

An image whose command is `app.wasm`, with pod args `/bin/sh` and no explicit
command, stays native: argv[0] is `/bin/sh`. An entrypoint that is still
`app.wasm`, with an empty handler, selects WASM. On a build without `zwasm`
that create fails closed.

Once the container is on the WASM path, argv[0] must be a `.wasm` file inside
the rootfs. The path is canonicalized and must stay inside the rootfs. A
missing file, a path that escapes the rootfs, or a non-regular file fails
create with `wasm container has no .wasm entrypoint` or `wasm module is not a
regular file inside the rootfs`.

At create, the runtime copies that file to `<root>/wasm/<id>.module` and
opens it with `O_NOFOLLOW`. Later starts read the copy. Remove deletes that
file when its path stays under the daemon `wasm` directory.

## What the guest receives

The module must export preview1 `_start`. zwasm-sdk 0.4 does not run the
component model. An image OS of `wasip2` or `wasip3` still selects the
container. Start prints that a component host is not wrapped and the process
exits 127.

The guest runs in a re-exec of the same `oci-cri` binary
(`OCI_CRI_ZWASM_RUN=1`). Stdout and stderr are copied into the CRI log.
Stdin is the spec, not an interactive attach stream.

- Args are the resolved command.
- Environment is the image environment, then the container environment. A
  container key replaces every earlier value of that key.
- Mounts are WASI preopens. The rootfs is preopened at `/` unless a mount
  already uses that guest path.
- A memory limit becomes a zwasm page cap. Pages are 64 KiB, rounded up. No
  limit leaves zwasm's default. The guest is not placed in a cgroup, so CPU
  limits and cgroup memory controls do not apply to it.
- A normal sandbox must have a running pause container with a network
  namespace. The guest joins that namespace before the engine is created.
  `hostNetwork` skips `setns`.

Stop sends SIGTERM, waits for the CRI timeout, then SIGKILL. A timeout of 0
or less sends SIGKILL immediately.

## What stays unimplemented

Exec, exec sync, and attach return unimplemented for a WASM container.
Port-forward is unchanged: it belongs to the sandbox network, not to a guest
shell.

There is no component-model host, no guest cgroup, no indexed WASM image
pull, and no `containerd-shim-zwasm`. Old container records with an empty
executor stay native.
