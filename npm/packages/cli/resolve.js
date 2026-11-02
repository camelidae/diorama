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
  "-V",
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

function hasRuntimeInConfig(content) {
  if (typeof content !== "string" || !content.trim()) {
    return false;
  }
  try {
    const parsed = JSON.parse(content);
    if (parsed && typeof parsed === "object") {
      const rt = parsed.runtime;
      if (typeof rt === "string" && rt.trim().length > 0) {
        return true;
      }
      if (rt && typeof rt === "object") {
        for (const key of ["endpoint", "backend", "runtime"]) {
          if (typeof rt[key] === "string" && rt[key].trim().length > 0) {
            return true;
          }
        }
      }
      return false;
    }
  } catch {
    // Fall back to YAML parsing
  }

  const lines = content.split(/\r?\n/);
  let inRuntime = false;
  let runtimeIndent = 0;

  for (const rawLine of lines) {
    const commentIdx = rawLine.indexOf("#");
    const line = commentIdx >= 0 ? rawLine.slice(0, commentIdx) : rawLine;
    if (!line.trim()) continue;

    const runtimeHeader = line.match(/^(\s*)runtime\s*:(.*)$/);
    if (runtimeHeader) {
      runtimeIndent = runtimeHeader[1].length;
      const inline = runtimeHeader[2].trim();
      if (inline) {
        if (inline.startsWith("{") && inline.endsWith("}")) {
          const body = inline.slice(1, -1);
          const m = body.match(
            /(?:['"]?(?:endpoint|backend|runtime)['"]?)\s*:\s*([^,}\s]+|"[^"]*"|'[^']*')/,
          );
          if (m) {
            const v = m[1].replace(/^['"]|['"]$/g, "").trim();
            if (v && v !== "null" && v !== "~" && v !== '""' && v !== "''") {
              return true;
            }
          }
        } else {
          const val = inline.replace(/^['"]|['"]$/g, "").trim();
          if (val && val !== "null" && val !== "~") return true;
        }
      } else {
        inRuntime = true;
      }
      continue;
    }

    if (inRuntime) {
      const indentMatch = line.match(/^(\s*)(.*)$/);
      const indent = indentMatch ? indentMatch[1].length : 0;
      if (indent <= runtimeIndent) {
        inRuntime = false;
      } else {
        const fieldMatch = line
          .trim()
          .match(/^(?:['"]?(?:endpoint|backend|runtime)['"]?)\s*:(.*)$/);
        if (fieldMatch) {
          const val = fieldMatch[1].trim().replace(/^['"]|['"]$/g, "").trim();
          if (val && val !== "null" && val !== "~") {
            return true;
          }
        }
      }
    }
  }

  return false;
}

function resolveConfigPath(args = [], env = process.env) {
  if (args) {
    for (let i = 0; i < args.length; i++) {
      const arg = args[i];
      if (arg === "--") {
        break;
      }
      if (arg.startsWith("--config=")) {
        return arg.slice("--config=".length);
      }
      if (arg === "--config" && i + 1 < args.length) {
        return args[i + 1];
      }
      if (arg.startsWith("-c=")) {
        return arg.slice("-c=".length);
      }
      if (arg === "-c" && i + 1 < args.length) {
        return args[i + 1];
      }
    }
  }
  if (
    env &&
    typeof env.DIORAMA_CONFIG === "string" &&
    env.DIORAMA_CONFIG.trim().length > 0
  ) {
    return env.DIORAMA_CONFIG;
  }
  return null;
}

function configHasRuntimeSelection(args = [], env = process.env) {
  const configPath = resolveConfigPath(args, env);
  if (!configPath) return false;
  try {
    const fs =
      typeof process.getBuiltinModule === "function"
        ? process.getBuiltinModule("node:fs")
        : require("node:fs");
    if (fs && fs.existsSync(configPath)) {
      const content = fs.readFileSync(configPath, "utf8");
      return hasRuntimeInConfig(content);
    }
  } catch {
    // Ignore read or parse errors
  }
  return false;
}

/**
 * Returns true if the CLI invocation only inspects version, help, or effective config
 * and must not start any background daemon or mutate host state.
 *
 * @param {readonly string[]} [args]
 * @returns {boolean}
 */
export function isReadOnlyInvocation(args = []) {
  if (!args) return false;
  for (const arg of args) {
    if (arg === "--") {
      break;
    }
    if (
      READ_ONLY_FLAGS.has(arg) ||
      arg === "--print-config" ||
      arg.startsWith("--print-config=")
    ) {
      return true;
    }
  }
  return false;
}

/**
 * Returns true if the caller explicitly selected a container runtime or endpoint
 * via CLI flags, environment variables, or config file.
 *
 * @param {readonly string[]} [args]
 * @param {NodeJS.ProcessEnv} [env]
 * @returns {boolean}
 */
export function hasExplicitRuntimeSelection(args = [], env = process.env) {
  if (args) {
    for (const arg of args) {
      if (arg === "--") {
        break;
      }
      if (
        arg === "--container-runtime" ||
        arg.startsWith("--container-runtime=") ||
        arg === "--container-runtime-endpoint" ||
        arg.startsWith("--container-runtime-endpoint=")
      ) {
        return true;
      }
    }
  }
  if (
    env &&
    (env.DIORAMA_CONTAINER_RUNTIME || env.DIORAMA_CONTAINER_RUNTIME_ENDPOINT)
  ) {
    return true;
  }
  return configHasRuntimeSelection(args, env);
}

/**
 * Builds the environment for spawning a Diorama binary, defaulting
 * `DIORAMA_CONTAINER_RUNTIME_ENDPOINT` to `"unix:///run/oci-cri/oci-cri.sock"`
 * when launching `diorama` unless a read-only flag is passed or an explicit
 * runtime or endpoint is set via CLI flags, environment variables, or config file.
 *
 * @param {"diorama" | "dioramactl" | "oci-cri"} binaryName
 * @param {NodeJS.ProcessEnv} [baseEnv]
 * @param {string} [socketPath]
 * @param {readonly string[]} [args]
 * @returns {NodeJS.ProcessEnv}
 */
export function buildRuntimeEnv(
  binaryName,
  baseEnv = process.env,
  socketPath = DEFAULT_OCI_CRI_SOCKET,
  args = typeof process !== "undefined" && process.argv
    ? process.argv.slice(2)
    : [],
) {
  if (
    binaryName === "diorama" &&
    !isReadOnlyInvocation(args) &&
    !hasExplicitRuntimeSelection(args, baseEnv)
  ) {
    return {
      ...baseEnv,
      DIORAMA_CONTAINER_RUNTIME_ENDPOINT: `unix://${socketPath}`,
    };
  }
  return baseEnv;
}

/**
 * Returns true if the CLI should automatically spawn the bundled `oci-cri` daemon.
 *
 * @param {string} binaryName
 * @param {readonly string[]} [args]
 * @param {NodeJS.ProcessEnv} [env]
 * @param {boolean} [socketExists]
 * @returns {boolean}
 */
export function shouldSpawnOciCri(
  binaryName,
  args = [],
  env = process.env,
  socketExists = false,
) {
  if (binaryName !== "diorama") {
    return false;
  }
  if (isReadOnlyInvocation(args)) {
    return false;
  }
  if (hasExplicitRuntimeSelection(args, env)) {
    return false;
  }
  return !socketExists;
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
