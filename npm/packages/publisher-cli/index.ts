#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";

export type ArchName = "amd64" | "arm64" | "aarch64";

export interface ArchTargetSpec {
  readonly arch: ArchName;
  readonly packageName: `@diorama/${ArchName}`;
  readonly packageDir: string;
  readonly distSuffixes: readonly string[];
  readonly rustTargets: readonly string[];
}

const NPM_ROOT = resolve(import.meta.dirname, "../..");
const REPO_ROOT = resolve(NPM_ROOT, "..");

export const REQUIRED_DIORAMA_FEATURE = "diorama/oci-cri";
export const OCI_CRI_ENABLED_MARKER = "oci_cri_readiness_failed";
const OCI_CRI_DISABLED_MARKER =
  "managed oci-cri requires a build with --features oci-cri";

export const ARCH_TARGETS: Readonly<Record<ArchName, ArchTargetSpec>> = {
  amd64: {
    arch: "amd64",
    packageName: "@diorama/amd64",
    packageDir: join(NPM_ROOT, "packages", "amd64"),
    distSuffixes: ["amd64", "x86_64"],
    rustTargets: ["x86_64-unknown-linux-musl", "x86_64-unknown-linux-gnu"],
  },
  arm64: {
    arch: "arm64",
    packageName: "@diorama/arm64",
    packageDir: join(NPM_ROOT, "packages", "arm64"),
    distSuffixes: ["arm64", "aarch64"],
    rustTargets: ["aarch64-unknown-linux-musl", "aarch64-unknown-linux-gnu"],
  },
  aarch64: {
    arch: "aarch64",
    packageName: "@diorama/aarch64",
    packageDir: join(NPM_ROOT, "packages", "aarch64"),
    distSuffixes: ["aarch64", "arm64"],
    rustTargets: ["aarch64-unknown-linux-musl", "aarch64-unknown-linux-gnu"],
  },
};

const BINARIES = ["diorama", "dioramactl"] as const;

export function normalizeVersion(rawVersion: string): string {
  const cleaned = rawVersion.trim().replace(/^v/, "");
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(cleaned)) {
    throw new Error(`Invalid semantic version "${rawVersion}"`);
  }
  return cleaned;
}

export function inferNpmDistTag(version: string): string {
  return version.includes("-") ? "next" : "latest";
}

function resolveUpxBinary(): string | null {
  for (const candidate of ["/opt/homebrew/bin/upx", "/usr/local/bin/upx", "upx"]) {
    const res = spawnSync(candidate, ["--version"], { stdio: "ignore" });
    if (res.status === 0) {
      return candidate;
    }
  }
  return null;
}

/**
 * Checks whether a compiled `diorama` binary includes the `oci-cri` feature.
 */
export function hasOciCriFeature(binaryPath: string): boolean {
  if (!existsSync(binaryPath) || statSync(binaryPath).size === 0) {
    return false;
  }
  let bytes = readFileSync(binaryPath);
  if (bytes.includes(Buffer.from("UPX!", "utf8"))) {
    const upx = resolveUpxBinary();
    if (!upx) {
      return false;
    }
    const tmpDir = mkdtempSync(join(tmpdir(), "diorama-upx-verify-"));
    const unpackedPath = join(tmpDir, basename(binaryPath));
    try {
      const res = spawnSync(upx, ["-q", "-d", `-o${unpackedPath}`, binaryPath], {
        stdio: "ignore",
      });
      if (res.status !== 0 || !existsSync(unpackedPath)) {
        return false;
      }
      bytes = readFileSync(unpackedPath);
    } finally {
      rmSync(tmpDir, { recursive: true, force: true });
    }
  }
  if (bytes.includes(Buffer.from(OCI_CRI_DISABLED_MARKER, "utf8"))) {
    return false;
  }
  return bytes.includes(Buffer.from(OCI_CRI_ENABLED_MARKER, "utf8"));
}

function updateJsonFile(
  filePath: string,
  updater: (data: Record<string, unknown>) => void,
): void {
  const raw = readFileSync(filePath, "utf8");
  const parsed = JSON.parse(raw) as Record<string, unknown>;
  updater(parsed);
  writeFileSync(filePath, `${JSON.stringify(parsed, null, 2)}\n`, "utf8");
}

