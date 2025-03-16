# python cedar

Experiment with [cedar](https://cedarpolicy.com/) and [pyo3](https://pyo3.rs/v0.23.5/rust-from-python.html).

## Install or update

```bash
uv sync --frozen --all-groups --reinstall-package cedar-python
```

This command must be run any time rust code is updated in order to use python bindings.

## Test

```bash
uv run pytest
```

## Build

```bash
uv build
```

## View documentation

```bash
uv run mkdocs serve
```

## Build documentation

```bash
uv run mkdocs build
```
