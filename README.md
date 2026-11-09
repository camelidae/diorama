# diorama

diorama brings Kubernetes cluster services, workload execution, networking, and
storage into a Rust runtime. It includes an API server, controllers, an embedded
datastore, and kubelet functionality, with CRI integration for container engines.
The companion `dioramactl` CLI manages installation, offline bundles, and upgrades.

Start on one Linux host or explore controller and worker joins across multiple
nodes. The project is under active development; see [project scope](docs/scope.md)
for current limits and [Kubernetes conformance](docs/conformance.md) for the
release-specific requirements checklist.

## Get started

[Install a release](docs/installation.md) using prebuilt Linux binaries and offline
bundles. The guide covers amd64 and arm64, container installation, host extraction,
and options for macOS users. Git and Rust are only needed for
[source development](docs/development.md).

## Documentation

| Guide | Contents |
| --- | --- |
| [Installation](docs/installation.md) | Download, install, and connect to a node |
| [Architecture](docs/architecture.md) | Components and controller/worker roles |
| [Development](docs/development.md) | Build, configure, and run from source |
| [Multi-node development](docs/multi-node.md) | Two-controller setup and worker joins |
| [Kubernetes conformance](docs/conformance.md) | Official Kubernetes 1.35 requirements checklist |
| [Project scope](docs/scope.md) | Deployment limits and platform targets |
| [Resources](docs/resources.md) | Component documentation and repository tooling |

## Contributing

See the [Contributor Guide](CONTRIBUTING.md). Use
[GitHub issues](https://github.com/geoffsee/diorama/issues) for bugs, questions,
and proposals, and [pull requests](https://github.com/geoffsee/diorama/pulls) for
contributions.

## License and conduct

Licensed under the [ISC License](LICENSE). Third-party code retains its own terms;
see, for example, the [saphyr-parser license](vendor/saphyr-parser/LICENSE).
This project adopts the [CNCF Code of Conduct](CODE_OF_CONDUCT.md).