export function syncWorkspaceVersion(
  rawVersion: string,
  npmRoot: string = NPM_ROOT,
  dryRun: boolean = false,
): string {
  const version = normalizeVersion(rawVersion);

  for (const arch of Object.keys(ARCH_TARGETS) as ArchName[]) {
    const pkgJson = join(npmRoot, "packages", arch, "package.json");
    if (dryRun) {
      console.log(`[dry-run] update ${pkgJson} -> version "${version}"`);
    } else {
      updateJsonFile(pkgJson, (pkg) => {
        pkg.version = version;
      });
    }
  }

  const publisherPkgJson = join(npmRoot, "packages", "publisher-cli", "package.json");
  if (existsSync(publisherPkgJson)) {
    if (dryRun) {
      console.log(`[dry-run] update ${publisherPkgJson} -> version "${version}"`);
    } else {
      updateJsonFile(publisherPkgJson, (pkg) => {
        pkg.version = version;
      });
    }
  }

  const rootPkgJson = join(npmRoot, "package.json");
  if (dryRun) {
    console.log(
      `[dry-run] update ${rootPkgJson} -> version "${version}" and optionalDependencies (@diorama/amd64, @diorama/arm64, @diorama/aarch64 = "${version}")`,
    );
  } else {
    updateJsonFile(rootPkgJson, (pkg) => {
      pkg.version = version;
      const optionalDeps = (pkg.optionalDependencies ?? {}) as Record<
        string,
        string
      >;
      for (const spec of Object.values(ARCH_TARGETS)) {
        optionalDeps[spec.packageName] = version;
      }
      pkg.optionalDependencies = optionalDeps;
    });
  }

  return version;
}

function resolveArches(archFlag?: string): ArchName[] {
  if (!archFlag || archFlag === "all") {
    return ["amd64", "arm64", "aarch64"];
  }
  if (archFlag === "amd64" || archFlag === "arm64" || archFlag === "aarch64") {
    return [archFlag];
  }
  throw new Error(
    `Invalid --arch "${archFlag}". Expected one of: amd64, arm64, aarch64, all`,
  );
}

function runCommand(
  cmd: string,
  args: readonly string[],
  cwd: string,
  env?: NodeJS.ProcessEnv,
): void {
  const res = spawnSync(cmd, args, {
    cwd,
    stdio: "inherit",
    env: env ? { ...process.env, ...env } : process.env,
  });
  if (res.error) {
    throw res.error;
  }
  if (res.status !== 0) {
    throw new Error(
      `Command failed (${res.status ?? 1}): ${cmd} ${args.join(" ")}`,
    );
  }
}

function isEligiblePrebuilt(
  binaryName: (typeof BINARIES)[number],
  candidatePath: string,
): boolean {
  if (!existsSync(candidatePath) || statSync(candidatePath).size === 0) {
    return false;
  }
  if (binaryName === "diorama") {
    return hasOciCriFeature(candidatePath);
  }
  return true;
}

function locatePrebuiltBinary(
  binaryName: (typeof BINARIES)[number],
  spec: ArchTargetSpec,
  fromDir?: string,
): string | null {
  const searchDirs = fromDir
    ? [resolve(fromDir)]
    : [join(REPO_ROOT, "release"), join(REPO_ROOT, "dist")];

  for (const dir of searchDirs) {
    for (const suffix of spec.distSuffixes) {
      const candidate = join(dir, `${binaryName}-linux-${suffix}`);
      if (isEligiblePrebuilt(binaryName, candidate)) {
        return candidate;
      }
    }
    const bareCandidate = join(dir, binaryName);
    if (fromDir && isEligiblePrebuilt(binaryName, bareCandidate)) {
      return bareCandidate;
    }
  }

  for (const target of spec.rustTargets) {
    for (const profile of ["dist", "release"]) {
      const candidate = join(REPO_ROOT, "target", target, profile, binaryName);
      if (isEligiblePrebuilt(binaryName, candidate)) {
        return candidate;
      }
    }
  }

  return null;
}

