# The World Is Your CI

[![CI](https://github.com/AMDmi3/wiyci/actions/workflows/ci.yml/badge.svg)](https://github.com/AMDmi3/wiyci/actions/workflows/ci.yml)

A web service which aggregates build logs of F/OSS projects and serves
as an impromptu CI, reporting build failures, warnings, and failed tests.

## Requirements

This project requires latest Rust-nightly to build and PostgreSQL 17+ to run.

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
   cargo run --bin wiyci-daemon -- --dsn postgresql://wiyci:wiyci@localhost/wiyci --storage-path /tmp/wiyci-logs
   ```

3. Run the webapp

   ```
   cargo run --bin wiyci-web -- --dsn postgresql://wiyci:wiyci@localhost/wiyci --listen 127.0.0.1:3000
   ```

## Author

- [Dmitry Marakasov](https://github.com/AMDmi3) <amdmi3@amdmi3.ru>

## License

- [GPLv3 or later](LICENSE).
