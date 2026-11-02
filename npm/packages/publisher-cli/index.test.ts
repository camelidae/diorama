import { describe, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  inferNpmDistTag,
  normalizeVersion,
  OCI_CRI_RELEASES,
  OCI_LIB_VERSION,
  sha256File,
  stageBinaries,
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