function resolveZigLibDir(): string | null {
  const res = spawnSync("zig", ["env"], { encoding: "utf8" });
  if (res.status === 0 && res.stdout) {
    const match = res.stdout.match(/lib_dir["\s:=]+"([^"]+)"/);
    if (match?.[1] && existsSync(match[1])) {
      return match[1];
    }
  }
  if (existsSync("/opt/homebrew/lib/zig")) {
    return "/opt/homebrew/lib/zig";
  }
  return null;
}

function createCrossWrappers(wrapperDir: string): {
  goWrapper: string;
  x86MuslCc: string;
  x86MuslCxx: string;
  x86MuslAr: string;
  x86MuslRanlib: string;
  aarch64MuslCc: string;
  aarch64MuslCxx: string;
  aarch64MuslAr: string;
  aarch64MuslRanlib: string;
  aarch64UsesZig: boolean;
} {
  mkdirSync(wrapperDir, { recursive: true });
  const writeScript = (name: string, content: string): string => {
    const p = join(wrapperDir, name);
    writeFileSync(p, content, { mode: 0o755 });
    chmodSync(p, 0o755);
    return p;
  };

  const zigFilter = `
args=""
for arg in "$@"; do
  case "$arg" in
    --target=*|-Wl,--fix-cortex-a53-843419) ;;
    -Wl,/*.a|-Wl,/*.o)
      stripped="\${arg#-Wl,}"
      args="$args \\"$stripped\\""
      ;;
    *) args="$args \\"$arg\\"" ;;
  esac
done
`;

  const x86MuslCc = writeScript(
    "x86_64-linux-musl-cc.sh",
    `#!/bin/sh\n${zigFilter}eval "exec zig cc -target x86_64-linux-musl $args"\n`,
  );
  const x86MuslCxx = writeScript(
    "x86_64-linux-musl-cxx.sh",
    `#!/bin/sh\n${zigFilter}eval "exec zig c++ -target x86_64-linux-musl $args"\n`,
  );
  const x86MuslAr = writeScript(
    "x86_64-linux-musl-ar.sh",
    `#!/bin/sh\nexec zig ar "$@"\n`,
  );
  const x86MuslRanlib = writeScript(
    "x86_64-linux-musl-ranlib.sh",
    `#!/bin/sh\nexec zig ranlib "$@"\n`,
  );

  const hasBrewAarch64 = existsSync(
    "/opt/homebrew/bin/aarch64-unknown-linux-musl-gcc",
  );
  const aarch64MuslCc = hasBrewAarch64
    ? "/opt/homebrew/bin/aarch64-unknown-linux-musl-gcc"
    : writeScript(
        "aarch64-linux-musl-cc.sh",
        `#!/bin/sh\n${zigFilter}eval "exec zig cc -target aarch64-linux-musl $args"\n`,
      );
  const aarch64MuslCxx = hasBrewAarch64
    ? "/opt/homebrew/bin/aarch64-unknown-linux-musl-g++"
    : writeScript(
        "aarch64-linux-musl-cxx.sh",
        `#!/bin/sh\n${zigFilter}eval "exec zig c++ -target aarch64-linux-musl $args"\n`,
      );
  const aarch64MuslAr = hasBrewAarch64
    ? "/opt/homebrew/bin/aarch64-unknown-linux-musl-ar"
    : x86MuslAr;
  const aarch64MuslRanlib = hasBrewAarch64
    ? "/opt/homebrew/bin/aarch64-unknown-linux-musl-ranlib"
    : x86MuslRanlib;

  const goWrapper = writeScript(
    "go-cross-wrapper.sh",
    `#!/bin/sh
if [ "\${GOOS:-}" = "linux" ]; then
  extar=""
  case "\${GOARCH:-}" in
    arm64)
      export CC="${aarch64MuslCc}"
      extar="${aarch64MuslAr}"
      ;;
    amd64)
      export CC="${x86MuslCc}"
      extar="${x86MuslAr}"
      ;;
  esac
  if [ "\${1:-}" = "build" ] && [ -n "\$extar" ]; then
    shift
    exec go build "-ldflags=-s -w -extar=\$extar" "$@"
  fi
fi
exec go "$@"
`,
  );

  return {
    goWrapper,
    x86MuslCc,
    x86MuslCxx,
    x86MuslAr,
    x86MuslRanlib,
    aarch64MuslCc,
    aarch64MuslCxx,
    aarch64MuslAr,
    aarch64MuslRanlib,
    aarch64UsesZig: !hasBrewAarch64,
  };
}

function compileRustTarget(
  rustTarget: string,
  profile: string,
  dryRun: boolean = false,
): void {
  const cargoArgs = [
    "build",
    "--locked",
    "--profile",
    profile,
    "--target",
    rustTarget,
    "-p",
    "diorama",
    "--features",
    REQUIRED_DIORAMA_FEATURE,
    "-p",
    "dioramactl",
  ];
  if (dryRun) {
    console.log(
      `[dry-run] (cwd: ${REPO_ROOT}) cargo ${cargoArgs.join(" ")}`,
    );
    return;
  }

  const cargoHome = process.env.CARGO_HOME ?? join(process.env.HOME ?? "", ".cargo");
  const pathEnv = `${join(cargoHome, "bin")}:/opt/homebrew/bin:${process.env.PATH ?? ""}`;
  const wrappers = createCrossWrappers(join(REPO_ROOT, "target", "cross-wrappers"));
  const env: NodeJS.ProcessEnv = {
    PATH: pathEnv,
    RUSTC_WRAPPER: "",
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? "4",
    GO: wrappers.goWrapper,
    ROK_ALLOW_CROSS: "1",
    ROR_ALLOW_CROSS: "1",
    ROK_DISABLE_OPTIONAL_LIBS: "1",
    ROR_DISABLE_OPTIONAL_LIBS: "1",
  };

  const zigLibDir = resolveZigLibDir();
  const muslInitSrc = join(REPO_ROOT, "target", "cross-wrappers", "musl_init_args.c");
  writeFileSync(
    muslInitSrc,
    [
      "#include <stddef.h>",
      "#include <stdint.h>",
      "extern void _init(void);",
      "extern char **__environ;",
      "extern char *__progname_full;",
      'extern __attribute__((weak, visibility("hidden"))) void (*const __init_array_start)(void);',
      'extern __attribute__((weak, visibility("hidden"))) void (*const __init_array_end)(void);',
      "void __libc_start_init(void) {",
      "  int argc = 0;",
      "  char **argv = NULL;",
      "  char **envp = __environ;",
      "  if (envp != NULL && envp[-1] == NULL) {",
      "    char **end = envp - 1;",
      "    for (char **cur = end - 1; (end - cur) <= 131072; --cur) {",
      "      size_t candidate = (size_t)(end - cur);",
      "      if (*(size_t *)(cur - 1) == candidate && (cur[0] == __progname_full || candidate == 1)) {",
      "        argc = (int)candidate;",
      "        argv = cur;",
      "        break;",
      "      }",
      "    }",
      "  }",
      "  _init();",
      "  uintptr_t a = (uintptr_t)&__init_array_start;",
      "  for (; a < (uintptr_t)&__init_array_end; a += sizeof(void (*)(void))) {",
      "    (*(void (**)(int, char **, char **))a)(argc, argv, envp);",
      "  }",
      "}",
      "",
    ].join("\n"),
    "utf8",
  );

  if (rustTarget === "aarch64-unknown-linux-musl") {
    const initObj = join(REPO_ROOT, "target", "cross-wrappers", "musl_init_args_aarch64.o");
    runCommand(wrappers.aarch64MuslCc, ["-O2", "-c", muslInitSrc, "-o", initObj], REPO_ROOT, env);
    env.CC_aarch64_unknown_linux_musl = wrappers.aarch64MuslCc;
    env.CXX_aarch64_unknown_linux_musl = wrappers.aarch64MuslCxx;
    env.AR_aarch64_unknown_linux_musl = wrappers.aarch64MuslAr;
    env.RANLIB_aarch64_unknown_linux_musl = wrappers.aarch64MuslRanlib;
    env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER = wrappers.aarch64MuslCc;

    if (wrappers.aarch64UsesZig) {
      env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
        `-C link-self-contained=no -C link-arg=${initObj}`;
      if (zigLibDir) {
        env.BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_musl = [
          `-isystem ${zigLibDir}/include`,
          `-isystem ${zigLibDir}/libc/include/aarch64-linux-musl`,
          `-isystem ${zigLibDir}/libc/include/generic-musl`,
          `-isystem ${zigLibDir}/libc/include/aarch64-linux-any`,
          `-isystem ${zigLibDir}/libc/include/any-linux-any`,
        ].join(" ");
      }
    } else {
      env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
        `-C link-arg=${initObj}`;
      const sysrootRes = spawnSync(wrappers.aarch64MuslCc, ["-print-sysroot"], {
        encoding: "utf8",
      });
      const sysroot = sysrootRes.stdout?.trim();
      if (sysroot) {
        env.BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_musl = `--sysroot=${sysroot}`;
      }
    }

    const toolchainFile = join(
      REPO_ROOT,
      "target",
      "aarch64-unknown-linux-musl-toolchain.cmake",
    );
    writeFileSync(
      toolchainFile,
      [
        "set(CMAKE_SYSTEM_NAME Linux)",
        "set(CMAKE_SYSTEM_PROCESSOR aarch64)",
        `set(CMAKE_C_COMPILER "${wrappers.aarch64MuslCc}")`,
        `set(CMAKE_CXX_COMPILER "${wrappers.aarch64MuslCxx}")`,
        `set(CMAKE_ASM_COMPILER "${wrappers.aarch64MuslCc}")`,
        `set(CMAKE_AR "${wrappers.aarch64MuslAr}" CACHE FILEPATH "Archiver" FORCE)`,
        `set(CMAKE_RANLIB "${wrappers.aarch64MuslRanlib}" CACHE FILEPATH "Ranlib" FORCE)`,
        "",
      ].join("\n"),
      "utf8",
    );
    env.CMAKE_TOOLCHAIN_FILE_aarch64_unknown_linux_musl = toolchainFile;
  } else if (rustTarget === "x86_64-unknown-linux-musl") {
    const initObj = join(REPO_ROOT, "target", "cross-wrappers", "musl_init_args_x86_64.o");
    runCommand(wrappers.x86MuslCc, ["-O2", "-c", muslInitSrc, "-o", initObj], REPO_ROOT, env);
    env.CC_x86_64_unknown_linux_musl = wrappers.x86MuslCc;
    env.CXX_x86_64_unknown_linux_musl = wrappers.x86MuslCxx;
    env.AR_x86_64_unknown_linux_musl = wrappers.x86MuslAr;
    env.RANLIB_x86_64_unknown_linux_musl = wrappers.x86MuslRanlib;
    env.CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = wrappers.x86MuslCc;
    env.CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
      `-C link-self-contained=no -C link-arg=${initObj}`;

    if (zigLibDir) {
      env.BINDGEN_EXTRA_CLANG_ARGS_x86_64_unknown_linux_musl = [
        `-isystem ${zigLibDir}/include`,
        `-isystem ${zigLibDir}/libc/include/x86_64-linux-musl`,
        `-isystem ${zigLibDir}/libc/include/generic-musl`,
        `-isystem ${zigLibDir}/libc/include/x86-linux-any`,
        `-isystem ${zigLibDir}/libc/include/any-linux-any`,
      ].join(" ");
    }

    const toolchainFile = join(
      REPO_ROOT,
      "target",
      "x86_64-unknown-linux-musl-toolchain.cmake",
    );
    writeFileSync(
      toolchainFile,
      [
        "set(CMAKE_SYSTEM_NAME Linux)",
        "set(CMAKE_SYSTEM_PROCESSOR x86_64)",
        `set(CMAKE_C_COMPILER "${wrappers.x86MuslCc}")`,
        `set(CMAKE_CXX_COMPILER "${wrappers.x86MuslCxx}")`,
        `set(CMAKE_ASM_COMPILER "${wrappers.x86MuslCc}")`,
        `set(CMAKE_AR "${wrappers.x86MuslAr}" CACHE FILEPATH "Archiver" FORCE)`,
        `set(CMAKE_RANLIB "${wrappers.x86MuslRanlib}" CACHE FILEPATH "Ranlib" FORCE)`,
        "",
      ].join("\n"),
      "utf8",
    );
    env.CMAKE_TOOLCHAIN_FILE_x86_64_unknown_linux_musl = toolchainFile;
  }

  runCommand("cargo", cargoArgs, REPO_ROOT, env);
}

export interface StageOptions {
  readonly arches: readonly ArchName[];
  readonly fromDir?: string;
  readonly compile: boolean;
  readonly profile: string;
  readonly version?: string;
  readonly npmRoot?: string;
  readonly dryRun?: boolean;
}

export function stageBinaries(options: StageOptions): void {
  const npmRoot = options.npmRoot ?? NPM_ROOT;
  const dryRun = options.dryRun ?? false;
  if (options.version) {
    const synced = syncWorkspaceVersion(options.version, npmRoot, dryRun);
    if (!dryRun) {
      console.log(`Synced workspace version to ${synced}`);
    }
  }

  const compiledTargets = new Set<string>();
  const compressedSources = new Map<string, string>();
  const upxBin = resolveUpxBinary();

  for (const arch of options.arches) {
    const spec = ARCH_TARGETS[arch];
    const targetBinDir = join(npmRoot, "packages", arch, "bin");
    if (!dryRun) {
      mkdirSync(targetBinDir, { recursive: true });
    }

    for (const binaryName of BINARIES) {
      let sourcePath = options.compile
        ? null
        : locatePrebuiltBinary(binaryName, spec, options.fromDir);

      if (!sourcePath) {
        const rustTarget = spec.rustTargets[0]!;
        if (!compiledTargets.has(rustTarget)) {
          if (!dryRun) {
            console.log(
              `Compiling ${rustTarget} (profile: ${options.profile}, features: ${REQUIRED_DIORAMA_FEATURE})...`,
            );
          }
          compileRustTarget(rustTarget, options.profile, dryRun);
          compiledTargets.add(rustTarget);
        }
        const profileDir = options.profile === "dev" ? "debug" : options.profile;
        const builtPath = join(
          REPO_ROOT,
          "target",
          rustTarget,
          profileDir,
          binaryName,
        );
        if (!dryRun && !existsSync(builtPath)) {
          throw new Error(`Expected compiled binary at ${builtPath}`);
        }
        sourcePath = builtPath;
      }

      const destPath = join(targetBinDir, binaryName);
      if (dryRun) {
        const sizeInfo = existsSync(sourcePath)
          ? ` (${(statSync(sourcePath).size / (1024 * 1024)).toFixed(2)} MB, oci-cri enabled)`
          : ` (will be compiled with --features ${REQUIRED_DIORAMA_FEATURE})`;
        console.log(
          `[dry-run] stage ${spec.packageName}/bin/${binaryName} <- ${sourcePath}${sizeInfo} (chmod 0755 -> ${destPath})`,
        );
      } else {
        if (binaryName === "diorama" && !hasOciCriFeature(sourcePath)) {
          throw new Error(
            `Binary ${sourcePath} was not built with --features ${REQUIRED_DIORAMA_FEATURE}`,
          );
        }
        const rawBytes = statSync(sourcePath).size;
        const alreadyCompressed = compressedSources.get(sourcePath);
        if (alreadyCompressed && existsSync(alreadyCompressed)) {
          copyFileSync(alreadyCompressed, destPath);
        } else {
          copyFileSync(sourcePath, destPath);
          chmodSync(destPath, 0o755);
          if (upxBin && rawBytes > 1024 * 1024) {
            const head = readFileSync(destPath);
            if (!head.includes(Buffer.from("UPX!", "utf8"))) {
              runCommand(upxBin, ["-q", "--best", "--lzma", destPath], npmRoot);
            }
          }
          compressedSources.set(sourcePath, destPath);
        }
        chmodSync(destPath, 0o755);
        const sizeMb = (statSync(destPath).size / (1024 * 1024)).toFixed(2);
        const rawMb = (rawBytes / (1024 * 1024)).toFixed(2);
        const sizeLabel =
          sizeMb !== rawMb ? `${sizeMb} MB (packed from ${rawMb} MB)` : `${sizeMb} MB`;
        console.log(
          `Staged ${spec.packageName}/bin/${binaryName} <- ${sourcePath} (${sizeLabel})`,
        );
      }
    }
  }
}

export function verifyPackageStaged(
  arch: ArchName,
  npmRoot: string = NPM_ROOT,
): void {
  const pkgDir = join(npmRoot, "packages", arch);
  for (const binaryName of BINARIES) {
    const binPath = join(pkgDir, "bin", binaryName);
    if (!existsSync(binPath) || statSync(binPath).size === 0) {
      throw new Error(
        `Missing staged binary ${binPath} for @diorama/${arch}. Run "bun run packages/publisher-cli/index.ts build" first.`,
      );
    }
    if (binaryName === "diorama" && !hasOciCriFeature(binPath)) {
      throw new Error(
        `Staged binary ${binPath} for @diorama/${arch} is missing --features ${REQUIRED_DIORAMA_FEATURE}.`,
      );
    }
  }
}

function packPackages(
  arches: readonly ArchName[],
  includeRoot: boolean,
  dryRun: boolean = false,
): void {
  for (const arch of arches) {
    const spec = ARCH_TARGETS[arch];
    const pkgDir = spec.packageDir;
    if (dryRun) {
      for (const binaryName of BINARIES) {
        const binPath = join(pkgDir, "bin", binaryName);
        const status = isEligiblePrebuilt(binaryName, binPath)
          ? `present (${(statSync(binPath).size / (1024 * 1024)).toFixed(2)} MB, oci-cri verified)`
          : "not yet staged (requires --features diorama/oci-cri build)";
        console.log(`[dry-run] verify ${spec.packageName}/bin/${binaryName}: ${status}`);
      }
      console.log(`[dry-run] (cwd: ${pkgDir}) bun pm pack`);
    } else {
      verifyPackageStaged(arch);
      console.log(`Packing ${spec.packageName}...`);
      runCommand("bun", ["pm", "pack"], pkgDir);
    }
  }
  if (includeRoot) {
    if (dryRun) {
      console.log(`[dry-run] (cwd: ${NPM_ROOT}) bun pm pack`);
    } else {
      console.log("Packing @diorama/cli...");
      runCommand("bun", ["pm", "pack"], NPM_ROOT);
    }
  }
}

interface PublishOptions {
  readonly arches: readonly ArchName[];
  readonly includeRoot: boolean;
  readonly dryRun: boolean;
  readonly tag?: string;
  readonly otp?: string;
  readonly effectiveVersion?: string;
}

async function runPublishCommand(
  cmd: string,
  args: readonly string[],
  cwd: string,
): Promise<void> {
  const openedUrls = new Set<string>();
  const proc = Bun.spawn([cmd, ...args], {
    cwd,
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
    env: {
      ...process.env,
      FORCE_COLOR: "1",
    },
  });

  const handleChunk = (chunk: Uint8Array, isStderr: boolean): void => {
    if (isStderr) {
      process.stderr.write(chunk);
    } else {
      process.stdout.write(chunk);
    }
    const text = new TextDecoder().decode(chunk).replace(/\x1b\[[0-9;]*m/g, "");
    const urlMatches = text.match(/https:\/\/www\.npmjs\.com\/[^\s)\]]+/g);
    if (urlMatches) {
      for (const rawUrl of urlMatches) {
        const cleanUrl = rawUrl.trim();
        if (!openedUrls.has(cleanUrl)) {
          openedUrls.add(cleanUrl);
          console.log(`\n-> Opening npm authentication URL in browser: ${cleanUrl}`);
          const opener = process.platform === "darwin" ? "/usr/bin/open" : "xdg-open";
          spawnSync(opener, [cleanUrl], { stdio: "ignore" });
          try {
            proc.stdin.write("\n");
            proc.stdin.flush();
          } catch {
            // Ignore if stdin closed
          }
        }
      }
    }
  };

  const pumpStream = async (
    stream: ReadableStream<Uint8Array> | null,
    isStderr: boolean,
  ): Promise<void> => {
    if (!stream) return;
    const reader = stream.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      if (value) {
        handleChunk(value, isStderr);
      }
    }
  };

  await Promise.all([
    pumpStream(proc.stdout, false),
    pumpStream(proc.stderr, true),
  ]);

  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`Command failed (${exitCode}): ${cmd} ${args.join(" ")}`);
  }
}

