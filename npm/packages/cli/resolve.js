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
 * Verifies that a resolved candidate binary and all parent directories up to
 * allowedRootDir are secure against untrusted manipulation, symlink escapes,
 * and insecure permissions when running as root.
 *
 * @param {string} candidatePath
 * @param {string} allowedRootDir
 * @param {{ euid?: number }} [options]
 * @returns {string}
 */
export function verifyTrustedBinaryPath(
  candidatePath,
  allowedRootDir,
  options = undefined,
) {
  const fs = process.getBuiltinModule("node:fs");
  const path = process.getBuiltinModule("node:path");

  const absCandidate = path.resolve(candidatePath);
  const absAllowedRoot = path.resolve(allowedRootDir);

  const relFromRoot = path.relative(absAllowedRoot, absCandidate);
  if (relFromRoot.startsWith("..") || path.isAbsolute(relFromRoot)) {
    throw new Error(
      `Candidate binary "${candidatePath}" is outside allowed root "${allowedRootDir}".`,
    );
  }

  let realAllowedRoot;
  try {
    realAllowedRoot = fs.realpathSync(absAllowedRoot);
  } catch {
    realAllowedRoot = absAllowedRoot;
  }

  const isWithinAllowed = (target) => {
    const r1 = path.relative(realAllowedRoot, target);
    if (r1 === "" || (!r1.startsWith("..") && !path.isAbsolute(r1))) {
      return true;
    }
    const r2 = path.relative(absAllowedRoot, target);
    return r2 === "" || (!r2.startsWith("..") && !path.isAbsolute(r2));
  };

  const euid = options?.euid ?? process.geteuid?.() ?? -1;
  const isRoot = euid === 0;

  const chain = [];
  let current = absCandidate;
  while (true) {
    chain.push(current);
    if (current === absAllowedRoot) {
      break;
    }
    const parent = path.dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }

  const stats = [];
  for (const item of chain) {
    const stat = fs.lstatSync(item);
    stats.push({ item, stat });

    if (stat.isSymbolicLink()) {
      const realTarget = fs.realpathSync(item);
      if (!isWithinAllowed(realTarget)) {
        throw new Error(
          `Untrusted symlink detected: "${item}" resolves outside allowed root "${allowedRootDir}" to "${realTarget}".`,
        );
      }
      if (isRoot) {
        const targetStat = fs.statSync(item);
        if ((targetStat.mode & 0o022) !== 0) {
          throw new Error(
            `Insecure target permissions: "${item}" target "${realTarget}" is writable by group or others (mode: 0o${(targetStat.mode & 0o777).toString(8)}).`,
          );
        }
      }
    } else if (isRoot && (stat.mode & 0o022) !== 0) {
      throw new Error(
        `Insecure permissions: "${item}" is writable by group or others (mode: 0o${(stat.mode & 0o777).toString(8)}).`,
      );
    }
  }

  if (isRoot) {
    for (const { item, stat } of stats) {
      if (stat.isSymbolicLink()) {
        const targetStat = fs.statSync(item);
        if (targetStat.uid !== 0) {
          const realTarget = fs.realpathSync(item);
          throw new Error(
            `Untrusted target ownership: "${item}" target "${realTarget}" is owned by UID ${targetStat.uid}, expected root (0).`,
          );
        }
        if (stat.uid !== 0) {
          throw new Error(
            `Untrusted symlink ownership: "${item}" is owned by UID ${stat.uid}, expected root (0).`,
          );
        }
      } else if (stat.uid !== 0) {
        throw new Error(
          `Untrusted ownership: "${item}" is owned by UID ${stat.uid}, expected root (0).`,
        );
      }
    }
  }

  return candidatePath;
}

/**
 * Resolves the absolute path to a native binary (`diorama`, `dioramactl`, or `oci-cri`)
 * from the installed platform-specific optional dependency or local workspace.
 *
 * @param {"diorama" | "dioramactl" | "oci-cri"} binaryName
 * @param {string} [platform]
 * @param {string} [arch]
 * @param {{ euid?: number }} [options]
 * @returns {string}
 */
export function resolveBinaryPath(
  binaryName,
  platform = process.platform,
  arch = process.arch,
  options = undefined,
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
    let pkgJsonPath;
    try {
      pkgJsonPath = require.resolve(`${pkg}/package.json`, {
        paths: [import.meta.dirname, PACKAGES_DIR],
      });
    } catch {
      // Try next candidate or workspace fallback
      continue;
    }
    const pkgDir = dirname(pkgJsonPath);
    const candidate = join(pkgDir, "bin", binaryName);
    if (existsSync(candidate)) {
      return verifyTrustedBinaryPath(candidate, pkgDir, options);
    }
  }

  for (const dir of entry.dirs) {
    const pkgDir = join(PACKAGES_DIR, dir);
    const candidate = join(pkgDir, "bin", binaryName);
    if (existsSync(candidate)) {
      return verifyTrustedBinaryPath(candidate, pkgDir, options);
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
  const binaryPath = resolveBinaryPath(binaryName);
  const socketPath =
    process.env.DIORAMA_OCI_CRI_SOCKET || DEFAULT_OCI_CRI_SOCKET;

  /** @type {import("node:child_process").ChildProcess | null} */
  let ociCriProc = null;

  if (
    binaryName === "diorama" &&
    !isReadOnlyInvocation(args) &&
    !hasExplicitRuntimeSelection(args, process.env) &&
    !existsSync(socketPath)
  ) {
    try {
      const ociCriBin = resolveBinaryPath("oci-cri");
      mkdirSync(dirname(socketPath), { recursive: true, mode: 0o700 });
      ociCriProc = spawn(ociCriBin, ["serve", "--socket", socketPath], {
        stdio: "ignore",
      });
    } catch {
      // Proceed and let diorama report the CRI readiness diagnostic if needed
    }
  }

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
