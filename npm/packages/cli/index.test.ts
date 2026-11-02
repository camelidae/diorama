import { describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import {
  buildRuntimeEnv,
  DEFAULT_CONTAINER_RUNTIME,
  DEFAULT_CONTAINER_RUNTIME_ENDPOINT,
  hasExplicitRuntimeSelection,
  isReadOnlyInvocation,
  PLATFORM_PACKAGES,
  resolveBinaryPath,
  resolvePlatformPackages,
} from "./index.ts";

const PACKAGES_DIR = resolve(import.meta.dir, "..");

describe("@diorama/cli multi-architecture resolution & oci-cri runtime linkage", () => {
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

  test("defaults DIORAMA_CONTAINER_RUNTIME_ENDPOINT to oci-cri socket while preserving overrides", () => {
    expect(DEFAULT_CONTAINER_RUNTIME).toBe("oci-cri");
    expect(DEFAULT_CONTAINER_RUNTIME_ENDPOINT).toBe(
      "unix:///run/oci-cri/oci-cri.sock",
    );
    expect(
      buildRuntimeEnv("diorama", {}).DIORAMA_CONTAINER_RUNTIME_ENDPOINT,
    ).toBe("unix:///run/oci-cri/oci-cri.sock");
    expect(
      buildRuntimeEnv("diorama", { DIORAMA_CONTAINER_RUNTIME: "containerd" })
        .DIORAMA_CONTAINER_RUNTIME_ENDPOINT,
    ).toBeUndefined();
    expect(
      buildRuntimeEnv("diorama", {
        DIORAMA_CONTAINER_RUNTIME_ENDPOINT: "unix:///run/containerd/containerd.sock",
      }).DIORAMA_CONTAINER_RUNTIME_ENDPOINT,
    ).toBe("unix:///run/containerd/containerd.sock");
    expect(
      buildRuntimeEnv("dioramactl", {}).DIORAMA_CONTAINER_RUNTIME_ENDPOINT,
    ).toBeUndefined();
  });

  test("identifies read-only flags and explicit runtime overrides", () => {
    expect(isReadOnlyInvocation(["--version"])).toBe(true);
    expect(isReadOnlyInvocation(["-v"])).toBe(true);
    expect(isReadOnlyInvocation(["--help"])).toBe(true);
    expect(isReadOnlyInvocation(["--print-config"])).toBe(true);
    expect(isReadOnlyInvocation(["--config=/etc/diorama/config.yaml"])).toBe(
      false,
    );

    expect(
      hasExplicitRuntimeSelection(["--container-runtime=containerd"], {}),
    ).toBe(true);
    expect(
      hasExplicitRuntimeSelection(
        ["--container-runtime-endpoint=unix:///run/crio/crio.sock"],
        {},
      ),
    ).toBe(true);
    expect(
      hasExplicitRuntimeSelection([], { DIORAMA_CONTAINER_RUNTIME: "containerd" }),
    ).toBe(true);
    expect(hasExplicitRuntimeSelection([], {})).toBe(false);
  });

  test("locates diorama, dioramactl, and oci-cri binaries from workspace platform packages", () => {
    for (const [key, spec] of Object.entries(PLATFORM_PACKAGES)) {
      const [platform, arch] = key.split("-") as [string, string];
      const primaryDir = spec.dirs[0]!;
      const binDir = join(PACKAGES_DIR, primaryDir, "bin");
      const createdPaths: string[] = [];
      const hadBinDir = existsSync(binDir);
      mkdirSync(binDir, { recursive: true });
      try {
        for (const bin of ["diorama", "dioramactl", "oci-cri"] as const) {
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
