import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";

const require = createRequire(import.meta.url);
const PACKAGES_DIR = resolve(import.meta.dirname, "..");

export const DEFAULT_CONTAINER_RUNTIME = "oci-cri";

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
  const entry = PLATFORM_PACKAGES[/** @type {keyof typeof PLATFORM_PACKAGES} */ (key)];
  if (!entry) {
    throw new Error(
      `Unsupported platform/architecture "${key}". @diorama/cli supports linux-x64 (@diorama/amd64) and linux-arm64 (@diorama/arm64, @diorama/aarch64).`,
    );
  }
  return entry.packages;
}

/**
 * Resolves the absolute path to a native binary (`diorama` or `dioramactl`)
 * from the installed platform-specific optional dependency or local workspace.
 *
 * @param {"diorama" | "dioramactl"} binaryName
 * @param {string} [platform]
 * @param {string} [arch]
 * @returns {string}
 */
export function resolveBinaryPath(
  binaryName,
  platform = process.platform,
  arch = process.arch,
) {
  if (binaryName !== "diorama" && binaryName !== "dioramactl") {
    throw new Error(`Unknown binary "${binaryName}"`);
  }

  const key = `${platform}-${arch}`;
  const entry = PLATFORM_PACKAGES[/** @type {keyof typeof PLATFORM_PACKAGES} */ (key)];
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
 * Builds the environment for spawning a Diorama binary, defaulting
 * `DIORAMA_CONTAINER_RUNTIME` to `"oci-cri"` when launching `diorama`.
 *
 * @param {"diorama" | "dioramactl"} binaryName
 * @param {NodeJS.ProcessEnv} [baseEnv]
 * @returns {NodeJS.ProcessEnv}
 */
export function buildRuntimeEnv(binaryName, baseEnv = process.env) {
  if (binaryName === "diorama" && !baseEnv.DIORAMA_CONTAINER_RUNTIME) {
    return {
      ...baseEnv,
      DIORAMA_CONTAINER_RUNTIME: DEFAULT_CONTAINER_RUNTIME,
    };
  }
  return baseEnv;
}

/**
 * Spawns the resolved native binary with inherited stdio and exits with its status code.
 *
 * @param {"diorama" | "dioramactl"} binaryName
 * @param {readonly string[]} [args]
 * @returns {never}
 */
export function runBinary(binaryName, args = process.argv.slice(2)) {
  const binaryPath = resolveBinaryPath(binaryName);
  const result = spawnSync(binaryPath, args, {
    stdio: "inherit",
    env: buildRuntimeEnv(binaryName),
  });

  if (result.error) {
    throw result.error;
  }

  if (result.signal) {
    process.kill(process.pid, result.signal);
  }

  process.exit(result.status ?? 1);
}
