import { describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  buildRuntimeEnv,
  DEFAULT_CONTAINER_RUNTIME,
  PLATFORM_PACKAGES,
  resolveBinaryPath,
  resolvePlatformPackages,
} from "./index.ts";

const PACKAGES_DIR = resolve(import.meta.dir, "..");

describe("@diorama/cli multi-architecture resolution & oci-cri runtime default", () => {
  test("resolves platform package candidates for linux-x64 and linux-arm64", () => {
    expect(resolvePlatformPackages("linux", "x64")).toEqual(["@diorama/amd64"]);
    expect(resolvePlatformPackages("linux", "arm64")).toEqual([
      "@diorama/arm64",
      "@diorama/aarch64",
    ]);
  });

  test("rejects unsupported platform or architecture", () => {
    expect(() => resolvePlatformPackages("win32", "x64")).toThrow(
      /Unsupported platform\/architecture "win32-x64"/,
    );
  });

  test("defaults DIORAMA_CONTAINER_RUNTIME to oci-cri for diorama while preserving overrides", () => {
    expect(DEFAULT_CONTAINER_RUNTIME).toBe("oci-cri");
    expect(buildRuntimeEnv("diorama", {}).DIORAMA_CONTAINER_RUNTIME).toBe(
      "oci-cri",
    );
    expect(
      buildRuntimeEnv("diorama", { DIORAMA_CONTAINER_RUNTIME: "containerd" })
        .DIORAMA_CONTAINER_RUNTIME,
    ).toBe("containerd");
    expect(
      buildRuntimeEnv("dioramactl", {}).DIORAMA_CONTAINER_RUNTIME,
    ).toBeUndefined();
  });

  test("locates diorama and dioramactl binaries from workspace platform packages", () => {
    for (const [key, spec] of Object.entries(PLATFORM_PACKAGES)) {
      const [platform, arch] = key.split("-") as [string, string];
      const primaryDir = spec.dirs[0]!;
      const binDir = join(PACKAGES_DIR, primaryDir, "bin");
      const createdPaths: string[] = [];
      const hadBinDir = existsSync(binDir);
      mkdirSync(binDir, { recursive: true });
      try {
        for (const bin of ["diorama", "dioramactl"] as const) {
          const binPath = join(binDir, bin);
          if (!existsSync(binPath)) {
            writeFileSync(binPath, "#!/bin/sh\nexit 0\n", { mode: 0o755 });
            createdPaths.push(binPath);
          }
          expect(resolveBinaryPath(bin, platform, arch)).toBe(binPath);
        }
      } finally {
        for (const p of createdPaths) {
          rmSync(p, { force: true });
        }
        if (!hadBinDir) {
          rmSync(binDir, { recursive: true, force: true });
        }
      }
    }
  });
});
