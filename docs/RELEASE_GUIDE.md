# Release Guide for `dcl`

This guide describes the automated release process and how to execute releases for the `dcl` project.

## Overview

`dcl` uses `cargo-dist` for cross-platform binary distribution via GitHub Releases. The release workflow is automated and triggers on pushes to `main` when the version in `Cargo.toml` changes.

### Supported Platforms

- macOS Intel (x86_64)
- macOS Apple Silicon (aarch64)
- Linux x86_64 (glibc)
- Linux aarch64 (glibc)
- Windows x86_64 (MSVC)

### Artifacts Generated per Release

- Prebuilt binaries for all platforms (tarballs on Unix, zip on Windows)
- Shell installer script (`dcl-installer.sh`) for macOS/Linux
- PowerShell installer script (`dcl-installer.ps1`) for Windows
- GitHub Release with all binaries and installer scripts

## Release Process

### Step 1: Prepare Your Changes

Ensure all desired features and fixes are merged into `main`:

```bash
git switch main
git pull origin main
```

Run tests and verify everything works:

```bash
cargo test
cargo clippy
cargo fmt --check
```

### Step 2: Bump the Version

Edit `Cargo.toml` and update the version number following [Semantic Versioning](https://semver.org/):

```toml
[package]
name = "dcl"
version = "0.2.0"  # was 0.1.0
```

### Step 3: Commit the Version Bump

Commit the version change:

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
```

### Step 4: Push to Main

Push the commit to `main`:

```bash
git push origin main
```

### Step 5: Automated Release Workflow

Once you push to `main`, the CI workflow runs first (tests, clippy, formatting on all platforms). After CI completes successfully, the GitHub Actions release workflow automatically triggers and:

1. **Detects the version change** by reading `Cargo.toml`
2. **Checks if the release already exists** to avoid duplicates
3. **Runs cargo-dist** to build cross-platform binaries:
   - Compiles for all target platforms
   - Generates installer scripts
   - Creates tarballs/zips
4. **Creates a GitHub Release** with:
   - Auto-generated release notes (from commits since last release)
   - All prebuilt binaries
   - Installer scripts

### Step 6: Verify the Release

After the workflow completes (check GitHub Actions tab):

1. Navigate to [Releases](https://github.com/EArnold1/devclone/releases)
2. Verify the new release is published with all platforms
3. Download and test a binary locally:

   ```bash
   # Example: macOS x86_64
   tar xzf dcl-v0.2.0-x86_64-apple-darwin.tar.gz
   ./dcl --version
   ```

4. Test the installer script (optional):

   ```bash
   # For macOS/Linux
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/EArnold1/devclone/releases/download/v0.2.0/dcl-installer.sh | sh
   ```

## Manual Release Trigger

If needed, you can manually trigger a release without waiting for CI:

1. Go to **Actions** → **Release**
2. Click **Run workflow**
3. (Optional) Specify a version in the input field
4. Click **Run workflow**

This is useful for:
- Re-releasing if the workflow failed or you need to force a release
- Testing the release process
- Rebuilding an existing release version for a platform that failed

## Troubleshooting

### Release Workflow Doesn't Trigger

**Symptom**: You pushed to `main` but no release is being created.

**Cause**: The CI workflow failed, a release with that version already exists, or the version in `Cargo.toml` hasn't changed.

**Solution**:
- Check that the **CI workflow** passed successfully on `main` (the release workflow only runs after CI succeeds)
- Verify you updated `Cargo.toml` with a new version
- Check if a release with that version already exists on the Releases page
- Check GitHub Actions logs for the exact error

### Build Fails on a Specific Platform

**Symptom**: The workflow completes but only some platforms' binaries are available.

**Cause**: The build failed for that platform (e.g., Windows compilation error).

**Solution**:
- Check the GitHub Actions workflow run logs
- Look for the failing job (e.g., "dist (x86_64-pc-windows-msvc)")
- Fix the issue locally, commit, and re-push to trigger the workflow again

### Installer Script Returns 404

**Symptom**: Running the installer script fails with a 404 error.

**Cause**: The release wasn't fully uploaded yet, or the URL is incorrect.

**Solution**:
- Wait a few minutes for the release to fully propagate
- Verify the release version in the URL matches the GitHub release
- Check that the release exists on the Releases page with binaries attached

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (e.g., 1.0.0): Breaking changes to the CLI interface or behavior
- **MINOR** (e.g., 0.2.0): New features, backward compatible
- **PATCH** (e.g., 0.1.1): Bug fixes, backward compatible

Example progression:
- 0.1.0 (initial release)
- 0.1.1 (bug fix)
- 0.2.0 (new feature)
- 1.0.0 (stable release, API locked)

## Files Modified During Release

When you bump the version and push to `main`:

- `Cargo.toml` — Version number updated
- `Cargo.lock` — Updated automatically on next build

No other files need manual updates. The workflow handles:
- Creating the GitHub Release
- Building and uploading binaries
- Generating release notes

## Future: Publishing to crates.io

Currently, `dcl` is distributed via GitHub Releases only. To also publish to [crates.io](https://crates.io):

1. Create an account at crates.io and generate an API token
2. Add `cargo publish` step to the release workflow
3. Update README with `cargo install dcl` instructions

This is optional and can be added in a future release without changing the current workflow.

## Rollback

If a release has critical issues:

1. Revert the problematic commit
2. Bump the version to a patch release (e.g., 0.2.1)
3. Push to `main` to trigger a new release
4. (Optional) Delete or mark the problematic release on GitHub as a pre-release or draft

## Summary Checklist

- [ ] All features/fixes merged to `main`
- [ ] Tests pass locally
- [ ] Update version in `Cargo.toml`
- [ ] Commit version bump
- [ ] Push to `main`
- [ ] Wait for GitHub Actions to complete
- [ ] Verify release on GitHub Releases page
- [ ] Test installer script
- [ ] Announce release in project documentation or changelog
