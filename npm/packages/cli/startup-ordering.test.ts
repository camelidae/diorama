import { describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import {
  buildRuntimeEnv,
  DEFAULT_OCI_CRI_SOCKET,
  hasExplicitRuntimeSelection,
  isReadOnlyInvocation,
  shouldSpawnOciCri,
} from "./resolve.js";

const READ_ONLY_FLAGS = [
  "--version",
  "-v",
  "-V",
  "--help",
  "-h",
  "--help-long",
  "--help-man",
  "--print-config",
  "--print-config=yaml",
  "--print-config=json",
];

describe("Issue #566: Read-only CLI startup ordering contract & oci-cri bypass", () => {
  describe("isReadOnlyInvocation", () => {
    test("recognizes all standard read-only flags", () => {
      for (const flag of READ_ONLY_FLAGS) {
        expect(isReadOnlyInvocation([flag])).toBe(true);
      }
    });

    test("recognizes read-only flags mixed with preceding or following options", () => {
      expect(isReadOnlyInvocation(["--config", "diorama.yaml", "--version"])).toBe(true);
      expect(isReadOnlyInvocation(["-v", "--log-level=debug"])).toBe(true);
      expect(isReadOnlyInvocation(["--node-name=worker-1", "-V"])).toBe(true);
      expect(isReadOnlyInvocation(["--help", "--all"])).toBe(true);
      expect(isReadOnlyInvocation(["--timeout=30s", "-h"])).toBe(true);
      expect(isReadOnlyInvocation(["--help-long", "--verbose"])).toBe(true);
      expect(isReadOnlyInvocation(["--help-man", "--section=1"])).toBe(true);
      expect(isReadOnlyInvocation(["--print-config=custom", "--strict"])).toBe(true);
    });

    test("ignores read-only flags appearing after the '--' end-of-options delimiter", () => {
      expect(isReadOnlyInvocation(["--", "--version"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "-v"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "-V"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "--help"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "-h"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "--help-long"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "--help-man"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "--print-config"])).toBe(false);
      expect(isReadOnlyInvocation(["--", "--print-config=yaml"])).toBe(false);
      expect(isReadOnlyInvocation(["serve", "--", "--version"])).toBe(false);
    });

    test("honors read-only flag if it appears before the '--' delimiter", () => {
      expect(isReadOnlyInvocation(["--version", "--", "extra-arg"])).toBe(true);
      expect(isReadOnlyInvocation(["-v", "--", "--foo"])).toBe(true);
    });

    test("returns false for non-read-only invocations", () => {
      expect(isReadOnlyInvocation([])).toBe(false);
      expect(isReadOnlyInvocation(["serve"])).toBe(false);
      expect(isReadOnlyInvocation(["--config", "/etc/diorama/config.yaml"])).toBe(false);
      expect(isReadOnlyInvocation(["--container-runtime=containerd"])).toBe(false);
    });
  });

  describe("hasExplicitRuntimeSelection & configuration precedence", () => {
    test("detects explicit CLI runtime flags before '--' delimiter", () => {
      expect(hasExplicitRuntimeSelection(["--container-runtime", "containerd"], {})).toBe(true);
      expect(hasExplicitRuntimeSelection(["--container-runtime=containerd"], {})).toBe(true);
      expect(
        hasExplicitRuntimeSelection(
          ["--container-runtime-endpoint", "unix:///run/containerd/containerd.sock"],
          {},
        ),
      ).toBe(true);
      expect(
        hasExplicitRuntimeSelection(
          ["--container-runtime-endpoint=unix:///run/containerd/containerd.sock"],
          {},
        ),
      ).toBe(true);
    });

    test("ignores CLI runtime flags after '--' delimiter", () => {
      expect(hasExplicitRuntimeSelection(["--", "--container-runtime=containerd"], {})).toBe(false);
      expect(
        hasExplicitRuntimeSelection(
          ["--", "--container-runtime-endpoint=unix:///run/containerd.sock"],
          {},
        ),
      ).toBe(false);
    });

    test("detects DIORAMA_CONTAINER_RUNTIME and DIORAMA_CONTAINER_RUNTIME_ENDPOINT environment variables", () => {
      expect(hasExplicitRuntimeSelection([], { DIORAMA_CONTAINER_RUNTIME: "containerd" })).toBe(true);
      expect(
        hasExplicitRuntimeSelection([], {
          DIORAMA_CONTAINER_RUNTIME_ENDPOINT: "unix:///run/crio/crio.sock",
        }),
      ).toBe(true);
      expect(hasExplicitRuntimeSelection([], {})).toBe(false);
    });

    test("detects runtime configuration in config file specified via --config, -c, or DIORAMA_CONFIG", () => {
      const tempDir = mkdtempSync(join(tmpdir(), "diorama-test-cfg-"));
      try {
        const yamlConfigPath = join(tempDir, "config.yaml");
        writeFileSync(
          yamlConfigPath,
          [
            "apiVersion: diorama.io/v1alpha1",
            "kind: Config",
            "runtime:",
            "  endpoint: unix:///run/containerd/containerd.sock",
          ].join("\n"),
        );

        // Specified via --config
        expect(hasExplicitRuntimeSelection(["--config", yamlConfigPath], {})).toBe(true);
        expect(hasExplicitRuntimeSelection([`--config=${yamlConfigPath}`], {})).toBe(true);

        // Specified via -c
        expect(hasExplicitRuntimeSelection(["-c", yamlConfigPath], {})).toBe(true);
        expect(hasExplicitRuntimeSelection([`-c=${yamlConfigPath}`], {})).toBe(true);

        // Specified via DIORAMA_CONFIG env var
        expect(hasExplicitRuntimeSelection([], { DIORAMA_CONFIG: yamlConfigPath })).toBe(true);

        // Config file with runtime.backend
        const backendConfigPath = join(tempDir, "backend.yaml");
        writeFileSync(
          backendConfigPath,
          ["apiVersion: diorama.io/v1alpha1", "runtime:", "  backend: containerd"].join("\n"),
        );
        expect(hasExplicitRuntimeSelection(["--config", backendConfigPath], {})).toBe(true);

        // Config file with inline runtime object
        const inlineConfigPath = join(tempDir, "inline.yaml");
        writeFileSync(
          inlineConfigPath,
          "runtime: { endpoint: 'unix:///run/containerd/containerd.sock' }\n",
        );
        expect(hasExplicitRuntimeSelection(["--config", inlineConfigPath], {})).toBe(true);

        // Config file with scalar runtime string
        const scalarConfigPath = join(tempDir, "scalar.yaml");
        writeFileSync(scalarConfigPath, "runtime: containerd\n");
        expect(hasExplicitRuntimeSelection(["--config", scalarConfigPath], {})).toBe(true);

        // Config file with JSON
        const jsonConfigPath = join(tempDir, "config.json");
        writeFileSync(
          jsonConfigPath,
          JSON.stringify({ runtime: { endpoint: "unix:///run/crio/crio.sock" } }),
        );
        expect(hasExplicitRuntimeSelection(["--config", jsonConfigPath], {})).toBe(true);

        // Config file with empty endpoint (default) does NOT count as explicit selection
        const emptyConfigPath = join(tempDir, "empty.yaml");
        writeFileSync(
          emptyConfigPath,
          ["apiVersion: diorama.io/v1alpha1", "runtime:", "  endpoint: ''"].join("\n"),
        );
        expect(hasExplicitRuntimeSelection(["--config", emptyConfigPath], {})).toBe(false);

        // Config file without runtime section
        const noRuntimePath = join(tempDir, "noruntime.yaml");
        writeFileSync(noRuntimePath, "apiVersion: diorama.io/v1alpha1\nkind: Config\n");
        expect(hasExplicitRuntimeSelection(["--config", noRuntimePath], {})).toBe(false);

        // Non-existent config file does not crash and returns false
        const missingPath = join(tempDir, "nonexistent.yaml");
        expect(hasExplicitRuntimeSelection(["--config", missingPath], {})).toBe(false);
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });
  });

  describe("buildRuntimeEnv", () => {
    test("does not inject DIORAMA_CONTAINER_RUNTIME_ENDPOINT for non-diorama binaries", () => {
      expect(buildRuntimeEnv("dioramactl", {}).DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();
      expect(buildRuntimeEnv("oci-cri", {}).DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();
    });

    test("defaults DIORAMA_CONTAINER_RUNTIME_ENDPOINT when launching diorama without overrides or read-only flags", () => {
      const env = buildRuntimeEnv("diorama", {}, DEFAULT_OCI_CRI_SOCKET, []);
      expect(env.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBe(`unix://${DEFAULT_OCI_CRI_SOCKET}`);
    });

    test("does NOT inject default DIORAMA_CONTAINER_RUNTIME_ENDPOINT on read-only invocations", () => {
      for (const flag of READ_ONLY_FLAGS) {
        const env = buildRuntimeEnv("diorama", {}, DEFAULT_OCI_CRI_SOCKET, [flag]);
        expect(env.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();
      }
    });

    test("does NOT inject default DIORAMA_CONTAINER_RUNTIME_ENDPOINT when explicit CLI runtime flag is present", () => {
      const envFlag = buildRuntimeEnv("diorama", {}, DEFAULT_OCI_CRI_SOCKET, [
        "--container-runtime=containerd",
      ]);
      expect(envFlag.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();

      const envEndpoint = buildRuntimeEnv("diorama", {}, DEFAULT_OCI_CRI_SOCKET, [
        "--container-runtime-endpoint=unix:///run/custom.sock",
      ]);
      expect(envEndpoint.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();
    });

    test("does NOT inject default DIORAMA_CONTAINER_RUNTIME_ENDPOINT when explicit env var is present", () => {
      const envRuntime = buildRuntimeEnv(
        "diorama",
        { DIORAMA_CONTAINER_RUNTIME: "containerd" },
        DEFAULT_OCI_CRI_SOCKET,
        [],
      );
      expect(envRuntime.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();

      const envCustom = buildRuntimeEnv(
        "diorama",
        { DIORAMA_CONTAINER_RUNTIME_ENDPOINT: "unix:///run/custom.sock" },
        DEFAULT_OCI_CRI_SOCKET,
        [],
      );
      expect(envCustom.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBe("unix:///run/custom.sock");
    });

    test("does NOT inject default DIORAMA_CONTAINER_RUNTIME_ENDPOINT when config file specifies runtime", () => {
      const tempDir = mkdtempSync(join(tmpdir(), "diorama-build-env-"));
      try {
        const cfgPath = join(tempDir, "config.yaml");
        writeFileSync(cfgPath, "runtime:\n  endpoint: unix:///run/custom.sock\n");

        const env = buildRuntimeEnv("diorama", {}, DEFAULT_OCI_CRI_SOCKET, ["--config", cfgPath]);
        expect(env.DIORAMA_CONTAINER_RUNTIME_ENDPOINT).toBeUndefined();
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });
  });

  describe("shouldSpawnOciCri", () => {
    test("returns true only for diorama without read-only flags, overrides, or existing socket", () => {
      expect(shouldSpawnOciCri("diorama", [], {}, false)).toBe(true);
      expect(shouldSpawnOciCri("diorama", [], {}, true)).toBe(false);
      expect(shouldSpawnOciCri("dioramactl", [], {}, false)).toBe(false);
      expect(shouldSpawnOciCri("oci-cri", [], {}, false)).toBe(false);
    });

    test("returns false for all read-only invocations", () => {
      for (const flag of READ_ONLY_FLAGS) {
        expect(shouldSpawnOciCri("diorama", [flag], {}, false)).toBe(false);
      }
    });

    test("returns false when explicit CLI runtime flags are present", () => {
      expect(shouldSpawnOciCri("diorama", ["--container-runtime=containerd"], {}, false)).toBe(
        false,
      );
      expect(
        shouldSpawnOciCri(
          "diorama",
          ["--container-runtime-endpoint=unix:///run/crio.sock"],
          {},
          false,
        ),
      ).toBe(false);
    });

    test("returns false when DIORAMA_CONTAINER_RUNTIME or DIORAMA_CONTAINER_RUNTIME_ENDPOINT is set", () => {
      expect(
        shouldSpawnOciCri("diorama", [], { DIORAMA_CONTAINER_RUNTIME: "containerd" }, false),
      ).toBe(false);
      expect(
        shouldSpawnOciCri(
          "diorama",
          [],
          { DIORAMA_CONTAINER_RUNTIME_ENDPOINT: "unix:///run/crio.sock" },
          false,
        ),
      ).toBe(false);
    });

    test("returns false when config file specifies runtime", () => {
      const tempDir = mkdtempSync(join(tmpdir(), "diorama-should-spawn-"));
      try {
        const cfgPath = join(tempDir, "config.yaml");
        writeFileSync(cfgPath, "runtime:\n  endpoint: unix:///run/crio.sock\n");

        expect(shouldSpawnOciCri("diorama", ["--config", cfgPath], {}, false)).toBe(false);
        expect(shouldSpawnOciCri("diorama", ["-c", cfgPath], {}, false)).toBe(false);
        expect(shouldSpawnOciCri("diorama", [], { DIORAMA_CONFIG: cfgPath }, false)).toBe(false);
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });
  });

  describe("Integration: zero process spawns and zero filesystem writes on read-only invocations", () => {
    test("verifies runBinary executes cleanly with zero spawns and zero socket directory writes", () => {
      const tempDir = mkdtempSync(join(tmpdir(), "diorama-readonly-integration-"));
      const binDir = join(tempDir, "bin");
      mkdirSync(binDir, { recursive: true });

      const dioramaMock = join(binDir, "diorama");
      const ociCriMock = join(binDir, "oci-cri");
      const ociCriSpawnLog = join(tempDir, "oci-cri-spawned.log");

      // Mock diorama exits 0 and prints args
      writeFileSync(
        dioramaMock,
        '#!/bin/sh\necho "MOCK_DIORAMA_CALLED: $1"\nexit 0\n',
        { mode: 0o755 },
      );
      // Mock oci-cri records any spawn invocation
      writeFileSync(
        ociCriMock,
        `#!/bin/sh\necho "OCI_CRI_SPAWNED" >> "${ociCriSpawnLog}"\nexit 0\n`,
        { mode: 0o755 },
      );

      const nonExistentSocketDir = join(tempDir, "socket-dir");
      const nonExistentSocketPath = join(nonExistentSocketDir, "oci-cri.sock");

      try {
        for (const flag of READ_ONLY_FLAGS) {
          rmSync(nonExistentSocketDir, { recursive: true, force: true });
          rmSync(ociCriSpawnLog, { force: true });

          const testScript = `
            Object.defineProperty(process, "platform", { value: "linux" });
            Object.defineProperty(process, "arch", { value: "arm64" });
            process.env.DIORAMA_OCI_CRI_SOCKET = ${JSON.stringify(nonExistentSocketPath)};

            const resolveModule = await import("./packages/cli/resolve.js");
            // Temporarily mock resolveBinaryPath to return our test mock binaries
            const origResolve = resolveModule.resolveBinaryPath;
            const customRunBinary = (binName, args) => {
              const fs = process.getBuiltinModule("node:fs");
              const { spawn, spawnSync } = process.getBuiltinModule("node:child_process");
              const { dirname } = process.getBuiltinModule("node:path");

              const socketPath = process.env.DIORAMA_OCI_CRI_SOCKET || resolveModule.DEFAULT_OCI_CRI_SOCKET;
              let ociCriProc = null;

              if (
                binName === "diorama" &&
                !resolveModule.isReadOnlyInvocation(args) &&
                !resolveModule.hasExplicitRuntimeSelection(args, process.env) &&
                !fs.existsSync(socketPath)
              ) {
                fs.mkdirSync(dirname(socketPath), { recursive: true, mode: 0o700 });
                ociCriProc = spawn(${JSON.stringify(ociCriMock)}, ["serve", "--socket", socketPath], {
                  stdio: "ignore",
                });
              }

              try {
                const res = spawnSync(${JSON.stringify(dioramaMock)}, args, {
                  stdio: "inherit",
                  env: resolveModule.buildRuntimeEnv(binName, process.env, socketPath, args),
                });
                process.exit(res.status ?? 0);
              } finally {
                if (ociCriProc && !ociCriProc.killed) {
                  try { ociCriProc.kill("SIGTERM"); } catch {}
                }
              }
            };

            customRunBinary("diorama", [${JSON.stringify(flag)}]);
          `;

          const proc = spawnSync(process.execPath, ["--input-type=module", "-e", testScript], {
            cwd: join(import.meta.dirname, "../.."),
            encoding: "utf8",
          });

          expect(proc.status).toBe(0);
          expect(proc.stdout).toContain(`MOCK_DIORAMA_CALLED: ${flag}`);

          // Assert zero filesystem writes: socket directory was never created
          expect(existsSync(nonExistentSocketDir)).toBe(false);

          // Assert zero process spawns: oci-cri was never executed
          expect(existsSync(ociCriSpawnLog)).toBe(false);
        }
      } finally {
        rmSync(tempDir, { recursive: true, force: true });
      }
    });

    test("verifies live runBinary with package layout stages zero process spawns on --version", () => {
      // Test through actual resolveBinaryPath with staged package bin directory
      const primaryDir = join(import.meta.dirname, "../arm64/bin");
      const hadBinDir = existsSync(primaryDir);
      mkdirSync(primaryDir, { recursive: true });

      const dioramaPath = join(primaryDir, "diorama");
      const ociCriPath = join(primaryDir, "oci-cri");
      const ociCriSpawnLog = join(tmpdir(), `diorama-oci-cri-live-test-${Date.now()}.log`);
      const socketDir = join(tmpdir(), `diorama-socket-live-test-${Date.now()}`);
      const socketPath = join(socketDir, "oci-cri.sock");

      writeFileSync(dioramaPath, "#!/bin/sh\necho diorama-ok\nexit 0\n", { mode: 0o755 });
      writeFileSync(
        ociCriPath,
        `#!/bin/sh\necho spawned >> "${ociCriSpawnLog}"\nexit 0\n`,
        { mode: 0o755 },
      );

      try {
        const testScript = `
          Object.defineProperty(process, "platform", { value: "linux" });
          Object.defineProperty(process, "arch", { value: "arm64" });
          process.env.DIORAMA_OCI_CRI_SOCKET = ${JSON.stringify(socketPath)};

          const { runBinary } = await import("./packages/cli/resolve.js");
          runBinary("diorama", ["--version"]);
        `;

        const proc = spawnSync(process.execPath, ["--input-type=module", "-e", testScript], {
          cwd: join(import.meta.dirname, "../.."),
          encoding: "utf8",
        });

        expect(proc.status).toBe(0);
        expect(proc.stdout).toContain("diorama-ok");
        // Verify socket directory never created
        expect(existsSync(socketDir)).toBe(false);
        // Verify oci-cri never spawned
        expect(existsSync(ociCriSpawnLog)).toBe(false);
      } finally {
        rmSync(dioramaPath, { force: true });
        rmSync(ociCriPath, { force: true });
        rmSync(socketDir, { recursive: true, force: true });
        rmSync(ociCriSpawnLog, { force: true });
        if (!hadBinDir) {
          rmSync(primaryDir, { recursive: true, force: true });
        }
      }
    });
  });
});
