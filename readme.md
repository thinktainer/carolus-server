[![Build Status](https://travis-ci.org/carolustv/carolus-server.svg?branch=master)](https://travis-ci.org/carolustv/carolus-server)

# Carolus

Currently only runs with nightly.

Quick start:

```bash
echo 'CAROLUS_MOVIES_PATH="/my/movies/path"' > .env
cargo run &
curl http://localhost:3000/api/movies
```

Currently only supports mp4 files.

## Database setup

- install diesel_cli for sqlite `cargo install diesel_cli --no-default-features --features-sqlite`
- set database path `export DATABASE_URL=/path/to/sqlite.db`
- set up / migrate database `diesel database setup`

## TLS support

A quick way to get started with using tls is included in the repo (taken
from ["Rocket"](https://github.com/SergioBenitez/Rocket/tree/master/examples/tls)
Compile with the `tls` flag and run the following:

```bash
(cd private && bash ./gen_cert.sh)
export ROCKET_TLS={certs="private/ca_cert.pem",key="private/ca_key.pem"}
cargo run --feature=tls &
```

## Windows

You may currently have some other issues running on Windows, but compling
with the `sqlite-bundle` feature can help if you compile issues related to
sqlite.

## License

This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at http://mozilla.org/MPL/2.0/.
