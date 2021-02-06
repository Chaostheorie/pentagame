# pentagame-test

## Building

> Only GNU/Linux distributions are supported at the moment

You need following free software for building:

-   [GNU Make](https://www.gnu.org/software/make/)
-   [GNU Bash](https://www.gnu.org/software/bash/)
-   [cargo (rustup)](https://rustup.rs/)
-   [diesel cli](https://diesel.rs/guides/getting-started/) _with features postgres_
-   [npm](https://www.npmjs.com/get-npm)
-   [git](https://git-scm.com/)

Only a working [PostgreSQL server](https://www.postgresql.org/) is required on runtime since the application relies on static compilation and embeds all assets in the binary on compilation time.

### Instructions

Build is broken ATM. I have seen spikes of over 7 GB RAM usage while building. Please ensure you have _at least 8 GB of RAM_ when building.

#### With optimizations

1. Clone and enter repository from Github: `https://github.com/Chaostheorie/pentagame && cd pentagame`
2. Build (This will take quite some time): `make build`
3. Configure `pentagame.toml` (See Config)
4. Setup Database: `make db-setup`
5. Configure application secret key: `make generate`
6. Serve Webserver: `make serve`

#### For development

1. Clone and enter repository from Github: `https://github.com/Chaostheorie/pentagame && cd pentagame`
2. Build (This will take quite some time): `make dev-build`
3. Configure `pentagame.toml` (See Config)
4. Setup Database: `make db-setup`
5. Configure application secret key: `./target/debug/pentagame-online generate`
6. Serve Webserver: `./target/debug/pentagame-online serve`

## Config

Configuration is done via the `pentagame.toml` file. It follows the [TOML](https://toml.io/en/) syntax. Below is an raw skelton. 

NOTE: Syntax relies on use of '...' and doesn't support "..." for e.g. a password

```toml
[server]
ip = '...'
port = 8080

[admin]
name = '...'
admin-password = '...'

[auth]
file = '...'
salt = '...'
session = 24

[database]
user = '...'
password = '...'
host = 'localhost'
port = 5432
database = '...'
```

> The config is read once when the server starts but is not watched during runtime

-   The `[server]` section specifies where the application is served to.
-   The `[admin]` section species credentials for the administrator of the instance. This credentials are needed for e.g shutting down the server.
-   The `[auth]` section specifies the behavior of the session cookie manager. The `session` key specifies the maximum lifetime of a signed cookie. The `file` section specifies the location of the secret key acquired through `make generate` for signing cookies. `salt` is for hashing password with argon2rs.
-   The `[database]` section specifies the credentials for connections from the applicatis to the postgresql server. Ensure that the user has permissions to create databases.