function isPackageVersionPublished(pkgName: string, version: string): boolean {
  const res = spawnSync("npm", ["view", `${pkgName}@${version}`, "version"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  });
  return res.status === 0 && res.stdout.trim() === version;
}

async function publishSinglePackage(
  pkgName: string,
  pkgVersion: string,
  pkgDir: string,
  distTag: string,
  otp?: string,
): Promise<void> {
  if (isPackageVersionPublished(pkgName, pkgVersion)) {
    console.log(
      `Skipping ${pkgName}@${pkgVersion}: already published on registry.npmjs.org.`,
    );
    return;
  }

  const useTrustedOidc =
    process.env.GITHUB_ACTIONS === "true" &&
    Boolean(process.env.ACTIONS_ID_TOKEN_REQUEST_URL);

  if (useTrustedOidc) {
    const npmArgs = [
      "publish",
      "--access",
      "public",
      "--tag",
      distTag,
      "--provenance",
    ];
    console.log(
      `Publishing ${pkgName}@${pkgVersion} via npm Trusted Publishing (OIDC, tag: ${distTag})...`,
    );
    runCommand("npm", npmArgs, pkgDir);
    return;
  }

  const bunArgs = [
    "publish",
    "--access",
    "public",
    "--tag",
    distTag,
    "--tolerate-republish",
  ];
  if (otp) {
    bunArgs.push("--otp", otp);
  }
  console.log(`Publishing ${pkgName}@${pkgVersion} (tag: ${distTag})...`);
  await runPublishCommand("bun", bunArgs, pkgDir);
}

