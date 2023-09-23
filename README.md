# Borges
A personal review and note-taking app for books that I read. This is a work-in-progress, so come back soon!

### Setup
- `cp env-example .env`
- `docker compose up -d` to get PostgreSQL running
- `cargo install sqlx-cli` to get the sqlx binary
- `sqlx database create && sqlx migrate run` to create tables and import sample data
- `cargo run`
- `open localhost:8000`

### Testing
Tests use [insta](https://insta.rs/) for snapshot/approval testing to compare expected responses from the GraphQL server. 
They are also using SQLx's [test](https://docs.rs/sqlx/latest/sqlx/attr.test.html) attribute in conjunction with the `migrate` feature to provide a fresh 
database for data isolation in each test.

Running tests:
- `cargo test`
- `cargo insta review` to review any changes to snapshot files
