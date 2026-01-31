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

## VI. Worker Life-cycle & IPC (Mission 014)
- **Fate-sharing**: Workers MUST implement a "Watchdog" thread monitoring `stdin`. If `stdin` closes, the process MUST exit immediately.
- **Standalone Mode**: Workers should be runnable via CLI with `--path` and `--lane` arguments for testing and isolation.
- **JSON Serialization**: Return data MUST be valid JSON matching the `ExtractionProduct` schema.
- **Dependency Isolation**: Prefer lightweight libraries for workers (e.g., polars, pymupdf) to keep memory footprint predictable.

## VII. Testing & Quality
- **Coverage**: ≥ 85% for core logic.
- **Layers**: Unit tests for logic, integration tests for pipelines.

## VIII. Observability
- **Logging**: Structured logging with contextual metadata.
- **Tracing**: Tracing hooks for critical flows.
## IX. Data Purity Protocol (The Janitor's Decree)
- **Encoding Boundary**: All text data sent to Rust via IPC MUST be strictly UTF-8. 
- **No Mojibake**: Workers are responsible for detecting and discarding malformed characters before serialization.
- **Pure Extraction**: Python Workers focus on RAW extraction. No cleaning or normalization of numbers should happen here; that is the Janitor's role in Rust.
- **IPC Contract**: Match the `camelCase` protocol and avoid non-UTF-8 compatible byte streams in JSON payloads.
