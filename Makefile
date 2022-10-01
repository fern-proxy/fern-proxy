# SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
# SPDX-License-Identifier: Apache-2.0

PROJECT := fern-proxy

COMPOSE = docker-compose -p $(PROJECT)
DOCKER  = DOCKER_BUILDKIT=1 docker


SHELL = /bin/sh
CURRENT_UID := $(shell id -u)
CURRENT_GID := $(shell id -g)


.DEFAULT_GOAL := help


.PHONY: build
build: build-release

.PHONY: build-nightly
build-nightly: clean
	@echo "\033[103m\033[30m\033[1m🍳 $@ \033[0m"
	$(DOCKER) build -f Dockerfile \
		--target 'release-env' --tag '${PROJECT}:nightly' .

.PHONY: build-release
build-release: clean
	@echo "\033[103m\033[30m\033[1m🍳 $@ \033[0m"
	$(DOCKER) build -f Dockerfile \
		--target 'release-env' --tag '${PROJECT}:latest' .

.PHONY: build-dev
build-dev: clean
	@echo "\033[103m\033[30m\033[1m🍳 $@ \033[0m"
	$(DOCKER) build -f Dockerfile \
		--target 'dev-env' --tag '${PROJECT}:dev' .


.PHONY: publish-nightly
publish-nightly: build-nightly
	@echo "\033[103m\033[30m\033[1m🚀 $@ \033[0m"
	@echo "Not implemented yet..."
#   docker push, etc.

.PHONY: publish-release
publish-release: build-release
	@echo "\033[103m\033[30m\033[1m🚀 $@ \033[0m"
	@echo "Not implemented yet..."
#   cargo release, docker push, etc.


.PHONY: clean clean-all
clean: clean-files
clean-all: clean-containers clean-files ## Launch all cleaning tasks (containers, temp files)

.PHONY: clean-containers
clean-containers: ## Stop and remove produced containers, networks, and volumes
	@echo "\033[47m\033[30m\033[1m🧹 $@ \033[0m"
	$(COMPOSE) down --volumes

.PHONY: clean-files
clean-files:
	@echo "\033[47m\033[30m\033[1m🧹 $@ \033[0m"
	find . \( -name '*~' -o -name '*.profraw' \) -exec rm --force  {} +

.PHONY: doc
doc: build-dev ## Generate and serve developer documentation (cargo doc)
	@echo "\033[47m\033[34m\033[1m📚 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    '${PROJECT}:dev'                                                   \
	    cargo doc --no-deps                                                \
	    && cd target/doc                                                   \
	    && python3 -m http.server 8080

.PHONY: test
test: test-unit ## Launch all testing tasks (unit, functional and integration)

.PHONY: test-unit
test-unit: build-dev  ## Launch unit tests (cargo test)
	@echo "\033[100m\033[92m\033[1m🧪 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    --mount type=bind,target='/app/',src='${CURDIR}'                   \
	    --user '${CURRENT_UID}:${CURRENT_GID}'                             \
	    '${PROJECT}:dev'                                                   \
	    cargo test --no-fail-fast

.PHONY: test-integration
test-integration: build-dev  ## Launch integration tests (cargo test --tests)
	@echo "\033[100m\033[92m\033[1m🧪 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    --mount type=bind,target='/app/',src='${CURDIR}'                   \
	    --user '${CURRENT_UID}:${CURRENT_GID}'                             \
	    '${PROJECT}:dev'                                                   \
	    cargo test --tests --no-fail-fast

.PHONY: format
format: format-code ## Format the code (rustfmt)

.PHONY: check-all check-format
check-all: security style
check-format: check-format-code ## Check the code format (rustfmt)

.PHONY: format-code
format-code: build-dev
	@echo "\033[104m\033[30m\033[1m✒️  $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    --mount type=bind,target='/app',src='${CURDIR}'                    \
	    --user '${CURRENT_UID}:${CURRENT_GID}'                             \
	    '${PROJECT}:dev'                                                   \
	    cargo fmt

.PHONY: check-format-code
check-format-code: build-dev
	@echo "\033[47m\033[34m\033[1m🛂 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    '${PROJECT}:dev'                                                   \
	    cargo fmt -- --check

.PHONY: style
style: build-dev check-format ## Check lint, code style rules (clippy, rustfmt)
	@echo "\033[47m\033[34m\033[1m🛂 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    '${PROJECT}:dev'                                                   \
	    cargo clippy -- -A clippy::complexity

.PHONY: complexity
complexity: ## Cyclomatic complexity check (clippy)
	@echo "\033[47m\033[34m\033[1m🛂 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    '${PROJECT}:dev'                                                   \
	    cargo clippy -- -W clippy::cognitive_complexity

.PHONY: security
security: security-sca ## Launch all security tasks (SCA)

.PHONY: security-sca
security-sca: build-dev ## Launch Software Composition Analysis (cargo-audit)
	@echo "\033[101m\033[30m\033[1m🛡️  $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    '${PROJECT}:dev'                                                   \
	    cargo audit

.PHONY: coverage
coverage: build-dev ## Measure code coverage (source-based grcov)
	@echo "\033[101m\033[30m\033[1m📐 $@ \033[0m"
	$(DOCKER) run --rm                                                     \
	    --env CARGO_INCREMENTAL=0                                          \
	    --env RUSTFLAGS='-Cinstrument-coverage -Clink-dead-code'           \
	    --env LLVM_PROFILE_FILE='profiler-%p-%m.profraw'                   \
	    --env RUST_LOG='trace'                                             \
	    --mount type=bind,target='/coverage',src='${CURDIR}'/.coverage     \
	    '${PROJECT}:dev'                                                   \
	    bash -c '                                                          \
	      cargo test --tests --no-fail-fast                                \
	      && for format in lcov html; do                                   \
	         grcov .                                                       \
	           --binary-path ./target/debug/deps/                          \
	           --source-dir .                                              \
	           --branch                                                    \
	           --ignore-not-existing                                       \
	           --ignore "../*"                                             \
	           --ignore "/*"                                               \
	           --output-type $${format}                                    \
	           --output-path /coverage/$${format}                          \
	         ; done                                                        \
	      && chown -R ${CURRENT_UID}:${CURRENT_GID} /coverage              \
	    '


.PHONY: run
run: run-release ## Locally run the application (release stack)

.PHONY: run-release
run-release: build-release
	@echo "\033[100m\033[37m\033[1m🏗️️  $@ \033[0m"
	$(COMPOSE) up

.PHONY: run-dev
run-dev: build-dev ## Locally run the application (dev stack)
	@echo "\033[100m\033[37m\033[1m🏗️️  $@ \033[0m"
	$(COMPOSE)                                                             \
	    -f docker-compose.yml                                              \
	    -f docker-compose.override.dev.yml                                 \
	    up

.PHONY: watch
watch: watch-dev ## Run application with hot reloading (dev stack)

.PHONY: watch-dev
watch-dev: build-dev
	@echo "\033[100m\033[96m\033[1m💡️️ $@ \033[0m"
	env CURRENT_USER=${CURRENT_UID}:${CURRENT_GID} $(COMPOSE)              \
	    -f docker-compose.yml                                              \
	    -f docker-compose.override.repl.yml                                \
	    -f docker-compose.override.dev.yml                                 \
	    up

# Implements this pattern for autodocumenting Makefiles:
# https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
# Picks up all comments that:
# - start with a `##`,
# - and are the end of a target definition line.
.PHONY: help
help:
	@grep -E '^[0-9a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST)                 \
	  | sort                                                               \
	  | awk 'BEGIN {FS = ":.*?## "}; \
	      {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
