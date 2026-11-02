#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { isAbsolute, join, normalize, resolve } from "node:path";

export type ArchName = "amd64" | "arm64" | "aarch64";

export interface ArchTargetSpec {
  readonly arch: ArchName;
  readonly packageName: `@diorama/${ArchName}`;
  readonly packageDir: string;
  readonly distSuffixes: readonly string[];
  readonly rustTargets: readonly string[];
  readonly ociCriArch: "amd64" | "arm64";
}

const NPM_ROOT = resolve(import.meta.dirname, "../..");
const REPO_ROOT = resolve(NPM_ROOT, "..");

export const OCI_LIB_VERSION = "v0.1.14";
export const MAX_OCI_CRI_ARCHIVE_BYTES = 64 * 1024 * 1024;
export const MAX_OCI_CRI_BINARY_BYTES = 128 * 1024 * 1024;
export const ALLOWED_OCI_LIB_RELEASE_PREFIX =
  "https://github.com/geoffsee/oci-lib/releases/download/";

export interface OciCriReleaseSpec {
  readonly url: string;
  readonly sha256: string;
}

export function validateOciCriReleaseSpec(
  arch: string,
  spec: { readonly url: string; readonly sha256: string },
): void {
  if (!spec || typeof spec !== "object") {
    throw new Error(`Release spec for architecture "${arch}" must be an object`);
  }
  const expectedPrefix = `${ALLOWED_OCI_LIB_RELEASE_PREFIX}${OCI_LIB_VERSION}/`;
  if (!spec.url || typeof spec.url !== "string" || !spec.url.startsWith(expectedPrefix)) {
    throw new Error(
      `Invalid release URL for architecture "${arch}": expected URL to start with "${expectedPrefix}", got "${spec?.url}"`,
    );
  }
  if (!spec.sha256 || typeof spec.sha256 !== "string" || !/^[0-9a-f]{64}$/.test(spec.sha256)) {
    throw new Error(
      `Invalid SHA-256 digest for architecture "${arch}": expected 64-character lowercase hex digest, got "${spec?.sha256}"`,
    );
  }
}

export const OCI_CRI_RELEASES: Readonly<
  Record<"amd64" | "arm64", OciCriReleaseSpec>
> = {
  amd64: {
    url: `https://github.com/geoffsee/oci-lib/releases/download/${OCI_LIB_VERSION}/oci-cri-${OCI_LIB_VERSION}-x86_64-unknown-linux-gnu.tar.gz`,
    sha256: "0e2b24dcfb36caf413fc1a390c32652f580d99a45fe6239959de887f506601d9",
  },
  arm64: {
    url: `https://github.com/geoffsee/oci-lib/releases/download/${OCI_LIB_VERSION}/oci-cri-${OCI_LIB_VERSION}-aarch64-unknown-linux-gnu.tar.gz`,
    sha256: "391e867c06fd9b2b1c344e9bd9f11c5758499786e58f12755dc0f570460a75b6",
  },
};

export const ARCH_TARGETS: Readonly<Record<ArchName, ArchTargetSpec>> = {
  amd64: {
    arch: "amd64",
    packageName: "@diorama/amd64",
    packageDir: join(NPM_ROOT, "packages", "amd64"),
    distSuffixes: ["amd64", "x86_64"],
    rustTargets: ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"],
    ociCriArch: "amd64",
  },
  arm64: {
    arch: "arm64",
    packageName: "@diorama/arm64",
    packageDir: join(NPM_ROOT, "packages", "arm64"),
    distSuffixes: ["arm64", "aarch64"],
    rustTargets: ["aarch64-unknown-linux-gnu", "aarch64-unknown-linux-musl"],
    ociCriArch: "arm64",
  },
  aarch64: {
    arch: "aarch64",
    packageName: "@diorama/aarch64",
    packageDir: join(NPM_ROOT, "packages", "aarch64"),
    distSuffixes: ["aarch64", "arm64"],
    rustTargets: ["aarch64-unknown-linux-gnu", "aarch64-unknown-linux-musl"],
    ociCriArch: "arm64",
  },
};

export const BINARIES = ["diorama", "dioramactl", "oci-cri"] as const;
export type BinaryName = (typeof BINARIES)[number];

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

