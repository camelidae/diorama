#!/usr/bin/env python3
"""Publish single, WASI-index, and mixed-index fixtures to a disposable HTTP registry.

Usage: registry.py probe.tar native.tar http://localhost:15000
The native archive must contain one Linux platform (podman save --format oci-archive).
"""
import json
import argparse
import tarfile
import urllib.request
import urllib.parse

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("probe", help="WASI probe OCI archive")
parser.add_argument("native", help="single-platform native OCI archive")
parser.add_argument("registry", help="disposable registry URL, e.g. http://localhost:15000")
args = parser.parse_args()
base = args.registry.rstrip('/')
repo = base + '/v2/p3-probe'

def request(url, data=None, method='GET', media='application/octet-stream'):
    return urllib.request.urlopen(urllib.request.Request(
        url, data=data, method=method, headers={'Content-Type': media}))

def publish(path):
    with tarfile.open(path) as archive:
        def read(name):
            return archive.extractfile(name).read()
        desc = json.loads(read('index.json'))['manifests'][0]
        # Unwrap an archive-level index if the exporter retained one.
        while 'index' in desc['mediaType']:
            desc = json.loads(read('blobs/sha256/' + desc['digest'].split(':')[1]))['manifests'][0]
        manifest = read('blobs/sha256/' + desc['digest'].split(':')[1])
        parsed = json.loads(manifest)
        for blob in [parsed['config'], *parsed['layers']]:
            data = read('blobs/sha256/' + blob['digest'].split(':')[1])
            location = request(repo + '/blobs/uploads/', b'', 'POST').headers['Location']
            location = urllib.parse.urljoin(base, location)
            request(location + '&digest=' + blob['digest'], data, 'PUT').close()
        request(repo + '/manifests/' + desc['digest'], manifest, 'PUT', desc['mediaType']).close()
        config = json.loads(read('blobs/sha256/' + parsed['config']['digest'].split(':')[1]))
        desc = {k: desc[k] for k in ('digest', 'size', 'mediaType')}
        desc['platform'] = {k: config[k] for k in ('os', 'architecture')}
        return desc, manifest

wasm, manifest = publish(args.probe)
native, _ = publish(args.native)
request(repo + '/manifests/single', manifest, 'PUT', wasm['mediaType']).close()
media = 'application/vnd.oci.image.index.v1+json'
for tag, manifests in [('indexed', [wasm]), ('mixed', [native, wasm])]:
    data = json.dumps({'schemaVersion': 2, 'mediaType': media, 'manifests': manifests}).encode()
    request(repo + '/manifests/' + tag, data, 'PUT', media).close()
print('Published p3-probe:single, :indexed, :mixed')
