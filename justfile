_list:
    just --list

[group("dev")]
install:
    uv sync --frozen --all-groups --reinstall-package cedar-python

[group("dev")]
format:
    uv run ruff format
    uv run ruff check --select I --fix
    just --dump > justfile.fmt
    mv justfile.fmt justfile

[group("dev")]
check-format:
    uv run ruff format --check

[group("dev")]
check-code:
    uv run ruff check

[group("dev")]
check-types:
    uv run python -m mypy.stubtest cedar._lib
    uv run mypy python tests

[group("pre-commit")]
check: check-code check-format check-types

[group("dev")]
test:
    uv run pytest

[group("package")]
lock:
    uv sync --all-groups

[group("package")]
build:
    uv build

[group("docs")]
docs:
    uv run mkdocs serve

[group("docs")]
deploy-docs:
    #!/usr/bin/env bash

    set -eu

    echo -e "Running command: mike deploy -u --push $VERSION $ALIAS"

    uv run --group docs --no-group dev mike deploy -u --push $VERSION $ALIAS