export function sha256File(filePath: string): string {
  const hash = createHash("sha256");
  hash.update(readFileSync(filePath));
  return hash.digest("hex");
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

function isNonEmptyFile(candidatePath: string): boolean {
  return existsSync(candidatePath) && statSync(candidatePath).size > 0;
}

function locatePrebuiltBinary(
  binaryName: BinaryName,
  spec: ArchTargetSpec,
  fromDir?: string,
): string | null {
  const searchDirs = fromDir
    ? [resolve(fromDir)]
    : [join(REPO_ROOT, "release"), join(REPO_ROOT, "dist")];

  for (const dir of searchDirs) {
    for (const suffix of spec.distSuffixes) {
      const candidate = join(dir, `${binaryName}-linux-${suffix}`);
      if (isNonEmptyFile(candidate)) {
        return candidate;
      }
    }
    const bareCandidate = join(dir, binaryName);
    if (fromDir && isNonEmptyFile(bareCandidate)) {
      return bareCandidate;
    }
  }

  if (binaryName !== "oci-cri") {
    for (const target of spec.rustTargets) {
      for (const profile of ["dist", "release"]) {
        const candidate = join(REPO_ROOT, "target", target, profile, binaryName);
        if (isNonEmptyFile(candidate)) {
          return candidate;
        }
      }
    }
  }

  return null;
}

const downloadedOciCriCache = new Map<"amd64" | "arm64", string>();

export function verifyArchiveSha256(
  archivePath: string,
  expectedSha256: string,
  maxBytes: number = MAX_OCI_CRI_ARCHIVE_BYTES,
): string {
  let stat;
  try {
    stat = lstatSync(archivePath);
  } catch (err) {
    throw new Error(`Archive not found at ${archivePath}: ${err}`);
  }

  if (stat.isSymbolicLink()) {
    throw new Error(`Archive at ${archivePath} is a symbolic link`);
  }
  if (!stat.isFile()) {
    throw new Error(`Archive at ${archivePath} is not a regular file`);
  }
  if (stat.size === 0) {
    throw new Error(`Archive at ${archivePath} is empty (0 bytes)`);
  }
  if (stat.size > maxBytes) {
    throw new Error(
      `Archive at ${archivePath} exceeds maximum size limit of ${maxBytes} bytes (size: ${stat.size} bytes)`,
    );
  }

  const actualSha256 = sha256File(archivePath);
  if (actualSha256 !== expectedSha256) {
    throw new Error(
      `SHA-256 mismatch for ${archivePath}: expected ${expectedSha256}, got ${actualSha256}`,
    );
  }

  return actualSha256;
}

function resolveTarBinary(): string {
  for (const candidate of ["/usr/bin/tar", "/bin/tar"]) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return "tar";
}

function extractTarEntryPath(line: string): string {
  const tokens = line.trim().split(/\s+/);
  if (
    tokens.length >= 6 &&
    /^\d{4}-\d{2}-\d{2}$/.test(tokens[3]!) &&
    /^\d{1,2}:\d{2}/.test(tokens[4]!)
  ) {
    let idx = 0;
    for (let i = 0; i < 5; i++) {
      idx = line.indexOf(tokens[i]!, idx) + tokens[i]!.length;
    }
    return line.slice(idx).trim();
  }
  if (tokens.length >= 9 && /^[A-Za-z]{3}$/.test(tokens[5]!)) {
    let idx = 0;
    for (let i = 0; i < 8; i++) {
      idx = line.indexOf(tokens[i]!, idx) + tokens[i]!.length;
    }
    return line.slice(idx).trim();
  }
  if (tokens.length >= 2) {
    return tokens[tokens.length - 1]!;
  }
  throw new Error(`Unable to parse entry path from archive entry: ${line}`);
}

export function extractAndValidateOciCriArchive(
  archivePath: string,
  destDir: string,
  maxBinaryBytes: number = MAX_OCI_CRI_BINARY_BYTES,
): string {
  const tarBin = resolveTarBinary();
  const listRes = spawnSync(tarBin, ["-ztvf", archivePath], {
    encoding: "utf8",
  });
  if (listRes.error) {
    throw listRes.error;
  }
  if (listRes.status !== 0) {
    throw new Error(
      `Failed to inspect archive ${archivePath}: ${listRes.stderr?.trim() || `exit code ${listRes.status}`}`,
    );
  }

  const entries = listRes.stdout
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);

  if (entries.length === 0) {
    throw new Error(`Archive ${archivePath} contains no entries`);
  }

  for (const entry of entries) {
    const firstChar = entry.charAt(0);
    if (firstChar !== "-" && firstChar !== "d") {
      throw new Error(
        `Disallowed entry type '${firstChar}' in archive entry: ${entry}`,
      );
    }
    if (entry.includes(" -> ") || entry.includes(" link to ")) {
      throw new Error(`Disallowed link entry in archive: ${entry}`);
    }

    const entryPath = extractTarEntryPath(entry);
    if (entryPath.startsWith("/") || entryPath.startsWith("\\") || isAbsolute(entryPath)) {
      throw new Error(
        `Disallowed absolute path in archive entry: "${entryPath}"`,
      );
    }
    const segments = entryPath.split(/[/\\]+/);
    if (segments.includes("..")) {
      throw new Error(
        `Path traversal ("..") detected in archive entry: "${entryPath}"`,
      );
    }
    const normalized = normalize(entryPath);
    if (
      normalized.startsWith("..") ||
      normalized === ".." ||
      isAbsolute(normalized)
    ) {
      throw new Error(
        `Entry path "${entryPath}" escapes archive root (normalizes to "${normalized}")`,
      );
    }
  }

  const extractRes = spawnSync(
    tarBin,
    [
      "-xzf",
      archivePath,
      "--no-same-owner",
      "--no-same-permissions",
      "-C",
      destDir,
    ],
    {
      cwd: destDir,
      encoding: "utf8",
    },
  );
  if (extractRes.error) {
    throw extractRes.error;
  }
  if (extractRes.status !== 0) {
    throw new Error(
      `Failed to extract archive ${archivePath}: ${extractRes.stderr?.trim() || `exit code ${extractRes.status}`}`,
    );
  }

  const extractedBin = join(destDir, "oci-cri");
  let binStat;
  try {
    binStat = lstatSync(extractedBin);
  } catch {
    throw new Error(
      `Extracted archive did not contain oci-cri at ${extractedBin}`,
    );
  }

  if (binStat.isSymbolicLink()) {
    throw new Error(`Extracted binary ${extractedBin} is a symbolic link`);
  }
  if (!binStat.isFile()) {
    throw new Error(`Extracted binary ${extractedBin} is not a regular file`);
  }
  if (binStat.size <= 0) {
    throw new Error(`Extracted binary ${extractedBin} is empty (0 bytes)`);
  }
  if (binStat.size > maxBinaryBytes) {
    throw new Error(
      `Extracted binary ${extractedBin} exceeds maximum size limit of ${maxBinaryBytes} bytes (size: ${binStat.size} bytes)`,
    );
  }

  return extractedBin;
}

