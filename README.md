# dcl

`dcl` (devclone) is a command-line tool for creating a ready-to-use clone of a local development project.

## The Problem

When working across Git branches, you may want to run or inspect another branch while keeping your current branch untouched.

Git worktrees can solve this, but they introduce worktree-specific metadata and aren't always the workflow you want.

An alternative is to clone the project again and set it up from scratch. However, that means reinstalling dependencies and recreating the local development environment. For projects with directories such as `node_modules`, this can be slow and wasteful.

The result is that you end up with multiple copies of the same dependencies and spend time getting a second checkout ready to use.

`dcl` solves this by creating a new local project clone and allowing reusable parts of the existing environment to be **symlinked instead of copied**.

## Installation

### From GitHub Releases

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/EArnold1/devclone/releases/latest/download/dcl-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://github.com/EArnold1/devclone/releases/latest/download/dcl-installer.ps1 | iex"
```

### From Source

If you have Rust installed:

```bash
cargo install --git https://github.com/EArnold1/devclone
```

For example:

```text
project/
├── node_modules/
├── src/
└── package.json

project-clone/
├── node_modules -> ../project/node_modules
├── src/
└── package.json
```

The clone can therefore be used immediately without reinstalling dependencies.

## Requirements

**⚠️ Important**: `dcl` requires Git to be installed and available in your PATH. The tool uses Git to create and manage project clones.

## How It Works

`dcl` operates on the current working directory:

```bash
dcl
```

It:

1. Uses the current directory as the project.
2. Reads the devclone configuration.
3. Creates a new clone of the project.
4. Copies files and directories that should be independent.
5. Symlinks files and directories that can be shared.
6. Ignores files and directories that should not be cloned.

The result is another working copy that is ready to use while sharing selected local resources with the original project.

## Configuration

Configuration is stored in:

```text
~/.config/devclone/config.toml
```

The configuration defines rules for:

- **copies** — files and directories that should be copied into the clone.
- **symlinks** — files and directories that should be shared through symbolic links.
- **ignore** — files and directories that should not be included.

For example, a project may copy its source and configuration files while symlinking `node_modules` so dependencies do not need to be installed again.

## Development

Build:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

## License

See `LICENSE` for details.
