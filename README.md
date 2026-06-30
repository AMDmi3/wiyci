# wiyci

[![CI](https://github.com/AMDmi3/wiyci/actions/workflows/ci.yml/badge.svg)](https://github.com/AMDmi3/wiyci/actions/workflows/ci.yml)

A template for rust project consisting of a website backend and update
daemon, connected through a PostgreSQL database. Suitable for building
services like [Repology](https://github.com/repology/repology-rs).

## Requirements

This code requires latest Rust-nightly.

## Running

1. Prepare the database

   (note that you likely want stronger password for production usage)

   ```
   sudo -u postgres psql -c "CREATE DATABASE wiyci"
   sudo -u postgres psql -c "CREATE USER wiyci WITH PASSWORD 'wiyci'"
   sudo -u postgres psql -c "GRANT ALL ON DATABASE wiyci TO wiyci"
   ```

2. Run the daemon

   (note that you DSN may vary depending on postgresql settings)

   ```
   cargo run --bin wiyci-daemon -- --dsn postgresql://wiyci:wiyci@localhost/wiyci
   ```

3. Run the webapp

   ```
   cargo run --bin wiyci-web -- --dsn postgresql://wiyci:wiyci@localhost/wiyci --listen 127.0.0.1:3000
   ```

## Author

- [Dmitry Marakasov](https://github.com/AMDmi3) <amdmi3@amdmi3.ru>

## License

- [GPLv3 or later](LICENSE).
