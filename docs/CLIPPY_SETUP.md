# Bazel Clippy Integration

## Overview

Clippy is now integrated into the Bazel build system as test targets, ensuring code quality checks are part of the standard testing workflow.

## Available Clippy Targets

- **`:clippy`** - Runs clippy on the main library code (`src/lib.rs`)
- **`:clippy_tests`** - Runs clippy on unit tests
- **`:clippy_integration`** - Runs clippy on integration tests

## Usage

### Run a specific clippy check

```bash
bazel test :clippy
bazel test :clippy_tests
bazel test :clippy_integration
```

### Run all clippy checks

```bash
bazel test :clippy :clippy_tests :clippy_integration
```

### Run all tests including clippy

```bash
bazel test :check
```

This runs the `:check` test suite which includes:
- All clippy checks (library, unit tests, integration tests)
- Unit tests
- Integration tests

### Run all tests in the workspace

```bash
bazel test //...
```

or

```bash
bazel test :all
```

This will run all test targets including clippy checks.

## Configuration

All clippy targets are marked with `testonly = True`, which means they are treated as test targets by Bazel. This ensures clippy runs as part of the standard test workflow rather than just as a build action.

## Continuous Integration

For CI pipelines, use:

```bash
bazel test :check
```

This ensures both functional tests and linting checks pass before merging code.

## Comparison with Cargo

| Cargo Command | Bazel Equivalent |
|---------------|------------------|
| `cargo clippy` | `bazel test :clippy :clippy_tests :clippy_integration` |
| `cargo test` | `bazel test :unit_tests :integration_tests` |
| `cargo clippy && cargo test` | `bazel test :check` |
