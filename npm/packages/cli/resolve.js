import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";

const require = createRequire(import.meta.url);
const PACKAGES_DIR = resolve(import.meta.dirname, "..");

export const DEFAULT_CONTAINER_RUNTIME = "oci-cri";
export const DEFAULT_OCI_CRI_SOCKET = "/run/oci-cri/oci-cri.sock";
export const DEFAULT_CONTAINER_RUNTIME_ENDPOINT = `unix://${DEFAULT_OCI_CRI_SOCKET}`;

const READ_ONLY_FLAGS = new Set([
  "--version",
  "-v",
  "--help",
  "-h",
  "--help-long",
  "--help-man",
  "--print-config",
]);

/**
 * Mapping from `${process.platform}-${process.arch}` to candidate `@diorama/*` packages
 * and local workspace fallback directories under `npm/packages/`.
 */
export const PLATFORM_PACKAGES = Object.freeze({
  "linux-x64": Object.freeze({
    packages: Object.freeze(["@diorama/amd64"]),
    dirs: Object.freeze(["amd64"]),
  }),
  "linux-arm64": Object.freeze({
    packages: Object.freeze(["@diorama/arm64", "@diorama/aarch64"]),
    dirs: Object.freeze(["arm64", "aarch64"]),
  }),
});

/**
 * @param {string} [platform]
 * @param {string} [arch]
 * @returns {readonly string[]}
 */
export function resolvePlatformPackages(
  platform = process.platform,
  arch = process.arch,
) {
  const key = `${platform}-${arch}`;
  const entry =
    PLATFORM_PACKAGES[/** @type {keyof typeof PLATFORM_PACKAGES} */ (key)];
  if (!entry) {
    throw new Error(
      `Unsupported platform/architecture "${key}". @diorama/cli supports linux-x64 (@diorama/amd64) and linux-arm64 (@diorama/arm64, @diorama/aarch64).`,
    );
  }
  return entry.packages;
}

/**
 * Resolves the absolute path to a native binary (`diorama`, `dioramactl`, or `oci-cri`)
 * from the installed platform-specific optional dependency or local workspace.
 *
 * @param {"diorama" | "dioramactl" | "oci-cri"} binaryName
 * @param {string} [platform]
 * @param {string} [arch]
 * @returns {string}
 */
