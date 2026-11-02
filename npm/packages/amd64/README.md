# `@diorama/amd64`

Linux `x86_64` (`amd64`) native binaries for [`@diorama/cli`](https://www.npmjs.com/package/@diorama/cli).

## Installation

Install the top-level [`@diorama/cli`](https://www.npmjs.com/package/@diorama/cli) package rather than installing `@diorama/amd64` directly. Your package manager will automatically select `@diorama/amd64` on Linux `x64` (`x86_64` / `amd64`) hosts via `optionalDependencies`:

```sh
npm install -g @diorama/cli
# or
bun add -g @diorama/cli
```

## Included Binaries

All executables are compressed with UPX (`--best --lzma`):

- **`bin/diorama`**: Ultra-lightweight (~8 MB) single-node Kubernetes distribution daemon (embedded MVCC/WAL datastore, API server, controller manager, kubelet, proxy, CoreDNS, and `local-path` storage).
- **`bin/dioramactl`**: Operator CLI for host readiness checks (`check`), offline bundle installation (`install`), add-on management (`addon load`), upgrades (`upgrade`), and cleanup (`reset` / `uninstall`).
- **`bin/oci-cri`**: Bundled CRI v1 container runtime (`geoffsee/oci-lib`) for zero-dependency execution over `/run/oci-cri/oci-cri.sock`.

## Programmatic Usage

```js
import { binDir, dioramaPath, dioramactlPath, ociCriPath } from "@diorama/amd64";
```

See [`@diorama/cli`](https://www.npmjs.com/package/@diorama/cli) and the [GitHub repository](https://github.com/geoffsee/diorama) for full CLI documentation.

## License

Apache-2.0
