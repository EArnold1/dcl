# dcl

`dcl` (devclone) is a command-line tool for creating a ready-to-use clone of a local development project without reinstalling dependencies.

## The Idea

When you need to run or test a different branch while keeping your current work untouched, you have a few options:

- **Git worktrees** work but add metadata and aren't always your preferred workflow
- **Full clones** give you a clean checkout but require reinstalling all dependencies, often time-consuming for projects with `node_modules`, `.venv`, or other large dependency directories

`dcl` solves this by creating a new project clone that **symlinks shared resources** (like `node_modules`) instead of copying them. You get a ready-to-use checkout without the installation overhead.

**Example:**

```
project/
├── node_modules/
├── src/
└── package.json

project_feature_branch/
├── node_modules → ../project/node_modules (symlink)
├── src/
└── package.json
```

The clone is ready to use immediately—no dependency reinstallation needed.

## Installation

### Homebrew (macOS)

```bash
brew install EArnold1/dcl/dcl
```

### From GitHub Releases

Download the archive for your platform from [GitHub Releases](https://github.com/EArnold1/dcl/releases/latest).

**macOS (Apple Silicon):**

```bash
curl -LO https://github.com/EArnold1/dcl/releases/latest/download/dcl-aarch64-apple-darwin.tar.gz
tar xzf dcl-aarch64-apple-darwin.tar.gz
sudo mv dcl-aarch64-apple-darwin/dcl /usr/local/bin/
```

**macOS (Intel):**

```bash
curl -LO https://github.com/EArnold1/dcl/releases/latest/download/dcl-x86_64-apple-darwin.tar.gz
tar xzf dcl-x86_64-apple-darwin.tar.gz
sudo mv dcl-x86_64-apple-darwin/dcl /usr/local/bin/
```

**Note:** If you encounter a Gatekeeper "unidentified developer" warning when running `dcl`, remove the quarantine attribute:

```bash
xattr -d com.apple.quarantine /usr/local/bin/dcl
```

**Linux (x86_64):**

```bash
curl -LO https://github.com/EArnold1/dcl/releases/latest/download/dcl-x86_64-unknown-linux-gnu.tar.gz
tar xzf dcl-x86_64-unknown-linux-gnu.tar.gz
sudo mv dcl-x86_64-unknown-linux-gnu/dcl /usr/local/bin/
```

**Linux (aarch64):**

```bash
curl -LO https://github.com/EArnold1/dcl/releases/latest/download/dcl-aarch64-unknown-linux-gnu.tar.gz
tar xzf dcl-aarch64-unknown-linux-gnu.tar.gz
sudo mv dcl-aarch64-unknown-linux-gnu/dcl /usr/local/bin/
```

**Windows:**

1. Download [`dcl-x86_64-pc-windows-msvc.zip`](https://github.com/EArnold1/dcl/releases/latest/download/dcl-x86_64-pc-windows-msvc.zip).
2. Extract the archive.
3. Move `dcl.exe` into a directory on your `PATH`.

Each archive has a matching `.sha256` checksum file. Verify a download with, e.g.:

```bash
shasum -a 256 -c dcl-x86_64-apple-darwin.tar.gz.sha256
```

### From Source

If you have Rust installed:

```bash
cargo install --git https://github.com/EArnold1/dcl
```

## Requirements

`dcl` requires Git to be installed and available in your PATH.

## How It Works

Navigate to your project root and run `dcl`:

```bash
cd my-project
dcl
```

It will then:

1. Read the devclone configuration
2. Create a new clone of the project in a sibling directory
3. Copy files and directories that should be independent
4. Symlink files and directories that can be shared
5. Ignore files and directories based on your configuration

The result is a ready-to-use working copy that shares selected local resources with the original project.

## Configuration

Configuration is stored in `~/.config/devclone/config.toml`.

The configuration file defines three categories:

- **symlinks** — files and directories that should be shared with the original project (e.g., `node_modules`, `.venv`)
- **copies** — files and directories that should be copied (e.g., `.env` files, configuration files)
- **ignore** — files and directories to exclude entirely (e.g., `.git`, build artifacts)

By default, `dcl` symlinks dependency directories and copies environment/configuration files, so you can use the clone immediately without reinstalling dependencies.

## Development

**Build:**

```bash
cargo build
```

**Run tests:**

```bash
cargo test
```

**Lint and format check:**

```bash
cargo clippy
cargo fmt --check
```

## License

See `LICENSE` for details.
