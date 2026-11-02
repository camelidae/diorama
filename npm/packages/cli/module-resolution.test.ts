import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { resolveBinaryPath, verifyTrustedBinaryPath } from "./resolve.js";

const PACKAGES_DIR = resolve(import.meta.dirname, "..");

describe("@diorama/cli secure module & binary resolution (Issue #565)", () => {
  let tempDirs: string[] = [];

  function createTempDir(prefix = "diorama-test-"): string {
    const dir = mkdtempSync(join(tmpdir(), prefix));
    tempDirs.push(dir);
    return dir;
  }

  afterEach(() => {
    for (const dir of tempDirs) {
      rmSync(dir, { recursive: true, force: true });
    }
    tempDirs = [];
  });

  test("anchors package resolution to installation root and rejects untrusted process.cwd()", () => {
    const evilCwd = createTempDir("diorama-evil-cwd-");
    const evilPkgDir = join(evilCwd, "node_modules", "@diorama", "amd64");
    const evilBinDir = join(evilPkgDir, "bin");
    mkdirSync(evilBinDir, { recursive: true, mode: 0o755 });
    writeFileSync(
      join(evilPkgDir, "package.json"),
      JSON.stringify({ name: "@diorama/amd64", version: "0.1.0" }),
      { mode: 0o644 },
    );
    const evilBinary = join(evilBinDir, "diorama");
    writeFileSync(evilBinary, "#!/bin/sh\necho malicious\n", { mode: 0o755 });

    const origCwd = process.cwd();
    const origNodePath = process.env.NODE_PATH;
    try {
      process.chdir(evilCwd);
      process.env.NODE_PATH = join(evilCwd, "node_modules");

      // Because platform package resolution is anchored strictly to import.meta.dirname
      // and PACKAGES_DIR, the rogue package in process.cwd() / untrusted NODE_PATH
      // must NOT be resolved.
      expect(() => resolveBinaryPath("diorama", "linux", "x64")).toThrow(
        /Could not locate native "diorama" binary for "linux-x64"/,
      );
    } finally {
      process.chdir(origCwd);
      if (origNodePath === undefined) {
        delete process.env.NODE_PATH;
      } else {
        process.env.NODE_PATH = origNodePath;
      }
    }
  });

  test("rejects candidate binary path outside allowed root directory", () => {
    const root = createTempDir("diorama-allowed-root-");
    const outsideDir = createTempDir("diorama-outside-");
    const outsideBin = join(outsideDir, "diorama");
    writeFileSync(outsideBin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    expect(() => verifyTrustedBinaryPath(outsideBin, root)).toThrow(
      /is outside allowed root/,
    );
  });

  test("rejects candidate binary when it is a symlink resolving outside allowed root", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });

    const externalDir = createTempDir("diorama-ext-");
    const externalBin = join(externalDir, "external-payload");
    writeFileSync(externalBin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    const symlinkCandidate = join(binDir, "diorama");
    symlinkSync(externalBin, symlinkCandidate);

    expect(() => verifyTrustedBinaryPath(symlinkCandidate, root)).toThrow(
      /Untrusted symlink detected/,
    );
  });

  test("rejects candidate binary when a parent directory is a symlink resolving outside allowed root", () => {
    const root = createTempDir("diorama-root-");
    const externalDir = createTempDir("diorama-ext-");
    const payload = join(externalDir, "diorama");
    writeFileSync(payload, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    // root/bin -> externalDir
    const binSymlink = join(root, "bin");
    symlinkSync(externalDir, binSymlink);

    const candidate = join(binSymlink, "diorama");
    expect(() => verifyTrustedBinaryPath(candidate, root)).toThrow(
      /Untrusted symlink detected/,
    );
  });

  test("rejects group-writable binary when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    chmodSync(bin, 0o775);

    expect(() =>
      verifyTrustedBinaryPath(bin, root, { euid: 0 }),
    ).toThrow(/Insecure permissions.*writable by group or others/);
  });

  test("rejects world-writable binary when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    chmodSync(bin, 0o757);

    expect(() =>
      verifyTrustedBinaryPath(bin, root, { euid: 0 }),
    ).toThrow(/Insecure permissions.*writable by group or others/);
  });

  test("rejects group-writable parent directory when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    chmodSync(binDir, 0o775);
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    expect(() =>
      verifyTrustedBinaryPath(bin, root, { euid: 0 }),
    ).toThrow(/Insecure permissions.*bin.*writable by group or others/);
  });

  test("rejects group-writable root directory when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    chmodSync(root, 0o777);
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    expect(() =>
      verifyTrustedBinaryPath(bin, root, { euid: 0 }),
    ).toThrow(/Insecure permissions.*writable by group or others/);
  });

  test("rejects non-root-owned binary when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    // File is owned by current non-root user (e.g. UID 501 or 1000).
    expect(() =>
      verifyTrustedBinaryPath(bin, root, { euid: 0 }),
    ).toThrow(/Untrusted ownership.*owned by UID [0-9]+, expected root \(0\)/);
  });

  test("rejects group-writable symlink target when resolving as root (euid === 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const realBin = join(binDir, "diorama-real");
    writeFileSync(realBin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    chmodSync(realBin, 0o777);

    const symlinkBin = join(binDir, "diorama");
    symlinkSync(realBin, symlinkBin);

    expect(() =>
      verifyTrustedBinaryPath(symlinkBin, root, { euid: 0 }),
    ).toThrow(/Insecure target permissions.*writable by group or others/);
  });

  test("allows verified binary for unprivileged execution (euid !== 0)", () => {
    const root = createTempDir("diorama-root-");
    const binDir = join(root, "bin");
    mkdirSync(binDir, { recursive: true, mode: 0o755 });
    const bin = join(binDir, "diorama");
    writeFileSync(bin, "#!/bin/sh\nexit 0\n", { mode: 0o755 });

    expect(verifyTrustedBinaryPath(bin, root, { euid: 1000 })).toBe(bin);
  });

  test("resolveBinaryPath enforces security verification options on candidate binaries", () => {
    const amd64BinDir = join(PACKAGES_DIR, "amd64", "bin");
    const hadBinDir = existsSync(amd64BinDir);
    mkdirSync(amd64BinDir, { recursive: true, mode: 0o755 });

    const candidate = join(amd64BinDir, "diorama");
    writeFileSync(candidate, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
    chmodSync(candidate, 0o777);

    try {
      // With euid: 0, the group/world-writable candidate in workspace must be rejected
      expect(() =>
        resolveBinaryPath("diorama", "linux", "x64", { euid: 0 }),
      ).toThrow(/Insecure permissions.*writable by group or others/);
    } finally {
      rmSync(candidate, { force: true });
      if (!hadBinDir) {
        rmSync(amd64BinDir, { recursive: true, force: true });
      }
    }
  });
});
