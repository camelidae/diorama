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

Both executables are 100% statically linked (`x86_64-unknown-linux-musl`) and compressed with UPX (`--best --lzma`), requiring no host `glibc`:

- **`bin/diorama`**: Single-node Kubernetes distribution daemon compiled with `--features diorama/oci-cri` (embedded `oci-cri` container runtime, embedded MVCC/WAL datastore, API server, controller manager, kubelet, proxy, CoreDNS, and `local-path` storage).
- **`bin/dioramactl`**: Operator CLI for host readiness checks (`check`), offline bundle installation (`install`), add-on management (`addon load`), upgrades (`upgrade`), and cleanup (`reset` / `uninstall`).

## Programmatic Usage

```js
import { getBinaryPath } from "@diorama/amd64";

const dioramaBin = getBinaryPath("diorama");
const dioramactlBin = getBinaryPath("dioramactl");
```

See [`@diorama/cli`](https://www.npmjs.com/package/@diorama/cli) and the [GitHub repository](https://github.com/geoffsee/diorama) for full CLI documentation.

## License

Apache-2.0
