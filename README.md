<!--
SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
SPDX-License-Identifier: Apache-2.0
-->

# Fern proxy

With the advent of Cloud Native applications, architecture patterns evolved
and emerged to take advantage of cloud computing, and build more scalable
and resilient systems. Many challenges remain to be addressed, data security
and privacy being one of them.

Fern proxy aims to fill the gap between applications and datastores, humbly
providing one of the missing pieces for modern architectures where business
logic is decoupled from specialized security and privacy components.


## Functionality Overview

Pursuing the principles of _{security,privacy}-by-{default,design}_ and
decoupling business logic from security and privacy operations, Fern proxy
sits between applications and datastores, providing off the shelf data
encryption, data masking, and data tokenization features.

Features provided by Fern proxy are configurable, to only apply one or
several transformations to the sole data requiring them. In a database context,
operating at this granularity means Fern proxy can support several kinds of
Row-level and Column-level security and privacy strategies.

For the time being, development in Fern proxy focuses on implementing core
features to support and provide value in a PostgreSQL datastore scenario.
Wire protocols and abstractions allowing other datastores will come later.
Same goes for deployment and optimizations leveraging DaemonSets and eBPF.

**Note: Fern proxy is definitely not ready yet for production.**


## Quickstart

If you are looking for a quick demo to see Fern proxy in action, you can
go directly to the [examples](examples/) directory!

Otherwise, Fern proxy follows the [_Gazr_](https://gazr.io) approach, to ease
evaluation, on-boarding, and development.

Most useful commands:

* `make run`: locally run Fern proxy, built in a _release_ mode
* `make watch`: locally run Fern proxy with hot reloading on code changes

For completeness, execute `make help` to display all available commands.


## Community and Contribution

Fern proxy is an ambitious project, and it can't reach its full potential
without a supporting community. Feeling curious or even adventurous? Please
have a look at our [Community and Contribution Guidelines](CONTRIBUTING.md),
as well as our [Code of Conduct](CODE_OF_CONDUCT.md)!


## Security

We believe it is never _too early_ to address security in a new project -
especially when the project is a security and privacy component like Fern.

While Fern proxy is being designed and implemented with high security standards,
nobody is perfect and mistakes can happen. If you think you found a security
issue, let us know through our [vulnerability reporting process](SECURITY.md).


## License

Fern proxy is licensed under the [Apache License, Version 2.0](LICENSE).
