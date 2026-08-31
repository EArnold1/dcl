# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

**dcl** (devclone) is a CLI tool for creating ready-to-use local clones of development projects. It solves the problem of needing multiple checkouts by allowing selective symlinking of shared resources (like `node_modules`) while copying independent parts, avoiding redundant reinstallation and setup time.

## Commands

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- create <revision> [--git]
```

- Creates a clone of the current project at the given revision
- `<revision>`: Git revision/branch name (e.g., `docs/new-feature`)
- `--git`: Uses `git clone --local` instead of `git archive` for project materialization

### Tests

```bash
cargo test
```

Run a single test:

```bash
cargo test ignored_paths_take_precedence_over_include_patterns
```

### Linting & Format Checking

```bash
cargo clippy
cargo fmt --check
```

## Architecture

### Entry Point Flow

1. **cli.rs** - Parses CLI arguments, delegates to command handlers
2. **commands/create.rs** - Coordinates the clone creation:
   - Loads config from `~/.config/devclone/config.toml`
   - Discovers project identity (name, root path)
   - Creates a `Materializer` instance and executes two phases

### State-Based Materializer Pattern

`materializer.rs` uses Rust's type-state pattern for a two-phase process:

**Phase 1: Project Materialization** (`Pending → ProjectMaterialized`)

- Handled by `ProjectMaterializer` in `materialization/project.rs`
- Creates the destination directory with normalized revision name (e.g., `project_docs_new_feature`)
- Two strategies (controlled by `--git` flag):
  - **Archive mode** (default): `git archive | tar` extracts files at a specific revision
  - **Git mode**: `git clone --local` creates a full git repository and checks out the revision
- Destination path: `<parent_of_project>/<project_name>_<normalized_revision>`

**Phase 2: Environment Materialization** (`ProjectMaterialized → Done`)

- Handled by `EnvironmentMaterializer` in `materialization/environment.rs`
- Discovers and processes files/directories matching config patterns
- Uses glob pattern matching to categorize paths
- Parallel processing with Rayon for performance
- Three operations applied in precedence order:
  1. **ignore** patterns - excluded entirely
  2. **symlinks** patterns - creates symlinks pointing to original project
  3. **copies** patterns - recursively copies files/directories
- Config file format (TOML with `[symlinks]`, `[copies]`, `[ignore]` sections) supports glob patterns

### Configuration System

- **File location**: `~/.config/devclone/config.toml`
- **Auto-initialization**: First run creates config with sensible defaults if missing
- **Default patterns** (in `config/paths.rs`):
  - **symlinks**: `node_modules`, `.pnpm-store`, `.yarn/cache`, `.bun`, `.venv`, `venv`
  - **copies**: `.env*` files, `.npmrc`, `.yarnrc*`, `bunfig.toml`, `.cargo/config.*`
  - **ignore**: `.git`, build outputs (`target`, `dist`, `build`), framework caches (`.next`, `.nuxt`, etc.), test caches, logs, OS files

### Error Handling

- Custom `DevCloneError` enum in `error.rs` using `thiserror` crate
- Key errors: I/O, glob pattern validation, config parsing, git command failures, destination conflicts
- Errors propagate up to `main()` which prints and exits with code 1

### Discovery

- `discovery/project.rs` discovers the current project by:
  - Getting current working directory
  - Using the directory name as the project name
  - No git-specific detection (works with any directory)

## Key Implementation Details

### Glob Pattern Matching

- Patterns support `**` for recursive matching
- Leading slashes are normalized away
- Relative path ancestors are checked (e.g., matching `packages/` also matches `packages/node_modules/react`)
- This allows efficient filtering with patterns like `**/node_modules`

### Parallel Processing

- `EnvironmentMaterializer::materialize()` collects all paths to materialize first
- Then processes them in parallel with Rayon's `par_iter()` for independent symlink/copy operations
- Error handling propagates the first failure via `try_for_each`

### Directory Naming Convention

- Source: `project`
- Clone of revision `docs/new-feature`: `project_docs_new_feature`
- Slashes and hyphens replaced with underscores to create valid directory names

## Commit Message Guidelines

**IMPORTANT**: Do NOT add `Co-Authored-By` or `Claude-Session` trailers to commit messages. Only include the substantive commit message content.

## Testing

- Tests are inline with `#[cfg(test)]` modules
- Current test coverage: `environment.rs` validates that ignore patterns take precedence
- Tests use hardcoded mock paths (not file system dependent)

## Dependencies

- **clap**: CLI argument parsing with derive macros
- **globset**: Efficient multi-pattern glob matching
- **rayon**: Data parallelism for environment materialization
- **serde/toml**: Configuration file parsing
- **thiserror**: Error type derivation
- **dirs**: Platform-aware config directory resolution
