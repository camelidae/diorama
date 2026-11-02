import { describe, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  ALLOWED_OCI_LIB_RELEASE_PREFIX,
  extractAndValidateOciCriArchive,
  inferNpmDistTag,
  MAX_OCI_CRI_ARCHIVE_BYTES,
  MAX_OCI_CRI_BINARY_BYTES,
  normalizeVersion,
  OCI_CRI_RELEASES,
  OCI_LIB_VERSION,
  sha256File,
  stageBinaries,
  validateOciCriReleaseSpec,
  verifyArchiveSha256,
  verifyPackageStaged,
} from "./index.ts";

describe("@diorama/publisher-cli build & publish helper", () => {
  test("normalizes semver tags and infers npm dist-tags", () => {
    expect(normalizeVersion("v0.1.0-pr.9")).toBe("0.1.0-pr.9");
    expect(normalizeVersion("0.2.0")).toBe("0.2.0");
    expect(() => normalizeVersion("invalid-version")).toThrow(
      /Invalid semantic version/,
    );
    expect(inferNpmDistTag("0.1.0-pr.9")).toBe("next");
    expect(inferNpmDistTag("0.1.0")).toBe("latest");
  });

  test("pins oci-lib release URLs and SHA-256 digests", () => {
    expect(OCI_LIB_VERSION).toBe("v0.1.14");
    expect(MAX_OCI_CRI_ARCHIVE_BYTES).toBe(64 * 1024 * 1024);
    expect(MAX_OCI_CRI_BINARY_BYTES).toBe(128 * 1024 * 1024);
    expect(ALLOWED_OCI_LIB_RELEASE_PREFIX).toBe(
      "https://github.com/geoffsee/oci-lib/releases/download/",
    );
    expect(OCI_CRI_RELEASES.amd64.sha256).toHaveLength(64);
    expect(OCI_CRI_RELEASES.arm64.sha256).toHaveLength(64);

    const tempRoot = mkdtempSync(join(tmpdir(), "diorama-sha256-check-"));
    try {
      const sample = join(tempRoot, "sample.bin");
      writeFileSync(sample, "diorama-test-payload");
      expect(sha256File(sample)).toBe(
        "256dc006eb7573f77dc92200412c8196d30b6ac359e4c2ba4f8fb3fcb628200e",
      );
    } finally {
      rmSync(tempRoot, { recursive: true, force: true });
    }
  });

  test("validates pinned oci-lib release spec URL origin and SHA-256 digest format", () => {
    expect(() =>
      validateOciCriReleaseSpec("amd64", OCI_CRI_RELEASES.amd64),
    ).not.toThrow();
    expect(() =>
      validateOciCriReleaseSpec("arm64", OCI_CRI_RELEASES.arm64),
    ).not.toThrow();

    expect(() =>
      validateOciCriReleaseSpec("amd64", {
        url: "https://malicious.org/geoffsee/oci-lib/releases/download/v0.1.14/oci-cri.tar.gz",
        sha256: "0e2b24dcfb36caf413fc1a390c32652f580d99a45fe6239959de887f506601d9",
      }),
    ).toThrow(/Invalid release URL/);

    expect(() =>
      validateOciCriReleaseSpec("amd64", {
        url: `https://github.com/geoffsee/oci-lib/releases/download/v0.1.13/oci-cri.tar.gz`,
        sha256: "0e2b24dcfb36caf413fc1a390c32652f580d99a45fe6239959de887f506601d9",
      }),
    ).toThrow(/Invalid release URL/);

    expect(() =>
      validateOciCriReleaseSpec("amd64", {
        url: `https://github.com/geoffsee/oci-lib/releases/download/${OCI_LIB_VERSION}/oci-cri.tar.gz`,
        sha256: "0E2B24DCFB36CAF413FC1A390C32652F580D99A45FE6239959DE887F506601D9",
      }),
    ).toThrow(/Invalid SHA-256 digest/);

    expect(() =>
      validateOciCriReleaseSpec("amd64", {
        url: `https://github.com/geoffsee/oci-lib/releases/download/${OCI_LIB_VERSION}/oci-cri.tar.gz`,
        sha256: "0e2b24dcfb36caf413fc1a390c32652f580d99a45fe6239959de887f506601d",
      }),
    ).toThrow(/Invalid SHA-256 digest/);

    expect(() =>
      validateOciCriReleaseSpec("amd64", {
        url: `https://github.com/geoffsee/oci-lib/releases/download/${OCI_LIB_VERSION}/oci-cri.tar.gz`,
        sha256: "zz2b24dcfb36caf413fc1a390c32652f580d99a45fe6239959de887f506601d9",
      }),
    ).toThrow(/Invalid SHA-256 digest/);

    expect(() => validateOciCriReleaseSpec("amd64", null as never)).toThrow(
      /Release spec for architecture/,
    );
  });

  test("verifies archive SHA-256 and rejects mismatches, empty files, and oversized archives", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "diorama-archive-verify-"));
    try {
      const validArchive = join(tempDir, "valid.bin");
      writeFileSync(validArchive, "test-archive-payload");
      const expectedDigest = sha256File(validArchive);

      expect(verifyArchiveSha256(validArchive, expectedDigest)).toBe(
        expectedDigest,
      );

      const wrongDigest = "0".repeat(64);
      expect(() =>
        verifyArchiveSha256(validArchive, wrongDigest),
      ).toThrow(/SHA-256 mismatch/);

      const emptyArchive = join(tempDir, "empty.bin");
      writeFileSync(emptyArchive, "");
      expect(() =>
        verifyArchiveSha256(emptyArchive, expectedDigest),
      ).toThrow(/empty/);

      expect(() =>
        verifyArchiveSha256(validArchive, expectedDigest, 5),
      ).toThrow(/exceeds maximum size limit/);

      const missingArchive = join(tempDir, "nonexistent.bin");
      expect(() =>
        verifyArchiveSha256(missingArchive, expectedDigest),
      ).toThrow(/Archive not found/);

      const symlinkArchive = join(tempDir, "symlink.bin");
      symlinkSync(validArchive, symlinkArchive);
      expect(() =>
        verifyArchiveSha256(symlinkArchive, expectedDigest),
      ).toThrow(/symbolic link/);
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("inspects archive entries and rejects symlinks, path traversals, or invalid extracted binaries", () => {
    const tempDir = mkdtempSync(join(tmpdir(), "diorama-extract-test-"));
    try {
      // 1. Archive with symlink entry
      const symlinkSrcDir = join(tempDir, "sym-src");
      mkdirSync(symlinkSrcDir, { recursive: true });
      writeFileSync(join(symlinkSrcDir, "real-file"), "binary content");
      symlinkSync("real-file", join(symlinkSrcDir, "symlink-file"));
      const symlinkArchive = join(tempDir, "symlink.tar.gz");
      spawnSync(
        "tar",
        ["-czf", symlinkArchive, "-C", symlinkSrcDir, "real-file", "symlink-file"],
        { stdio: "ignore" },
      );

      const symlinkDest = join(tempDir, "dest-sym");
      mkdirSync(symlinkDest, { recursive: true });
      expect(() =>
        extractAndValidateOciCriArchive(symlinkArchive, symlinkDest),
      ).toThrow(/Disallowed entry type 'l'|Disallowed link entry/);

      // 2. Archive with path traversal (..)
      const travSrcDir = join(tempDir, "trav-src", "sub");
      mkdirSync(travSrcDir, { recursive: true });
      writeFileSync(join(tempDir, "trav-src", "outside-file"), "outside content");
      const travArchive = join(tempDir, "traversal.tar.gz");
      spawnSync("tar", ["-czf", travArchive, "../outside-file"], {
        cwd: travSrcDir,
        stdio: "ignore",
      });

      const travDest = join(tempDir, "dest-trav");
      mkdirSync(travDest, { recursive: true });
      expect(() =>
        extractAndValidateOciCriArchive(travArchive, travDest),
      ).toThrow(/Path traversal/);

      // 3. Archive missing oci-cri
      const missingBinSrc = join(tempDir, "missing-bin-src");
      mkdirSync(missingBinSrc, { recursive: true });
      writeFileSync(join(missingBinSrc, "not-oci-cri"), "dummy");
      const missingBinArchive = join(tempDir, "missing-bin.tar.gz");
      spawnSync(
        "tar",
        ["-czf", missingBinArchive, "-C", missingBinSrc, "not-oci-cri"],
        { stdio: "ignore" },
      );

      const missingBinDest = join(tempDir, "dest-missing");
      mkdirSync(missingBinDest, { recursive: true });
      expect(() =>
        extractAndValidateOciCriArchive(missingBinArchive, missingBinDest),
      ).toThrow(/Extracted archive did not contain oci-cri/);

      // 4. Archive with empty oci-cri binary
      const emptyBinSrc = join(tempDir, "empty-bin-src");
      mkdirSync(emptyBinSrc, { recursive: true });
      writeFileSync(join(emptyBinSrc, "oci-cri"), "");
      const emptyBinArchive = join(tempDir, "empty-bin.tar.gz");
      spawnSync(
        "tar",
        ["-czf", emptyBinArchive, "-C", emptyBinSrc, "oci-cri"],
        { stdio: "ignore" },
      );

      const emptyBinDest = join(tempDir, "dest-empty");
      mkdirSync(emptyBinDest, { recursive: true });
      expect(() =>
        extractAndValidateOciCriArchive(emptyBinArchive, emptyBinDest),
      ).toThrow(/empty/);

      // 5. Archive with oversized oci-cri binary
      const oversizedBinSrc = join(tempDir, "oversized-bin-src");
      mkdirSync(oversizedBinSrc, { recursive: true });
      writeFileSync(join(oversizedBinSrc, "oci-cri"), "1234567890");
      const oversizedBinArchive = join(tempDir, "oversized-bin.tar.gz");
      spawnSync(
        "tar",
        ["-czf", oversizedBinArchive, "-C", oversizedBinSrc, "oci-cri"],
        { stdio: "ignore" },
      );

      const oversizedBinDest = join(tempDir, "dest-oversized");
      mkdirSync(oversizedBinDest, { recursive: true });
      expect(() =>
        extractAndValidateOciCriArchive(oversizedBinArchive, oversizedBinDest, 5),
      ).toThrow(/exceeds maximum size limit/);

      // 6. Valid oci-cri archive extraction
      const validSrc = join(tempDir, "valid-src");
      mkdirSync(validSrc, { recursive: true });
      const validContent = "#!/bin/sh\necho valid oci-cri";
      writeFileSync(join(validSrc, "oci-cri"), validContent);
      chmodSync(join(validSrc, "oci-cri"), 0o755);
      const validArchive = join(tempDir, "valid-oci-cri.tar.gz");
      spawnSync(
        "tar",
        ["-czf", validArchive, "-C", validSrc, "oci-cri"],
        { stdio: "ignore" },
      );

      const validDest = join(tempDir, "dest-valid");
      mkdirSync(validDest, { recursive: true });
      const extractedBin = extractAndValidateOciCriArchive(
        validArchive,
        validDest,
      );
      expect(extractedBin).toBe(join(validDest, "oci-cri"));
      expect(readFileSync(extractedBin, "utf8")).toBe(validContent);
    } finally {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("syncs versions and stages prebuilt release binaries across workspace", () => {
    const tempRoot = mkdtempSync(join(tmpdir(), "diorama-npm-test-"));
    const distDir = join(tempRoot, "dist");
    const npmRoot = join(tempRoot, "npm");
    try {
      mkdirSync(distDir, { recursive: true });
      for (const arch of ["amd64", "arm64", "aarch64"] as const) {
        mkdirSync(join(npmRoot, "packages", arch), { recursive: true });
        writeFileSync(
          join(npmRoot, "packages", arch, "package.json"),
          JSON.stringify({ name: `@diorama/${arch}`, version: "0.0.0" }),
        );
        writeFileSync(join(distDir, `diorama-linux-${arch}`), `diorama-${arch}`);
        writeFileSync(
          join(distDir, `dioramactl-linux-${arch}`),
          `dioramactl-${arch}`,
        );
        writeFileSync(join(distDir, `oci-cri-linux-${arch}`), `oci-cri-${arch}`);
      }
      writeFileSync(
        join(npmRoot, "package.json"),
        JSON.stringify({
          name: "@diorama/cli",
          version: "0.0.0",
          optionalDependencies: {},
        }),
      );

      stageBinaries({
        arches: ["amd64", "arm64", "aarch64"],
        fromDir: distDir,
        compile: false,
        profile: "dist",
        version: "v0.1.1",
        npmRoot,
      });

      const rootManifest = JSON.parse(
        readFileSync(join(npmRoot, "package.json"), "utf8"),
      );
      expect(rootManifest.version).toBe("0.1.1");
      expect(rootManifest.optionalDependencies).toEqual({
        "@diorama/amd64": "0.1.1",
        "@diorama/arm64": "0.1.1",
        "@diorama/aarch64": "0.1.1",
      });

      for (const arch of ["amd64", "arm64", "aarch64"] as const) {
        expect(() => verifyPackageStaged(arch, npmRoot)).not.toThrow();
      }
    } finally {
      rmSync(tempRoot, { recursive: true, force: true });
    }
  });
});
