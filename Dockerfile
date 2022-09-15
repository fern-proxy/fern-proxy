# SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
# SPDX-License-Identifier: Apache-2.0

FROM rust:1.63 AS build-env

WORKDIR /app
COPY . .
RUN cargo build --release


# Google provided Distroless base image
FROM gcr.io/distroless/cc AS release-env

COPY --from=build-env /app/target/release/fern-proxy /app/fern-proxy
WORKDIR /app
CMD [ "./fern-proxy" ]


FROM rust:1.63-slim AS dev-env

# Required for standard code formatting
RUN rustup component add rustfmt

# Required for linting (style, complexity, ...)
RUN rustup component add clippy

# Required for code coverage
RUN cargo install grcov

# Required for REPL
RUN cargo install cargo-watch

# Required for SCA
RUN cargo install cargo-audit

# Super dirty hack to allow and speed-up builds in REPL mode (`make watch`)
RUN chmod -R o+rwx /usr/local/cargo/

# Most moving parts at the end, even if BuildKit is parallelizing the steps
WORKDIR /app
COPY . .
CMD [ "cargo", "run" ]
