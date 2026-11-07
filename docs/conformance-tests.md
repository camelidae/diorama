# Running conformance tests

There are three different test paths in this repository. Choose the one that
matches what you want to check:

| Path | Runs against | Purpose |
| --- | --- | --- |
| [Local regression tests](#local-regression-tests) | In-memory services and temporary API servers | Check implementation changes without a node container |
| [Live-node probes](#live-node-probes) | A fresh diorama container | Exercise client and workload behavior end to end |
| [Official Kubernetes suite](#official-kubernetes-suite) | An existing cluster | Run the upstream conformance requirements through Sonobuoy or Hydrophone |

The [conformance checklist](conformance.md) targets Kubernetes 1.35. The
repository also contains a [catalog printer](#upstream-catalog-printer) labelled
1.37.1; it does not execute the official suite.

## Local regression tests

From the repository root, with the [development toolchain](development.md):

```sh
cargo test --locked -p diorama-conformance --test suite -- --nocapture
cargo test --locked -p diorama-apiserver --test suite conformance -- --nocapture
```

The first command runs the two non-ignored client regression tests for Events,
server dry-run, and workload operations against an in-memory HTTP server. These
can print a skip message if client construction fails; check the output. The
second selects the API server's `conformance_c08` and `wave7a_conformance` modules,
which use temporary state and local API servers.

Neither command starts the live-node probes. To list all tests in the dedicated
conformance crate without executing them:

```sh
cargo test --locked -p diorama-conformance --test suite -- --list
```

## Live-node probes

Use a Linux environment with the development toolchain and Podman able to launch
privileged containers. The node needs access to the workload images used by the
probes, including `busybox:1.36`. Build and run from the repository root:

```sh
make CONTAINER_ENGINE=podman image
make CONTAINER_ENGINE=podman conformance
```

To build inside a container, substitute `make CONTAINER_ENGINE=podman image-source`
for the first command. To test another image already present in the same engine:

```sh
make CONTAINER_ENGINE=podman conformance IMAGE=localhost/diorama:my-build
```

The equivalent filtered Cargo command is:

```sh
CONTAINER_ENGINE=podman DIORAMA_IMAGE=localhost/diorama:latest \
  cargo test --locked -p diorama-conformance --test suite \
  client_gaps::client_gaps -- --exact --ignored --nocapture
```

The [probe harness](../crates/diorama-conformance/tests/client_gaps.rs) creates
`diorama-conformance-<process-id>`, publishes an available local API port, reads its
kubeconfig, and waits for the API. You do not need to start a node or supply a
kubeconfig beforehand. It tests discovery, resource operations, controllers,
volumes, logs, and exec, then prints a PASS/FAIL table and totals. A failed probe
makes the command fail. Setup failures stop the run before the table is printed.

The harness attempts to remove its container when the test exits. An interrupted
process may leave that container behind; remove only that test container after
collecting any logs you need. The probes are diorama-specific and do not replace
the official Kubernetes suite.

## Upstream catalog printer

```sh
cargo test --locked -p diorama-conformance --test suite \
  client_gaps::upstream_conformance_catalog -- --exact --ignored --nocapture
```

This reads [conformance.yaml](../crates/diorama-conformance/tests/fixtures/conformance.yaml),
groups its 462 entries by SIG, prints every entry as `not covered`, and
**intentionally fails**. The source identifies this catalog as Kubernetes 1.37.1.
It does not start a cluster or measure diorama's support for those requirements.

Keep the exact filters above: an unfiltered `--ignored` run selects both the
live probes and this deliberately failing catalog printer. Do not use the 1.37.1
catalog to score the 1.35 checklist.

## Official Kubernetes suite

CNCF documents [Sonobuoy and Hydrophone as runners](https://github.com/cncf/k8s-conformance/blob/master/README.md#run-the-tests)
for the upstream suite. The following uses its
[Sonobuoy instructions](https://github.com/cncf/k8s-conformance/blob/master/instructions.md#running).

### Prepare a cluster

Use a dedicated test cluster: certified-conformance mode includes disruptive
tests and may take hours. Install `kubectl` and a
[Sonobuoy CLI release](https://github.com/vmware-tanzu/sonobuoy/releases), and obtain
an administrator kubeconfig using the [installation guide](installation.md#connect-to-the-cluster).
The nodes must be able to run the runner and workload images; see
[Sonobuoy's image configuration guidance](https://github.com/vmware-tanzu/sonobuoy#troubleshooting)
if using a mirror.

```sh
export KUBECONFIG="$PWD/kubeconfig.yaml"
kubectl config current-context
kubectl version
kubectl get nodes
```

Confirm the context and server version before proceeding. Use the official suite
for that Kubernetes version; the checklist on this branch covers 1.35. Record the
diorama build, server version, runner version, and selected test image with the
results. The local catalog's version is not a reason to select a different suite.

### Run and collect results

```sh
sonobuoy run --mode=certified-conformance --wait
```

In another terminal using the same kubeconfig:

```sh
sonobuoy status
sonobuoy logs
```

Once the run is complete, retrieve the archive before cleanup. Run the following
in a directory for this run's results:

```sh
(
  set -eu
  archive=$(sonobuoy retrieve)
  mkdir results
  tar -xzf "$archive" -C results
)
```

Inspect `results/plugins/e2e/results/global/e2e.log` and `junit_01.xml` for test
outcomes. A completed runner is not necessarily a passing run. Then remove the
runner's cluster resources:

```sh
sonobuoy delete --wait
```

For certification, run all conformance tests without skip filters. CNCF submission
also requires `PRODUCT.yaml` and reproduction instructions; follow the
[submission guide](https://github.com/cncf/k8s-conformance/blob/master/instructions.md#contents-of-the-pr).
A passing local probe suite or catalog listing is not a certification result.

### Hydrophone alternative

[Hydrophone](https://github.com/kubernetes-sigs/hydrophone#running-tests) runs the
upstream conformance image directly. Its guide covers `--conformance`, kubeconfig
selection, and selecting a versioned conformance image. Use it as an alternative
to Sonobuoy, following the same Kubernetes-version and full-suite requirements.

[Back to conformance checklist](conformance.md) · [Back to documentation](../README.md#documentation)
