# Borges
A personal review and note-taking app for books that I read. This is a work-in-progress, so come back soon!

### Setup
- `cp env-example .env`
- `docker compose up -d` to get PostgreSQL running
- `psql -h localhost -d borges -U root -f dump.sql` to create tables and import sample data
- `cargo run`
- `open localhost:8000`
