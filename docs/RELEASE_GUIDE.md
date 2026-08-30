# Release Guide for `dcl`

This guide describes the release process for the `dcl` project: hand-rolled GitHub Actions, triggered by pushing a version tag. There is no `cargo-dist` involved.

## Overview

Pushing a tag matching `v*.*.*` (e.g. `v0.2.0`) triggers `.github/workflows/release.yml`, which:

1. Builds release binaries for five targets in parallel (`build` job, matrix).
2. Stages each binary with `README.md` and `LICENSE`, archives it (`.tar.gz` on macOS/Linux, `.zip` on Windows), and generates a `.sha256` checksum.
3. Uploads each archive as a GitHub Actions artifact.
4. A single `release` job (`needs: build`) downloads all artifacts and creates exactly **one** GitHub Release, attaching every archive and checksum, with auto-generated release notes.

### Supported Platforms

| Target | Runner |
|---|---|
| `aarch64-apple-darwin` (macOS Apple Silicon) | `macos-latest` |
| `x86_64-apple-darwin` (macOS Intel) | `macos-latest` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` (native ARM64, no Docker/`cross`) |
| `x86_64-pc-windows-msvc` | `windows-latest` |

### Artifacts Generated per Release

- `dcl-<target>.tar.gz` (macOS/Linux) or `dcl-<target>.zip` (Windows), each containing `dcl` (or `dcl.exe`), `README.md`, and `LICENSE`.
- A matching `dcl-<target>.tar.gz.sha256` / `.zip.sha256` checksum file per archive.
- No installer scripts — users download and extract manually.

## Release Process

### Step 1: Prepare Your Changes

```bash
git switch main
git pull origin main
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

### Step 2: Bump the Version

Edit `Cargo.toml`:

```toml
[package]
name = "dcl"
version = "0.2.0"  # was 0.1.0
```

### Step 3: Commit the Version Bump

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
```

### Step 4: Tag and Push

The tag **must** match `v<version-from-Cargo.toml>` — the workflow does not read `Cargo.toml` to determine the release version, it uses the pushed tag directly (`github.ref_name`).

```bash
git tag v0.2.0
git push origin main --tags
```

This single command pushes both the commit and the tag; the tag push is what triggers the `Release` workflow. (The `CI` workflow also runs separately on the `push to main`, as before — the two workflows are independent.)

### Step 5: Watch the Workflow

Go to **Actions → Release** and confirm:
- All 5 `Build (<target>)` jobs succeed.
- The single `Create GitHub Release` job succeeds after all builds finish.

### Step 6: Verify the Release

1. Open <https://github.com/EArnold1/dcl/releases> and confirm the new release has 5 archives + 5 checksum files attached (10 assets total).
2. Download and smoke-test a binary:

   ```bash
   curl -LO https://github.com/EArnold1/dcl/releases/download/v0.2.0/dcl-x86_64-apple-darwin.tar.gz
   tar xzf dcl-x86_64-apple-darwin.tar.gz
   ./dcl-x86_64-apple-darwin/dcl --version
   ```

## Troubleshooting

### Tag doesn't match the version in `Cargo.toml`

**Symptom**: You tagged `v0.2.0` but forgot to bump `Cargo.toml`, or vice versa — the release gets created with the wrong version baked into the binary (`dcl --version` reports the old number), or the tag itself is just wrong.

**Cause**: The release tag and `Cargo.toml`'s `version` field are two independent, manually-synchronized values — nothing in the workflow currently cross-checks them.

**Solution**:
- Before tagging, double check `grep '^version' Cargo.toml` matches the tag you're about to push.
- If you already pushed a wrong tag: delete the tag locally and on the remote (`git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`), delete the resulting GitHub Release if one was created, fix `Cargo.toml`, and re-tag.
- (Optional future hardening: add a cheap `run` step at the top of the `build` job that parses `Cargo.toml` and fails fast if it doesn't match `github.ref_name`.)

### Release job fails to find artifacts

**Symptom**: The `Create GitHub Release` job fails, or the release is missing some platform's archive.

**Cause**: One or more `build` matrix jobs failed (check each job individually — `fail-fast: false` means the others still ran), or `actions/download-artifact` didn't find anything because every `build` job failed.

**Solution**:
- Open the failed `Build (<target>)` job's logs and fix the underlying build error.
- Re-run only the failed jobs from the Actions UI, or push a new patch tag after fixing.
- Confirm each `build` job actually produced a non-empty `dcl-<target>.tar.gz`/`.zip` before the `Upload artifact` step (the workflow uses `if-no-files-found: error` there specifically to catch this early rather than silently skipping).

### Native ARM64 Linux runner (`ubuntu-24.04-arm`) unavailable

**Symptom**: The `aarch64-unknown-linux-gnu` job fails to even start, or GitHub reports the runner label as invalid/unavailable.

**Cause**: `ubuntu-24.04-arm` hosted runners are free and generally available for public repositories as of 2025, but availability could change if the repo becomes private (still usable, billed at the same per-minute rate as `ubuntu-latest` — no extra multiplier for Linux ARM64) or if GitHub changes runner offerings.

**Fallback** (not currently implemented, documented here for reference): cross-compile using the [`cross`](https://github.com/cross-rs/cross) tool with Docker on a regular `ubuntu-latest` runner, per the [Rust CLI book's approach](https://rust-cli.github.io/book/tutorial/packaging.html#distributing-binaries). This is more complex (requires Docker-in-Docker on the runner and a `Cross.toml`) and was intentionally avoided while native ARM64 runners are available.

### Build fails on a specific platform

**Solution**: same as before — check that job's logs, fix locally, commit, re-tag (or delete and re-push the same tag if you deleted the bad release first).

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (e.g., 1.0.0): Breaking changes to the CLI interface or behavior.
- **MINOR** (e.g., 0.2.0): New features, backward compatible.
- **PATCH** (e.g., 0.1.1): Bug fixes, backward compatible.

## Rollback

If a release has critical issues:

1. Revert the problematic commit(s).
2. Bump to a new patch version (e.g., `0.2.1`).
3. Commit, tag `v0.2.1`, push with `--tags`.
4. Optionally mark the bad release as a pre-release/draft, or delete it, on the GitHub Releases page.

## Summary Checklist

- [ ] All features/fixes merged to `main`
- [ ] Tests, clippy, fmt pass locally
- [ ] `Cargo.toml` version bumped
- [ ] Commit the version bump
- [ ] `git tag vX.Y.Z` (matches `Cargo.toml`)
- [ ] `git push origin main --tags`
- [ ] All 5 `build` jobs green in Actions
- [ ] `release` job green, GitHub Release has 10 assets (5 archives + 5 checksums)
- [ ] Smoke-test at least one downloaded binary