function downloadAndVerifyOciCri(
  ociCriArch: "amd64" | "arm64",
  dryRun: boolean = false,
): string {
  const cached = downloadedOciCriCache.get(ociCriArch);
  if (cached && isNonEmptyFile(cached)) {
    return cached;
  }

  const releaseSpec = OCI_CRI_RELEASES[ociCriArch];
  validateOciCriReleaseSpec(ociCriArch, releaseSpec);

  if (dryRun) {
    console.log(
      `[dry-run] download ${releaseSpec.url} (sha256: ${releaseSpec.sha256})`,
    );
    return `<downloaded:oci-cri-${ociCriArch}>`;
  }

  const tempDir = mkdtempSync(join(tmpdir(), `diorama-oci-cri-${ociCriArch}-`));
  const archivePath = join(tempDir, `oci-cri-${ociCriArch}.tar.gz`);
  try {
    console.log(
      `Downloading prebuilt oci-cri (${OCI_LIB_VERSION}, ${ociCriArch}) from ${releaseSpec.url}...`,
    );
    runCommand(
      "curl",
      [
        "--fail",
        "--location",
        "--retry",
        "3",
        "--proto",
        "=https",
        "--tlsv1.2",
        "--output",
        archivePath,
        releaseSpec.url,
      ],
      tempDir,
    );

    verifyArchiveSha256(
      archivePath,
      releaseSpec.sha256,
      MAX_OCI_CRI_ARCHIVE_BYTES,
    );
    const extractedBin = extractAndValidateOciCriArchive(
      archivePath,
      tempDir,
      MAX_OCI_CRI_BINARY_BYTES,
    );
    chmodSync(extractedBin, 0o755);
    downloadedOciCriCache.set(ociCriArch, extractedBin);
    return extractedBin;
  } catch (err) {
    rmSync(tempDir, { recursive: true, force: true });
    throw err;
  }
}