async function publishPackages(options: PublishOptions): Promise<void> {
  const rootPkg = JSON.parse(
    readFileSync(join(NPM_ROOT, "package.json"), "utf8"),
  ) as { version: string };
  const targetVersion = options.effectiveVersion ?? rootPkg.version;
  const distTag = options.tag ?? inferNpmDistTag(targetVersion);

  for (const arch of options.arches) {
    const spec = ARCH_TARGETS[arch];
    const pkgDir = spec.packageDir;
    const pkgJson = JSON.parse(
      readFileSync(join(pkgDir, "package.json"), "utf8"),
    ) as { name: string; version: string; files?: string[] };
    const pkgVersion = options.effectiveVersion ?? pkgJson.version;

    if (pkgVersion !== targetVersion) {
      throw new Error(
        `Version mismatch: ${pkgJson.name} is ${pkgVersion}, but @diorama/cli is ${targetVersion}`,
      );
    }

    if (options.dryRun) {
      const args = [
        "publish",
        "--access",
        "public",
        "--tag",
        distTag,
        "--tolerate-republish",
        "--dry-run",
      ];
      if (options.otp) {
        args.push("--otp", options.otp);
      }
      const stagedStatuses = BINARIES.map((bin) => {
        const binPath = join(pkgDir, "bin", bin);
        if (isEligiblePrebuilt(bin, binPath)) {
          return `bin/${bin} (${(statSync(binPath).size / (1024 * 1024)).toFixed(2)} MB staged, oci-cri verified)`;
        }
        const prebuilt = locatePrebuiltBinary(bin, spec);
        if (prebuilt) {
          return `bin/${bin} (ready to stage from ${prebuilt}, ${(statSync(prebuilt).size / (1024 * 1024)).toFixed(2)} MB)`;
        }
        return `bin/${bin} (requires cargo build --features ${REQUIRED_DIORAMA_FEATURE})`;
      });
      console.log(
        `[dry-run] publish ${pkgJson.name}@${pkgVersion} -> (cwd: ${pkgDir}) bun ${args.join(" ")}`,
      );
      console.log(`          payloads: ${stagedStatuses.join(", ")}`);
    } else {
      verifyPackageStaged(arch);
      await publishSinglePackage(
        pkgJson.name,
        pkgVersion,
        pkgDir,
        distTag,
        options.otp,
      );
    }
  }

  if (options.includeRoot) {
    if (options.dryRun) {
      const args = [
        "publish",
        "--access",
        "public",
        "--tag",
        distTag,
        "--tolerate-republish",
        "--dry-run",
      ];
      console.log(
        `[dry-run] publish @diorama/cli@${targetVersion} -> (cwd: ${NPM_ROOT}) bun ${args.join(" ")}`,
      );
      console.log(
        `          optionalDependencies: @diorama/amd64@${targetVersion}, @diorama/arm64@${targetVersion}, @diorama/aarch64@${targetVersion}`,
      );
    } else {
      await publishSinglePackage(
        "@diorama/cli",
        targetVersion,
        NPM_ROOT,
        distTag,
        options.otp,
      );
    }
  }
}

