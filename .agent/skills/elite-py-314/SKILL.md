---
name: elite-py-314
description: Python Engineering Doctrine (Python ≥ 3.14, Nogil-Ready, Correctness-First)
---

# Python Engineering Doctrine (Python ≥ 3.14)

## I. Core Doctrine
- **Runtime**: Python ≥ 3.14 (Nogil-Ready).
- **Mindset**: Design as if the GIL does not exist (thread-safe by default).

## II. Typing & Static Guarantees
- **Typing**: Mandatory strict typing. Use `pyright` (preferred) or `mypy --strict`.
- **Type Escape Policy**: `# type: ignore` is forbidden unless justified and documented.

## III. Code Style & Structure
- **Formatting**: `ruff` as the single source of truth.
- **Data Modeling**: Prefer immutable models. Use `@dataclass(frozen=True)`.
- **Async**: Use `asyncio.TaskGroup` for structured concurrency.

## IV. Error Model & Boundaries
- **No Silent Failures**: Bare `except:` is forbidden.
- **Explicit Handling**: Errors must be handled explicitly or re-raised with context.
- **Custom Exceptions**: Domain errors must use custom exception classes.

## V. Concurrency & Nogil Safety
- **State**: Avoid shared mutable global state. Prefer message passing or immutable data structures.
- **Primitives**: Thread-safe primitives must be explicit.

## VI. Refactor Governance
- **Legacy Migration**: Migrate legacy code toward strict typing incrementally.
- **Plan First**: Present a refactor plan before changing behavior.

## VII. Testing & Quality
- **Coverage**: ≥ 85% for core logic.
- **Layers**: Unit tests for logic, integration tests for pipelines.

## VIII. Observability
- **Logging**: Structured logging with contextual metadata.
- **Tracing**: Tracing hooks for critical flows.
