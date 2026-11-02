import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  chmodSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import * as net from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  prepareCriSocketDirectory,
  runBinary,
  validateCriSocketPath,
} from "./resolve.js";

describe("@diorama/cli CRI Unix socket security validations", () => {
  let testRoot: string;
  const currentUid =
    typeof process.getuid === "function" ? process.getuid() : 0;

  beforeEach(() => {
    testRoot = mkdtempSync(
      join(realpathSync(tmpdir()), "diorama-socket-sec-test-"),
    );
    chmodSync(testRoot, 0o700);
  });

  afterEach(() => {
    try {
      rmSync(testRoot, { recursive: true, force: true });
    } catch {
      // Ignore cleanup failures
    }
  });

  describe("validateCriSocketPath", () => {
    test("rejects invalid, empty, or relative paths", () => {
      // @ts-expect-error test invalid types
      expect(() => validateCriSocketPath(null)).toThrow(
        /non-empty string/,
      );
      // @ts-expect-error test invalid types
      expect(() => validateCriSocketPath(undefined)).toThrow(
        /non-empty string/,
      );
      expect(() => validateCriSocketPath("")).toThrow(
        /non-empty string/,
      );
      expect(() => validateCriSocketPath("   ")).toThrow(
        /non-empty string/,
      );

      expect(() => validateCriSocketPath("cri.sock")).toThrow(
        /absolute path/,
      );
      expect(() => validateCriSocketPath("./cri.sock")).toThrow(
        /absolute path/,
      );
      expect(() => validateCriSocketPath("../var/run/cri.sock")).toThrow(
        /absolute path/,
      );
      expect(() => validateCriSocketPath("relative/path/to/socket.sock")).toThrow(
        /absolute path/,
      );
    });

    test("rejects paths located in world-writable temporary directories", () => {
      const worldWritableCandidates = [
        "/tmp/cri.sock",
        "/tmp/sub/cri.sock",
        "/tmp",
        "/var/tmp/cri.sock",
        "/var/tmp/oci-cri/socket.sock",
        "/dev/shm/cri.sock",
        "/dev/shm/runtime/cri.sock",
        "/private/tmp/cri.sock",
        "/private/var/tmp/cri.sock",
      ];

      for (const socketPath of worldWritableCandidates) {
        expect(() => validateCriSocketPath(socketPath)).toThrow(
          /world-writable directory/,
        );
      }
    });

    test("rejects socket path when the socket file is a symlink", () => {
      const realTarget = join(testRoot, "target-file");
      writeFileSync(realTarget, "dummy");
      const symlinkSocket = join(testRoot, "symlink-sock");
      symlinkSync(realTarget, symlinkSocket);

      expect(lstatSync(symlinkSocket).isSymbolicLink()).toBe(true);
      expect(() => validateCriSocketPath(symlinkSocket)).toThrow(
        /cannot be a symlink/,
      );
    });

    test("rejects socket path when parent directory component is a symlink", () => {
      const realDir = join(testRoot, "real-parent-dir");
      mkdirSync(realDir, { mode: 0o700 });
      const symlinkDir = join(testRoot, "symlink-parent-dir");
      symlinkSync(realDir, symlinkDir);

      const socketInSymlinkDir = join(symlinkDir, "cri.sock");
      expect(() => validateCriSocketPath(socketInSymlinkDir)).toThrow(
        /component cannot be a symlink/,
      );
    });

    test("rejects socket path when ancestor directory component is a symlink", () => {
      const realAncestor = join(testRoot, "real-ancestor");
      mkdirSync(realAncestor, { mode: 0o700 });
      const symlinkAncestor = join(testRoot, "symlink-ancestor");
      symlinkSync(realAncestor, symlinkAncestor);

      const deepSocketPath = join(symlinkAncestor, "child", "cri.sock");
      expect(() => validateCriSocketPath(deepSocketPath)).toThrow(
        /component cannot be a symlink/,
      );
    });

    test("accepts valid absolute paths under secure directories", () => {
      const validSocketPath = join(testRoot, "oci-cri", "cri.sock");
      expect(() => validateCriSocketPath(validSocketPath)).not.toThrow();
    });
  });

  describe("prepareCriSocketDirectory: parent directory permissions and ownership", () => {
    test("creates parent directory with mode 0700 when it does not exist", () => {
      const socketDir = join(testRoot, "new-runtime-dir");
      const socketPath = join(socketDir, "cri.sock");

      prepareCriSocketDirectory(socketPath);

      const stat = lstatSync(socketDir);
      expect(stat.isDirectory()).toBe(true);
      expect(stat.mode & 0o777).toBe(0o700);
      expect(stat.mode & 0o077).toBe(0);
    });

    test("hardens existing parent directory permissions from 0755 to 0700", () => {
      const socketDir = join(testRoot, "existing-0755-dir");
      mkdirSync(socketDir, { mode: 0o755 });
      chmodSync(socketDir, 0o755);
      const socketPath = join(socketDir, "cri.sock");

      prepareCriSocketDirectory(socketPath);

      const stat = lstatSync(socketDir);
      expect(stat.mode & 0o777).toBe(0o700);
      expect(stat.mode & 0o077).toBe(0);
    });

    test("rejects parent directory with group-write or world-write permissions", () => {
      const worldWritableDir = join(testRoot, "world-writable-dir");
      mkdirSync(worldWritableDir, { mode: 0o777 });
      chmodSync(worldWritableDir, 0o777);
      expect(() =>
        prepareCriSocketDirectory(join(worldWritableDir, "cri.sock")),
      ).toThrow(/group\/other write permissions|world-writable/);

      const groupWritableDir = join(testRoot, "group-writable-dir");
      mkdirSync(groupWritableDir, { mode: 0o770 });
      chmodSync(groupWritableDir, 0o770);
      expect(() =>
        prepareCriSocketDirectory(join(groupWritableDir, "cri.sock")),
      ).toThrow(/group\/other write permissions/);

      const groupWriteOnlyDir = join(testRoot, "group-write-only-dir");
      mkdirSync(groupWriteOnlyDir, { mode: 0o775 });
      chmodSync(groupWriteOnlyDir, 0o775);
      expect(() =>
        prepareCriSocketDirectory(join(groupWriteOnlyDir, "cri.sock")),
      ).toThrow(/group\/other write permissions/);
    });

    test("rejects parent directory with other-write permissions (e.g. 0757)", () => {
      const otherWritableDir = join(testRoot, "other-writable-dir");
      mkdirSync(otherWritableDir, { mode: 0o757 });
      chmodSync(otherWritableDir, 0o757);
      expect(() =>
        prepareCriSocketDirectory(join(otherWritableDir, "cri.sock")),
      ).toThrow(/group\/other write permissions|world-writable/);
    });

    test("rejects parent directory when it is a symlink", () => {
      const realDir = join(testRoot, "real-dir");
      mkdirSync(realDir, { mode: 0o700 });
      const symlinkDir = join(testRoot, "symlink-dir");
      symlinkSync(realDir, symlinkDir);

      expect(() =>
        prepareCriSocketDirectory(join(symlinkDir, "cri.sock")),
      ).toThrow(/cannot be a symlink/);
    });

    test("verifies parent directory ownership against expectedUid", () => {
      const socketDir = join(testRoot, "ownership-dir");
      mkdirSync(socketDir, { mode: 0o700 });
      const socketPath = join(socketDir, "cri.sock");

      // Matching currentUid passes
      expect(() =>
        prepareCriSocketDirectory(socketPath, { expectedUid: currentUid }),
      ).not.toThrow();

      // Mismatched expectedUid (non-matching and non-zero) fails
      expect(() =>
        prepareCriSocketDirectory(socketPath, { expectedUid: 65432 }),
      ).toThrow(/Insecure CRI socket parent directory ownership/);

      // When running as non-root, expecting root UID 0 fails if dir is owned by current non-root user
      if (currentUid !== 0) {
        expect(() =>
          prepareCriSocketDirectory(socketPath, { expectedUid: 0 }),
        ).toThrow(/Insecure CRI socket parent directory ownership/);
      }
    });

    test("prepareCriSocketDirectory rejects relative paths and /tmp directly", () => {
      expect(() => prepareCriSocketDirectory("relative/cri.sock")).toThrow(
        /absolute path/,
      );
      expect(() => prepareCriSocketDirectory("/tmp/cri.sock")).toThrow(
        /world-writable directory/,
      );
    });
  });

  describe("prepareCriSocketDirectory: socket file validation and mode 0600 enforcement", () => {
    test("enforces mode 0600 on an existing socket file", () => {
      const socketDir = join(testRoot, "socket-file-dir");
      mkdirSync(socketDir, { mode: 0o700 });
      const socketPath = join(socketDir, "cri.sock");

      // Create a file simulating a socket or dummy file with permissive mode 0666
      writeFileSync(socketPath, "");
      chmodSync(socketPath, 0o666);
      expect(lstatSync(socketPath).mode & 0o777).toBe(0o666);

      prepareCriSocketDirectory(socketPath);

      const socketStat = lstatSync(socketPath);
      expect(socketStat.mode & 0o777).toBe(0o600);
      expect(socketStat.mode & 0o077).toBe(0);
    });

    test("enforces mode 0600 on active listening Unix domain socket", async () => {
      const socketDir = join(testRoot, "active-socket-dir");
      mkdirSync(socketDir, { mode: 0o700 });
      const socketPath = join(socketDir, "active.sock");

      await new Promise<void>((resolve, reject) => {
        const server = net.createServer();
        server.listen(socketPath, () => {
          try {
            // Verify and enforce mode 0600 on the live socket
            prepareCriSocketDirectory(socketPath);
            const socketStat = lstatSync(socketPath);
            expect(socketStat.mode & 0o777).toBe(0o600);
            expect(socketStat.mode & 0o077).toBe(0);
            server.close(() => resolve());
          } catch (err) {
            server.close(() => reject(err));
          }
        });
        server.on("error", reject);
      });
    });

    test("rejects existing socket file when it is a symlink", () => {
      const socketDir = join(testRoot, "symlink-sock-dir");
      mkdirSync(socketDir, { mode: 0o700 });
      const realTarget = join(testRoot, "real-sock-target");
      writeFileSync(realTarget, "");
      const symlinkSocket = join(socketDir, "cri.sock");
      symlinkSync(realTarget, symlinkSocket);

      expect(() => prepareCriSocketDirectory(symlinkSocket)).toThrow(
        /cannot be a symlink/,
      );
    });

    test("rejects existing socket file when owned by unexpected UID", () => {
      const socketDir = join(testRoot, "uid-sock-dir");
      mkdirSync(socketDir, { mode: 0o700 });
      const socketPath = join(socketDir, "cri.sock");
      writeFileSync(socketPath, "");

      expect(() =>
        prepareCriSocketDirectory(socketPath, {
          expectedUid: currentUid,
          expectedSocketUid: 65431,
        }),
      ).toThrow(/Insecure CRI socket file ownership/);
    });
  });

  describe("runBinary integration with socket security checks", () => {
    test("rejects relative socket path in DIORAMA_OCI_CRI_SOCKET", () => {
      const oldEnv = process.env.DIORAMA_OCI_CRI_SOCKET;
      process.env.DIORAMA_OCI_CRI_SOCKET = "relative/path/socket.sock";
      try {
        expect(() => runBinary("diorama", [])).toThrow(/absolute path/);
      } finally {
        if (oldEnv === undefined) {
          delete process.env.DIORAMA_OCI_CRI_SOCKET;
        } else {
          process.env.DIORAMA_OCI_CRI_SOCKET = oldEnv;
        }
      }
    });

    test("rejects world-writable temporary directory in DIORAMA_OCI_CRI_SOCKET", () => {
      const oldEnv = process.env.DIORAMA_OCI_CRI_SOCKET;
      process.env.DIORAMA_OCI_CRI_SOCKET = "/tmp/malicious.sock";
      try {
        expect(() => runBinary("diorama", [])).toThrow(
          /world-writable directory/,
        );
      } finally {
        if (oldEnv === undefined) {
          delete process.env.DIORAMA_OCI_CRI_SOCKET;
        } else {
          process.env.DIORAMA_OCI_CRI_SOCKET = oldEnv;
        }
      }
    });

    test("skips socket preparation and validation on read-only invocations", () => {
      const oldEnv = process.env.DIORAMA_OCI_CRI_SOCKET;
      process.env.DIORAMA_OCI_CRI_SOCKET = "/tmp/should-be-ignored-on-readonly.sock";
      try {
        // Does not throw world-writable error; proceeds to binary resolution
        expect(() => runBinary("diorama", ["--version"])).toThrow(
          /Unsupported platform/,
        );
      } finally {
        if (oldEnv === undefined) {
          delete process.env.DIORAMA_OCI_CRI_SOCKET;
        } else {
          process.env.DIORAMA_OCI_CRI_SOCKET = oldEnv;
        }
      }
    });

    test("skips socket preparation when explicit container runtime is specified", () => {
      const oldEnv = process.env.DIORAMA_OCI_CRI_SOCKET;
      process.env.DIORAMA_OCI_CRI_SOCKET = "/tmp/should-be-ignored-on-explicit-runtime.sock";
      try {
        expect(() =>
          runBinary("diorama", ["--container-runtime=containerd"]),
        ).toThrow(/Unsupported platform/);
      } finally {
        if (oldEnv === undefined) {
          delete process.env.DIORAMA_OCI_CRI_SOCKET;
        } else {
          process.env.DIORAMA_OCI_CRI_SOCKET = oldEnv;
        }
      }
    });

    test("prepares socket directory with mode 0700 when runBinary initializes CRI daemon", () => {
      const oldEnv = process.env.DIORAMA_OCI_CRI_SOCKET;
      const targetDir = join(testRoot, "runbinary-cri-dir");
      const socketPath = join(targetDir, "oci-cri.sock");
      process.env.DIORAMA_OCI_CRI_SOCKET = socketPath;
      try {
        // Will attempt to prepare socket dir, then fail on resolveBinaryPath or ociCriBin on darwin
        try {
          runBinary("diorama", []);
        } catch {
          // Expected on unsupported host platform darwin
        }
        const stat = lstatSync(targetDir);
        expect(stat.isDirectory()).toBe(true);
        expect(stat.mode & 0o777).toBe(0o700);
      } finally {
        if (oldEnv === undefined) {
          delete process.env.DIORAMA_OCI_CRI_SOCKET;
        } else {
          process.env.DIORAMA_OCI_CRI_SOCKET = oldEnv;
        }
      }
    });
  });
});