function parseFlag(args: readonly string[], name: string): string | undefined {
  for (let i = 0; i < args.length; i++) {
    const arg = args[i]!;
    if (arg === name) {
      return args[i + 1];
    }
    if (arg.startsWith(`${name}=`)) {
      return arg.slice(name.length + 1);
    }
  }
  return undefined;
}

function printHelp(): void {
  console.log(`Usage: bun run packages/publisher-cli/index.ts <command> [options]

Commands:
  version <semver>   Sync package.json versions across @diorama/cli and @diorama/{amd64,arm64,aarch64}
  build              Build (with --features diorama/oci-cri) or stage binaries into packages/<arch>/bin
  stage              Alias for build (stages prebuilt oci-cri release binaries when present)
  pack               Verify staged oci-cri binaries and create .tgz tarballs via "bun pm pack"
  publish            Publish @diorama/{amd64,arm64,aarch64} and @diorama/cli in dependency order

Options:
  --arch <name>      Target architecture: amd64 | arm64 | aarch64 | all (default: all)
  --from-dir <dir>   Directory containing diorama-linux-<arch> & dioramactl-linux-<arch>
  --compile          Force compiling binaries with cargo even if prebuilt artifacts exist
  --profile <name>   Cargo profile when compiling (default: dist)
  --version <ver>    Sync workspace version before building/packing/publishing
  --tag <dist-tag>   npm dist-tag (default: "latest" for stable, "next" for prereleases)
  --otp <code>       One-time password for npm 2FA
  --dry-run          Print planned version edits, binary staging, and pack/publish commands without mutating or uploading
  --skip-root        Only pack/publish platform package(s), skipping root @diorama/cli
  --only-root        Only pack/publish root @diorama/cli
`);
}

