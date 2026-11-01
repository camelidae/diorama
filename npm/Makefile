# npm workspace build, verification, and multi-architecture publishing targets.

BUN ?= bun
PUBLISHER := $(BUN) run packages/publisher-cli/index.ts

ARCH ?= all
VERSION ?=
FROM_DIR ?=
TAG ?=
OTP ?=
COMPILE ?=

CLI_FLAGS := --arch $(ARCH) \
	$(if $(VERSION),--version $(VERSION),) \
	$(if $(FROM_DIR),--from-dir $(FROM_DIR),) \
	$(if $(TAG),--tag $(TAG),) \
	$(if $(OTP),--otp $(OTP),) \
	$(if $(COMPILE),--compile,)

.PHONY: help install typecheck test check version \
	build build-dry-run stage pack pack-dry-run \
	publish publish-dry-run clean

help:
	@printf '%s\n' \
		"Usage: make <target> [ARCH=all|amd64|arm64|aarch64] [VERSION=x.y.z]" \
		"" \
		"Verification targets:" \
		"  install          Install workspace dependencies with bun" \
		"  typecheck        Run TypeScript type checking" \
		"  test             Run bun test suite across packages/*" \
		"  check            Run typecheck and test" \
		"" \
		"Build & release targets:" \
		"  version          Sync VERSION across @diorama/cli and platform packages" \
		"  build            Stage or compile diorama and dioramactl into packages/<arch>/bin" \
		"  build-dry-run    Preview binary staging/compilation plan without writing files" \
		"  stage            Alias for build" \
		"  pack             Verify staged binaries and create .tgz tarballs" \
		"  pack-dry-run     Preview .tgz packaging plan" \
		"  publish          Publish @diorama/{amd64,arm64,aarch64} and @diorama/cli to npm" \
		"  publish-dry-run  Preview npm publish commands and payloads (--dry-run)" \
		"  clean            Remove staged binaries and .tgz archives"

install:
	$(BUN) install

typecheck:
	$(BUN) run typecheck

test:
	$(BUN) test

check: typecheck test

version:
	@if [ -z "$(VERSION)" ]; then \
		printf 'error: VERSION is required (e.g. make version VERSION=0.1.0)\n' >&2; \
		exit 1; \
	fi
	$(PUBLISHER) version $(VERSION)

build stage:
	$(PUBLISHER) build $(CLI_FLAGS)

build-dry-run:
	$(PUBLISHER) build $(CLI_FLAGS) --dry-run

pack:
	$(PUBLISHER) pack $(CLI_FLAGS)

pack-dry-run:
	$(PUBLISHER) pack $(CLI_FLAGS) --dry-run

publish:
	$(PUBLISHER) publish $(CLI_FLAGS)

publish-dry-run:
	$(PUBLISHER) publish $(CLI_FLAGS) --dry-run

clean:
	rm -rf packages/*/bin *.tgz packages/*/*.tgz
