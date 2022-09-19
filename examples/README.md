<!--
SPDX-FileCopyrightText:  Copyright © 2022 The Fern Authors <team@fernproxy.io>
SPDX-License-Identifier: Apache-2.0
-->

# Example

At the moment there is only one basic example, which showcases a _dummy_ data
masking feature for a PostgreSQL datastore. While simple in appearance, a full
Query/Response cycle is demonstrated, will **all** data rows being processed.

The dummy data masking transformation is replacing every 'o' with an '*'.
It is kind of a worse case scenario as the masking is applied to _every_ single
field in _each_ row, without filtering on column names by choice, parsing every
byte of every DataRow returned as response, etc. Really terrible design. :-)


## Architecture

This example spawns three containers:

* a PostgreSQL server, our datastore
* a PostgreSQL client, we will simply be using `psql`
* a Fern proxy, sitting between the PostgreSQL server and `psql`

```
        +-------+       +------------+       +-----------------+
        | psqgl | <---> | fern-proxy | <---> | postgres-server |
        +-------+       +------------+       +-----------------+
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
     pg_catal*g | pg_aggregate                    | table | r**t  | permanent   | heap          | 56 kB      |
     pg_catal*g | pg_am                           | table | r**t  | permanent   | heap          | 40 kB      |
     pg_catal*g | pg_am*p                         | table | r**t  | permanent   | heap          | 88 kB      |
     pg_catal*g | pg_ampr*c                       | table | r**t  | permanent   | heap          | 72 kB      |
     pg_catal*g | pg_attrdef                      | table | r**t  | permanent   | heap          | 8192 bytes |
     pg_catal*g | pg_attribute                    | table | r**t  | permanent   | heap          | 472 kB     |
     [...]
    ```
5. Observe the _dummy_ data masking in action on all `o` characters.

6. Once done, dispose of this example by running:
    ```console
    $ docker-compose down --volumes
    ```