export interface StageOptions {
  readonly arches: readonly ArchName[];
  readonly fromDir?: string;
  readonly compile: boolean;
  readonly profile: string;
  readonly version?: string;
  readonly dryRun?: boolean;
  readonly npmRoot?: string;
}

export function stageBinaries(options: StageOptions): void {
  const npmRoot = options.npmRoot ?? NPM_ROOT;
  const dryRun = options.dryRun ?? false;

  if (options.version) {
    syncWorkspaceVersion(options.version, npmRoot, dryRun);
  }

  const compiledTargets = new Set<string>();
  const compressedSources = new Map<string, string>();
  const upxBin = resolveUpxBinary();

  for (const arch of options.arches) {
    const spec = ARCH_TARGETS[arch];
    const pkgDir = join(npmRoot, "packages", arch);
    const binDir = join(pkgDir, "bin");

    if (!dryRun) {
      mkdirSync(binDir, { recursive: true });
    }

    for (const binaryName of BINARIES) {
      let sourcePath = options.compile
        ? null
        : locatePrebuiltBinary(binaryName, spec, options.fromDir);

      if (!sourcePath && binaryName === "oci-cri") {
        sourcePath = downloadAndVerifyOciCri(spec.ociCriArch, dryRun);
      }

      if (!sourcePath) {
        const rustTarget = spec.rustTargets[0]!;
        if (!compiledTargets.has(rustTarget)) {
          const cargoArgs = [
            "build",
            "--locked",
            "--profile",
            options.profile,
            "-p",
            "diorama",
            "-p",
            "dioramactl",
            "--target",
            rustTarget,
          ];
          if (dryRun) {
            console.log(
              `[dry-run] (cwd: ${REPO_ROOT}) cargo ${cargoArgs.join(" ")}`,
            );
          } else {
            console.log(
              `Building diorama and dioramactl for ${rustTarget} (profile: ${options.profile})...`,
            );
            runCommand("cargo", cargoArgs, REPO_ROOT);
          }
          compiledTargets.add(rustTarget);
        }
        sourcePath = join(
          REPO_ROOT,
          "target",
          rustTarget,
          options.profile,
          binaryName,
        );
      }

      const destPath = join(binDir, binaryName);
      if (dryRun) {
        console.log(
          `[dry-run] stage ${sourcePath} -> ${destPath} (chmod 0755${upxBin ? ", upx --best --lzma" : ""})`,
        );
      } else {
        if (!existsSync(sourcePath)) {
          throw new Error(
            `Expected binary not found at ${sourcePath} for ${spec.packageName}`,
          );
        }
        const rawBytes = statSync(sourcePath).size;
        const alreadyCompressedPath = compressedSources.get(sourcePath);
        if (alreadyCompressedPath && existsSync(alreadyCompressedPath)) {
          copyFileSync(alreadyCompressedPath, destPath);
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
    if (!isNonEmptyFile(binPath)) {
      throw new Error(
        `Missing staged binary ${binPath} for @diorama/${arch}. Run "bun run packages/publisher-cli/index.ts build" first.`,
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
        const status = isNonEmptyFile(binPath)
          ? `present (${(statSync(binPath).size / (1024 * 1024)).toFixed(2)} MB)`
          : "not yet staged";
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
        if (isNonEmptyFile(binPath)) {
          return `bin/${bin} (${(statSync(binPath).size / (1024 * 1024)).toFixed(2)} MB staged)`;
        }
        const prebuilt = locatePrebuiltBinary(bin, spec);
        if (prebuilt) {
          return `bin/${bin} (ready to stage from ${prebuilt}, ${(statSync(prebuilt).size / (1024 * 1024)).toFixed(2)} MB)`;
        }
        if (bin === "oci-cri") {
          return `bin/${bin} (will download from geoffsee/oci-lib@${OCI_LIB_VERSION})`;
        }
        return `bin/${bin} (requires cargo build)`;
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
  build              Stage release binaries and prebuilt oci-cri into packages/<arch>/bin
  stage              Alias for build (stages prebuilt release binaries when present)
  pack               Verify staged binaries and create .tgz tarballs via "bun pm pack"
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
