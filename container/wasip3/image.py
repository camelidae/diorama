#!/usr/bin/env python3
"""Package a P3 component as a single-manifest wasip3/wasm32 OCI archive."""
import hashlib
import io
import json
import pathlib
import sys
import tarfile

wasm, output = map(pathlib.Path, sys.argv[1:3])
name = sys.argv[3] if len(sys.argv) > 3 else "localhost/diorama-p3-probe:dev"
blobs = {}

def blob(data, media):
    digest = hashlib.sha256(data).hexdigest()
    blobs["blobs/sha256/" + digest] = data
    return {"mediaType": media, "digest": "sha256:" + digest, "size": len(data)}

def encoded(value):
    return json.dumps(value, separators=(",", ":")).encode()

layer = io.BytesIO()
with tarfile.open(fileobj=layer, mode="w") as archive:
    data = wasm.read_bytes()
    entry = tarfile.TarInfo("probe.wasm")
    entry.mode = 0o444
    entry.size = len(data)
    archive.addfile(entry, io.BytesIO(data))
layer_desc = blob(layer.getvalue(), "application/vnd.oci.image.layer.v1.tar")
config = {
    "architecture": "wasm32", "os": "wasip3",
    "config": {"Entrypoint": ["/probe.wasm"], "WorkingDir": "/"},
    "rootfs": {"type": "layers", "diff_ids": [layer_desc["digest"]]},
}
manifest = {
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "config": blob(encoded(config), "application/vnd.oci.image.config.v1+json"),
    "layers": [layer_desc],
}
manifest_desc = blob(encoded(manifest), "application/vnd.oci.image.manifest.v1+json")
manifest_desc["annotations"] = {"org.opencontainers.image.ref.name": name, "io.containerd.image.name": name}
manifest_desc["platform"] = {"os": "wasip3", "architecture": "wasm32"}
blobs["index.json"] = encoded({"schemaVersion": 2, "manifests": [manifest_desc]})
blobs["oci-layout"] = encoded({"imageLayoutVersion": "1.0.0"})
with tarfile.open(output, "w") as archive:
    for path, data in blobs.items():
        entry = tarfile.TarInfo(path)
        entry.mode = 0o644
        entry.size = len(data)
        archive.addfile(entry, io.BytesIO(data))
