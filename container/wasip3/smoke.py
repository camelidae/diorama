#!/usr/bin/env python3
"""Exercise registry.py's tags against a disposable Diorama node via kubectl."""
import json
import subprocess
import sys
import time

repository = sys.argv[1]
prefix = f"p3-{int(time.time())}"
for n, (tag, handler, policy) in enumerate([
    ("single", "wasmtime", "Always"),
    ("indexed", "wasmtime", "Always"),
    ("mixed", "wasmtime", "Always"),
    ("mixed", "", "IfNotPresent"),
    ("mixed", "wasmtime", "IfNotPresent"),
]):
    name = f"{prefix}-{n}"
    container = {
        "name": "probe", "image": f"{repository}:{tag}", "imagePullPolicy": policy,
        "resources": {"limits": {"memory": "128Mi", "cpu": "500m"}},
    }
    if handler:
        container.update(args=["hello", "from-pod"], env=[{"name": "P3_TEST", "value": "from-env"}])
    else:
        container.update(command=["/bin/sh", "-c", "echo native-ok"])
    spec = {"restartPolicy": "Never", "containers": [container]}
    if handler:
        spec["runtimeClassName"] = handler
    pod = {"apiVersion": "v1", "kind": "Pod", "metadata": {"name": name}, "spec": spec}
    subprocess.run(["kubectl", "apply", "-f", "-"], input=json.dumps(pod), text=True, check=True)
    subprocess.run(["kubectl", "wait", "--for=jsonpath={.status.phase}=Succeeded", f"pod/{name}", "--timeout=120s"], check=True)
    logs = subprocess.check_output(["kubectl", "logs", name], text=True)
    assert ("hello from WASI 0.3" if handler else "native-ok") in logs, logs
    if handler:
        assert "arg=from-pod" in logs and "env=from-env" in logs, logs
    print(f"PASS {tag} handler={handler!r} policy={policy}: {logs}")
    subprocess.run(["kubectl", "delete", "pod", name, "--wait=false"], check=True)
