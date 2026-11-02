import { join } from "node:path";

export const binDir = join(import.meta.dirname, "bin");
export const dioramaPath = join(binDir, "diorama");
export const dioramactlPath = join(binDir, "dioramactl");
export const ociCriPath = join(binDir, "oci-cri");