export function resolveBinaryPath(
  binaryName,
  platform = process.platform,
  arch = process.arch,
) {
  if (
    binaryName !== "diorama" &&
    binaryName !== "dioramactl" &&
    binaryName !== "oci-cri"
  ) {
    throw new Error(`Unknown binary "${binaryName}"`);
  }

  const key = `${platform}-${arch}`;
  const entry =
    PLATFORM_PACKAGES[/** @type {keyof typeof PLATFORM_PACKAGES} */ (key)];
  if (!entry) {
    throw new Error(
      `Unsupported platform/architecture "${key}". @diorama/cli supports linux-x64 (@diorama/amd64) and linux-arm64 (@diorama/arm64, @diorama/aarch64).`,
    );
  }

  for (const pkg of entry.packages) {
    try {
      const pkgJsonPath = require.resolve(`${pkg}/package.json`);
      const candidate = join(dirname(pkgJsonPath), "bin", binaryName);
      if (existsSync(candidate)) {
        return candidate;
      }
    } catch {
      // Try next candidate or workspace fallback
    }
  }

  for (const dir of entry.dirs) {
    const candidate = join(PACKAGES_DIR, dir, "bin", binaryName);
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error(
    `Could not locate native "${binaryName}" binary for "${key}". Expected one of [${entry.packages.join(", ")}] to be installed with bin/${binaryName}.`,
  );
}

/**
 * Returns true if the CLI invocation only inspects version, help, or effective config
 * and must not start any background daemon or mutate host state.
 *
 * @param {readonly string[]} args
 * @returns {boolean}
 */
export function isReadOnlyInvocation(args) {
  return args.some((arg) => READ_ONLY_FLAGS.has(arg));
}

/**
 * Returns true if the caller explicitly selected a container runtime or endpoint
 * via CLI flags or environment variables.
 *
 * @param {readonly string[]} args
 * @param {NodeJS.ProcessEnv} [env]
 * @returns {boolean}
 */
export function hasExplicitRuntimeSelection(args, env = process.env) {
  if (env.DIORAMA_CONTAINER_RUNTIME || env.DIORAMA_CONTAINER_RUNTIME_ENDPOINT) {
    return true;
  }
  return args.some(
    (arg) =>
      arg === "--container-runtime" ||
      arg.startsWith("--container-runtime=") ||
      arg === "--container-runtime-endpoint" ||
      arg.startsWith("--container-runtime-endpoint="),
  );
}

/**
 * Builds the environment for spawning a Diorama binary, defaulting
 * `DIORAMA_CONTAINER_RUNTIME_ENDPOINT` to `"unix:///run/oci-cri/oci-cri.sock"`
 * when launching `diorama` unless an explicit runtime or endpoint is set.
 *
 * @param {"diorama" | "dioramactl" | "oci-cri"} binaryName
 * @param {NodeJS.ProcessEnv} [baseEnv]
 * @param {string} [socketPath]
 * @returns {NodeJS.ProcessEnv}
 */
export function buildRuntimeEnv(
  binaryName,
  baseEnv = process.env,
  socketPath = DEFAULT_OCI_CRI_SOCKET,
) {
  if (
    binaryName === "diorama" &&
    !baseEnv.DIORAMA_CONTAINER_RUNTIME &&
    !baseEnv.DIORAMA_CONTAINER_RUNTIME_ENDPOINT
  ) {
    return {
      ...baseEnv,
      DIORAMA_CONTAINER_RUNTIME_ENDPOINT: `unix://${socketPath}`,
    };
  }
  return baseEnv;
}

/**
 * Spawns the resolved native binary with inherited stdio and exits with its status code.
 * When launching `diorama` without read-only flags or explicit runtime overrides,
 * automatically starts the bundled `oci-cri` daemon on the CRI Unix socket.
 *
 * @param {"diorama" | "dioramactl" | "oci-cri"} binaryName
 * @param {readonly string[]} [args]
 * @returns {never}
 */
export function runBinary(binaryName, args = process.argv.slice(2)) {
  const socketPath =
    process.env.DIORAMA_OCI_CRI_SOCKET || DEFAULT_OCI_CRI_SOCKET;

  /** @type {import("node:child_process").ChildProcess | null} */
  let ociCriProc = null;

  if (
    binaryName === "diorama" &&
    !isReadOnlyInvocation(args) &&
    !hasExplicitRuntimeSelection(args, process.env)
  ) {
    validateCriSocketPath(socketPath);
    try {
      prepareCriSocketDirectory(socketPath);
      if (!existsSync(socketPath)) {
        const ociCriBin = resolveBinaryPath("oci-cri");
        ociCriProc = spawn(ociCriBin, ["serve", "--socket", socketPath], {
          stdio: "ignore",
        });
      }
    } catch {
      // Proceed and let diorama report the CRI readiness diagnostic if needed
    }
  }

  const binaryPath = resolveBinaryPath(binaryName);

  try {
    const result = spawnSync(binaryPath, args, {
      stdio: "inherit",
      env: buildRuntimeEnv(binaryName, process.env, socketPath),
    });

    if (result.error) {
      throw result.error;
    }

    if (result.signal) {
      process.kill(process.pid, result.signal);
    }

    process.exit(result.status ?? 1);
  } finally {
    if (ociCriProc && !ociCriProc.killed) {
      try {
        ociCriProc.kill("SIGTERM");
      } catch {
        // Ignore cleanup errors
      }
    }
  }
}

const WORLD_WRITABLE_DIRECTORIES = Object.freeze([
  "/tmp",
  "/var/tmp",
  "/dev/shm",
  "/private/tmp",
  "/private/var/tmp",
]);

/**
 * Validates CRI Unix domain socket path for security constraints:
 * - Must be an absolute path (rejects relative paths)
 * - Must not be located inside world-writable temporary directories (/tmp, /var/tmp, /dev/shm)
 * - Traverses ancestors and the socket path itself via lstatSync to reject symlinks (O_NOFOLLOW equivalent)
 *
 * @param {string} socketPath
 * @param {{ fs?: typeof import("node:fs"); expectedUid?: number }} [options]
 */
export function validateCriSocketPath(socketPath, options = {}) {
  if (typeof socketPath !== "string" || socketPath.trim().length === 0) {
    throw new Error("CRI socket path must be a non-empty string");
  }

  if (!socketPath.startsWith("/")) {
    throw new Error(`CRI socket path must be an absolute path: ${socketPath}`);
  }

  const normalized = resolve(socketPath);

  for (const dir of WORLD_WRITABLE_DIRECTORIES) {
    if (normalized === dir || normalized.startsWith(dir + "/")) {
      throw new Error(
        `CRI socket path cannot be located in world-writable directory: ${socketPath}`,
      );
    }
  }

  const fs = options.fs || process.getBuiltinModule("node:fs");

  // Inspect socket path itself if it exists (O_NOFOLLOW check)
  try {
    const socketStat = fs.lstatSync(normalized);
    if (socketStat.isSymbolicLink()) {
      throw new Error(`CRI socket path cannot be a symlink: ${socketPath}`);
    }
  } catch (err) {
    if (err && err.code !== "ENOENT") {
      throw err;
    }
  }

  // Inspect ancestor and parent directories for symlinks and world-writable bits
  let current = dirname(normalized);
  while (current && current !== "/" && current !== ".") {
    if (
      process.platform === "darwin" &&
      (current === "/var" || current === "/etc")
    ) {
      break;
    }
    try {
      const dirStat = fs.lstatSync(current);
      if (dirStat.isSymbolicLink()) {
        throw new Error(
          `CRI socket parent directory component cannot be a symlink: ${current}`,
        );
      }
      if ((dirStat.mode & 0o002) !== 0) {
        throw new Error(
          `CRI socket path cannot be located in world-writable directory: ${current}`,
        );
      }
    } catch (err) {
      if (err && err.code !== "ENOENT") {
        throw err;
      }
    }
    const parent = dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }
}

/**
 * Prepares and validates the parent directory and file permissions for a CRI Unix socket:
 * - Validates path safety via validateCriSocketPath
 * - Ensures parent directory is created with mode 0700
 * - Verifies parent directory ownership (uid === expectedUid || uid === 0 when running as root)
 * - Verifies parent directory permissions ((stat.mode & 0o022) === 0, no group/other write bits;
 *   and (stat.mode & 0o077) === 0 after chmodSync(dir, 0o700))
 * - If socket file already exists, verifies it is not a symlink, is owned by expectedUid / root,
 *   and enforces mode 0600 (chmodSync(socketPath, 0o600))
 *
 * @param {string} socketPath
 * @param {{ fs?: typeof import("node:fs"); expectedUid?: number; expectedSocketUid?: number }} [options]
 */
export function prepareCriSocketDirectory(socketPath, options = {}) {
  validateCriSocketPath(socketPath, options);

  const fs = options.fs || process.getBuiltinModule("node:fs");
  const normalized = resolve(socketPath);
  const dir = dirname(normalized);

  const currentUid =
    typeof process.getuid === "function" ? process.getuid() : 0;
  const expectedUid =
    options.expectedUid !== undefined ? options.expectedUid : currentUid;
  const expectedSocketUid =
    options.expectedSocketUid !== undefined
      ? options.expectedSocketUid
      : expectedUid;

  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
  }

  const dirStat = fs.lstatSync(dir);
  if (dirStat.isSymbolicLink()) {
    throw new Error(`CRI socket parent directory cannot be a symlink: ${dir}`);
  }

  // Ownership verification
  if (typeof dirStat.uid === "number") {
    const isRootExpected = expectedUid === 0 || currentUid === 0;
    if (isRootExpected && dirStat.uid !== 0) {
      throw new Error(
        `Insecure CRI socket parent directory ownership: uid is ${dirStat.uid}, expected root (0)`,
      );
    }
    if (dirStat.uid !== expectedUid && dirStat.uid !== 0) {
      throw new Error(
        `Insecure CRI socket parent directory ownership: uid is ${dirStat.uid}, expected ${expectedUid} or 0`,
      );
    }
  }

  // Reject group-writable or other/world-writable permissions
  if ((dirStat.mode & 0o022) !== 0) {
    throw new Error(
      `Insecure CRI socket parent directory permissions: ${dir} has group/other write permissions (mode 0o${(dirStat.mode & 0o777).toString(8)})`,
    );
  }

  // Enforce mode 0700
  fs.chmodSync(dir, 0o700);

  const updatedDirStat = fs.lstatSync(dir);
  if ((updatedDirStat.mode & 0o077) !== 0) {
    throw new Error(
      `Insecure CRI socket parent directory permissions: expected mode 0700 but got 0o${(updatedDirStat.mode & 0o777).toString(8)}`,
    );
  }

  // If socket file already exists or after binding, verify via lstatSync and enforce mode 0600
  try {
    const socketStat = fs.lstatSync(socketPath);
    if (socketStat.isSymbolicLink()) {
      throw new Error(`CRI socket path cannot be a symlink: ${socketPath}`);
    }

    if (typeof socketStat.uid === "number") {
      const isRootExpected = expectedSocketUid === 0 || currentUid === 0;
      if (isRootExpected && socketStat.uid !== 0) {
        throw new Error(
          `Insecure CRI socket file ownership: uid is ${socketStat.uid}, expected root (0)`,
        );
      }
      if (socketStat.uid !== expectedSocketUid && socketStat.uid !== 0) {
        throw new Error(
          `Insecure CRI socket file ownership: uid is ${socketStat.uid}, expected ${expectedSocketUid} or 0`,
        );
      }
    }

    fs.chmodSync(socketPath, 0o600);

    const updatedSocketStat = fs.lstatSync(socketPath);
    if ((updatedSocketStat.mode & 0o077) !== 0) {
      throw new Error(
        `Insecure CRI socket file permissions: expected mode 0600 but got 0o${(updatedSocketStat.mode & 0o777).toString(8)}`,
      );
    }
  } catch (err) {
    if (err && err.code !== "ENOENT") {
      throw err;
    }
  }
}
