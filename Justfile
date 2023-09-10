# just manual: https://github.com/casey/just

_default:
	@just --list

# Run the fastapi server
python:
	#!/usr/bin/env bash
	set -euxo pipefail
	cd {{justfile_directory()}}/python-fastapi
	poetry run uvicorn app.main:app --log-level critical

# Run the axum server
rust:
	#!/usr/bin/env bash
	set -euxo pipefail
	cd {{justfile_directory()}}/rust-axum
	DATABASE_URL=postgres://postgres:password@localhost:5432/benchmark cargo run --release

# Open psql for the database
psql:
	PGPASSWORD=password psql -h localhost -p 5432 -U postgres benchmark

# run the stress test for N requests and C workers
oha N='50000' C='10':
	oha -n {{N}} -c {{C}} http://localhost:8000/

# Initialize the postgres database
initdb:
	#!/usr/bin/env bash
	set -euxo pipefail
	cd {{justfile_directory()}}/scripts
	bash init_db.sh
