export type BinaryName = "diorama" | "dioramactl" | "oci-cri";

export {
  buildRuntimeEnv,
  DEFAULT_CONTAINER_RUNTIME,
  DEFAULT_CONTAINER_RUNTIME_ENDPOINT,
  DEFAULT_OCI_CRI_SOCKET,
  hasExplicitRuntimeSelection,
  isReadOnlyInvocation,
  PLATFORM_PACKAGES,
  resolveBinaryPath,
  resolvePlatformPackages,
  runBinary,
} from "./resolve.js";
