import { join } from "node:path";

export const binDir = join(import.meta.dirname, "bin");
export const dioramaPath = join(binDir, "diorama");
export const dioramactlPath = join(binDir, "dioramactl");
