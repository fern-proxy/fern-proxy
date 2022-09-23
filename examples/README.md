<!--
SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
SPDX-License-Identifier: Apache-2.0
-->

# Example

At the moment there is only one basic example, which showcases the data masking
feature with for a PostgreSQL datastore. Two masking strategies are available.

The data masking enabled in the `config.toml` configuration file applies a
shape-preserving caviar strategy on some columns, replacing each alphanumeric
character with a `*`, preserving therefore spacing, emptiness, and punctuation.  
Such masking strategy isn't ideal from a privacy perspective, as it leaks both
length and general aspect information on the original data.

A fixed-length caviar strategy, not leaking such information, can be enabled by
simply changing the `strategy` setting in the `config.toml` configuration file.

Columns subject to data masking are configurable in `config.toml` as well.


## Architecture

This example spawns three containers:

* a PostgreSQL server, our datastore,
* a PostgreSQL client, we will simply be using `psql`,
* a Fern proxy, sitting between the PostgreSQL server and `psql`.

```
        +------+       +------------+       +-----------------+
        | psql | <---> | fern-proxy | <---> | postgres-server |
        +------+       +------------+       +-----------------+
```


## Steps

1. (if you haven't already) Clone the Fern proxy Git repository:
    ```console
    $ git clone https://github.com/fern-proxy/fern-proxy.git
    ```

2. Change to the `examples/` directory, and run the example architecture:
    ```console
    $ cd fern-proxy/examples
    $ docker-compose up
    ```

3. From **another terminal** run (SSL/TLS not being supported at the moment):
    ```console
    $ docker exec -it postgres-client psql 'postgresql://root:testpassword@fern-proxy:30000/testdb?sslmode=disable'
    ```
    A regular `psql` prompt will appear.

4. Type any SQL query you would like. If looking for inspiration, you can try:
    ```console
    testdb# \dS+;
    
       Schema   |              Name               | Type  | Owner | Persistence | Access method |    Size    | Description
    ------------+---------------------------------+-------+-------+-------------+---------------+------------+-------------
     pg_catalog | **_*********                    | table | ****  | permanent   | ****          | 56 kB      |
     pg_catalog | **_**                           | table | ****  | permanent   | ****          | 40 kB      |
     pg_catalog | **_****                         | table | ****  | permanent   | ****          | 88 kB      |
     pg_catalog | **_******                       | table | ****  | permanent   | ****          | 72 kB      |
     pg_catalog | **_*******                      | table | ****  | permanent   | ****          | 8192 bytes |
     pg_catalog | **_*********                    | table | ****  | permanent   | ****          | 472 kB     |
     pg_catalog | **_****_*******                 | table | ****  | permanent   | ****          | 40 kB      |
     pg_catalog | **_******                       | table | ****  | permanent   | ****          | 48 kB      |
     pg_catalog | **_*********_*********_******** | view  | ****  | permanent   |               | 0 bytes    |
     pg_catalog | **_*********_**********         | view  | ****  | permanent   |               | 0 bytes    |
     pg_catalog | **_*******_******_********      | view  | ****  | permanent   |               | 0 bytes    |
     pg_catalog | **_****                         | table | ****  | permanent   | ****          | 48 kB      |
     [...]
    ```
5. Observe the data masking strategy in action on all columns defined in `config.toml`.

6. Feel free to comment/uncomment configuration directives in `config.toml`
   to observe other available behaviors.

7. Once done, dispose of this example by running:
    ```console
    $ docker-compose down --volumes
    ```
