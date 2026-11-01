export type BinaryName = "diorama" | "dioramactl";

export {
  buildRuntimeEnv,
  DEFAULT_CONTAINER_RUNTIME,
  PLATFORM_PACKAGES,
  resolveBinaryPath,
  resolvePlatformPackages,
  runBinary,
} from "./resolve.js";