export async function main(
  argv: readonly string[] = process.argv.slice(2),
): Promise<void> {
  const command = argv[0];
  if (!command || command === "--help" || command === "-h" || command === "help") {
    printHelp();
    return;
  }

  const rest = argv.slice(1);
  const archFlag = parseFlag(rest, "--arch");
  const versionFlag = parseFlag(rest, "--version");
  const fromDirFlag = parseFlag(rest, "--from-dir");
  const profileFlag = parseFlag(rest, "--profile") ?? "dist";
  const tagFlag = parseFlag(rest, "--tag");
  const otpFlag = parseFlag(rest, "--otp");
  const compile = rest.includes("--compile");
  const dryRun = rest.includes("--dry-run");
  const skipRoot = rest.includes("--skip-root");
  const onlyRoot = rest.includes("--only-root");

  switch (command) {
    case "version": {
      const positionalVersion = rest.find((arg) => !arg.startsWith("-"));
      const targetVersion = versionFlag ?? positionalVersion;
      if (!targetVersion) {
        throw new Error("Usage: bun run packages/publisher-cli/index.ts version <semver>");
      }
      const synced = syncWorkspaceVersion(targetVersion, NPM_ROOT, dryRun);
      if (!dryRun) {
        console.log(`Updated @diorama/cli and platform packages to version ${synced}`);
      }
      break;
    }
    case "build":
    case "stage": {
      const arches = resolveArches(archFlag);
      stageBinaries({
        arches,
        fromDir: fromDirFlag,
        compile,
        profile: profileFlag,
        version: versionFlag,
        dryRun,
      });
      break;
    }
    case "pack": {
      if (versionFlag) {
        syncWorkspaceVersion(versionFlag, NPM_ROOT, dryRun);
      }
      const arches = onlyRoot ? [] : resolveArches(archFlag);
      packPackages(arches, !skipRoot, dryRun);
      break;
    }
    case "publish": {
      const effectiveVersion = versionFlag
        ? syncWorkspaceVersion(versionFlag, NPM_ROOT, dryRun)
        : undefined;
      const arches = onlyRoot ? [] : resolveArches(archFlag);
      await publishPackages({
        arches,
        includeRoot: !skipRoot,
        dryRun,
        tag: tagFlag,
        otp: otpFlag,
        effectiveVersion,
      });
      break;
    }
    default:
      throw new Error(`Unknown command "${command}". Run with --help for usage.`);
  }
}

if (import.meta.main) {
  main().catch((err: unknown) => {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`error: ${message}`);
    process.exit(1);
  });
}
