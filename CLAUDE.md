# CLAUDE.md — AI Assistant Guide for aiobbox

## Project Overview

**aiobbox** is an async Python client library for interacting with the Bouygues Telecom Bbox router API. It wraps the router's HTTP API (tested on F@st5696b, firmware 25.5.28) and exposes methods to retrieve router info, connected hosts, and WAN/IP statistics.

- **Package name:** `aiobbox`
- **Current version:** 0.3.1
- **License:** Apache-2.0
- **Python requirement:** 3.13+
- **Package manager:** `uv`

---

## Repository Structure

```
aiobbox/
├── aiobbox/               # Main package
│   ├── __init__.py        # Public API exports and __version__
│   ├── client.py          # BboxApi async client class
│   ├── exceptions.py      # Custom exception hierarchy
│   ├── models.py          # Pydantic v2 data models
│   └── py.typed           # PEP 561 marker
├── tests/
│   ├── conftest.py        # Pytest fixtures and mock data
│   ├── test_client.py     # Client unit tests
│   ├── test_models.py     # Model validation tests
│   └── integration.py     # Real router integration tests (not run in CI)
├── pyproject.toml         # Project config, dependencies, tool settings
├── .pre-commit-config.yaml
├── flake.nix              # Nix dev environment
└── uv.lock
```

---

## Development Setup

```bash
# Install dependencies (uses uv)
uv sync

# Activate environment (if using direnv)
direnv allow

# Install pre-commit hooks
uv run pre-commit install
```

---

## Running Tests

```bash
# Run all unit tests
uv run pytest

# Run with branch coverage (as CI does)
uv run pytest --cov --cov-branch --cov-report=xml

# Run integration tests against a real router (not in CI)
BBOX_PASSWORD=your_password uv run pytest tests/integration.py -vvs --no-cov
```

Pytest is configured with `asyncio_mode = "auto"` — all async test functions work without explicit decorators.

---

## Linting and Formatting

All checks are enforced via pre-commit and CI. Run manually:

```bash
# Lint and format with ruff
uv run ruff check --fix .
uv run ruff format .

# Type check (excludes tests/)
uv run mypy aiobbox/

# Run all pre-commit hooks
uv run pre-commit run --all-files
```

**Ruff config:** line length 88, rules E/W/F/I/B/C4/UP/ARG/SIM, target Python 3.13.
**MyPy config:** strict mode, pydantic plugin enabled.

---

## Key Code Conventions

### Naming
- Classes: `PascalCase` (e.g., `BboxApi`, `Router`)
- Functions/methods: `snake_case`
- Module-level constants: `UPPER_SNAKE_CASE`
- Private attributes/methods: leading underscore (e.g., `_session`, `_authenticated`)
- Module loggers: `_LOGGER = logging.getLogger(__name__)`

### Type Hints
- All functions must have type annotations (mypy strict mode enforced)
- Use `X | None` union syntax (Python 3.10+), not `Optional[X]`
- All models use Pydantic v2

### Async Patterns
- `BboxApi` supports async context manager (`async with BboxApi(password) as bbox:`)
- Session creation is lazy; the client tracks `_owns_session` to avoid closing externally-provided sessions
- Use `asyncio.timeout()` (Python 3.11+ stdlib) for request timeouts

### Error Handling
- Raise from the custom exception hierarchy (`BboxApiError` base class)
- HTTP 401 → `BboxInvalidCredentialsError` or `BboxSessionExpiredError`
- HTTP 429 → `BboxRateLimitError`
- Timeout → `BboxTimeoutError`

---

## Architecture: Key Files

### `aiobbox/client.py`
The `BboxApi` class is the main entry point. Key methods:
- `authenticate()` — POST form login, stores session cookie
- `get_router_info()` → `Router`
- `get_hosts()` → `list[Host]`
- `get_wan_ip_stats()` → `WANIPStats`

All API methods call `_request()` which handles auth checking, HTTP errors, and JSON parsing.

### `aiobbox/models.py`
Pydantic v2 models. Notable base class behaviors defined in `CustomBaseModel`:
- **Mojibake fix:** `fix_mojibake()` corrects mis-decoded Latin-1/UTF-8 strings from the router API.
- **Empty string normalization:** A model validator converts all empty string fields to `None`.
- **Response unpacking:** Automatically unwraps single-item list API responses.

### `aiobbox/exceptions.py`
Full exception hierarchy:
```
BboxApiError
├── BboxAuthError
│   ├── BboxInvalidCredentialsError
│   └── BboxSessionExpiredError
├── BboxTimeoutError
├── BboxUnauthenticatedError
└── BboxRateLimitError
```

### `aiobbox/__init__.py`
Defines `__version__` and `__all__`. All public classes and exceptions are re-exported here. When adding new public symbols, update both `__all__` and the imports in this file.

---

## Testing Conventions

- Unit tests mock `aiohttp.ClientSession` using fixtures in `conftest.py`
- `conftest.py` defines a `MockResponseFactory` protocol for typing mock responses
- Sample JSON payloads are stored as fixtures (not inline) for reuse across tests
- Tests are async; no `@pytest.mark.asyncio` decorator needed (auto mode)
- Integration tests in `tests/integration.py` are excluded from CI and require `BBOX_PASSWORD` env var

---

## CI/CD

**CI** (`.github/workflows/ci.yml`) runs on push to `main` and PRs:
1. `pre-commit` job — runs all hooks
2. `test` job — matrix over Python 3.13 and 3.14, uploads coverage to Codecov

**Publish** (`.github/workflows/publish.yml`) triggers on `v*` tags:
- Builds wheel/sdist with `uv build`
- Publishes to PyPI via OIDC (no token needed)
- Creates a GitHub Release

---

## Version Management

Uses `commitizen` with conventional commits. Version is stored in:
- `aiobbox/__init__.py` as `__version__`
- `flake.nix` as `version`

Tags use the format `v{version}` with GPG-signed annotated tags.

To bump version:
```bash
uv run cz bump
```

---

## Common Tasks

| Task | Command |
|------|---------|
| Run tests | `uv run pytest` |
| Run tests with coverage | `uv run pytest --cov --cov-branch` |
| Lint | `uv run ruff check --fix .` |
| Format | `uv run ruff format .` |
| Type check | `uv run mypy aiobbox/` |
| All pre-commit checks | `uv run pre-commit run --all-files` |
| Bump version | `uv run cz bump` |
| Build package | `uv build` |

---

## Adding New API Methods

1. Add a new method to `BboxApi` in `client.py` following the existing pattern (call `_request()`, parse response into a model).
2. Define the corresponding Pydantic model in `models.py` (inherit from `CustomBaseModel`).
3. Export the new model from `aiobbox/__init__.py` (add to imports and `__all__`).
4. Add unit tests in `tests/test_client.py` and model tests in `tests/test_models.py`.
5. Add sample fixture data to `tests/conftest.py`.
