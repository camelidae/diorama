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
  hasOciCriFeature,
  inferNpmDistTag,
  normalizeVersion,
  OCI_CRI_ENABLED_MARKER,
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

  test("detects oci-cri feature marker and rejects non-oci-cri diorama binaries", () => {
    const tempRoot = mkdtempSync(join(tmpdir(), "diorama-oci-cri-check-"));
    try {
      const enabledBin = join(tempRoot, "diorama-oci");
      const disabledBin = join(tempRoot, "diorama-no-oci");
      writeFileSync(enabledBin, `ELF...${OCI_CRI_ENABLED_MARKER}...`);
      writeFileSync(
        disabledBin,
        "ELF...managed oci-cri requires a build with --features oci-cri...",
      );
      expect(hasOciCriFeature(enabledBin)).toBe(true);
      expect(hasOciCriFeature(disabledBin)).toBe(false);
    } finally {
      rmSync(tempRoot, { recursive: true, force: true });
    }
  });

  test("syncs versions and stages prebuilt oci-cri release binaries across workspace", () => {
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
        writeFileSync(
          join(distDir, `diorama-linux-${arch}`),
          `diorama-${arch}-${OCI_CRI_ENABLED_MARKER}`,
        );
        writeFileSync(
          join(distDir, `dioramactl-linux-${arch}`),
          `dioramactl-${arch}`,
        );
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
        version: "v0.1.0-pr.9",
        npmRoot,
      });

      const rootManifest = JSON.parse(
        readFileSync(join(npmRoot, "package.json"), "utf8"),
      );
      expect(rootManifest.version).toBe("0.1.0-pr.9");
      expect(rootManifest.optionalDependencies).toEqual({
        "@diorama/amd64": "0.1.0-pr.9",
        "@diorama/arm64": "0.1.0-pr.9",
        "@diorama/aarch64": "0.1.0-pr.9",
      });

      for (const arch of ["amd64", "arm64", "aarch64"] as const) {
        expect(() => verifyPackageStaged(arch, npmRoot)).not.toThrow();
      }
    } finally {
      rmSync(tempRoot, { recursive: true, force: true });
    }
  });
});
