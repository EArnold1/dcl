# Instance Lifecycle Management (list / remove)

## Context

`dcl create` currently computes a destination path, materializes the project there (via `git archive|tar` or `git clone --local`), then discards all knowledge of what it just created — nothing is persisted anywhere. There is no way to enumerate, inspect, or clean up clones the tool has made, and no record survives a mid-materialization failure (e.g. a crashed `git archive`, or partial symlink/copy pass), leaving orphaned directories with no trace in the tool itself.

This step adds a persisted registry of managed instances plus `dcl list` and `dcl remove` commands, so that: every instance `dcl` creates is tracked (even if it fails partway through), users can see all clones and their status, and removal is safe — it only ever touches directories that the registry itself created, and works uniformly whether the clone used archive-mode or git-mode.

Decisions confirmed with the user:
- `dcl list` shows **all** instances globally (not scoped to the current project's CWD).
- `dcl remove <name>` targets instances by their **clone directory name** (the destination basename).
- `dcl remove` **prompts for confirmation by default**; `-y`/`--yes` skips the prompt.

## Design

### Registry storage
- New TOML file, **separate from `config.toml`**, rooted at the currently-unused `dirs::data_dir()` (parallel to the existing `dirs::config_dir()`-rooted config): `<data_dir>/devclone/registry.toml`.
- Format: `Registry { instances: Vec<Instance> }`, serialized as TOML array-of-tables via existing `serde`+`toml` deps — no new dependencies (no `serde_json`, `uuid`, `chrono`, or DB crate).
- Unlike `config.toml` (must exist with documented defaults on first run), `registry.toml` is internal bookkeeping: only the directory is eagerly created; the file itself is created lazily on first `Registry::save()`. `Registry::load()` treats a missing file as an empty registry.
- Writes are atomic: serialize to a `.tmp` sibling file, then `fs::rename` over the real path (rename is atomic on POSIX same-filesystem).

### `Instance` record
```rust
pub struct Instance {
    pub name: String,          // destination basename, e.g. "project_docs_new_feature" — the removal target
    pub destination: PathBuf,  // true unique key; uniqueness already enforced by existing DestinationExists check
    pub source: PathBuf,       // project.root_path
    pub revision: String,
    pub mode: Materialization, // reused enum, see below
    pub status: InstanceStatus,
    pub created_at: u64,       // SystemTime -> UNIX_EPOCH seconds (no chrono dep needed)
}

pub enum InstanceStatus { InProgress, Ready, Failed { reason: String } }
```
- `Failed` stores `err.to_string()` (via `thiserror`'s `Display`), not the error itself — `DevCloneError` wraps `io::Error` which isn't `Serialize`.
- Reuse the existing `Materialization` enum in `materialization/project.rs` (currently private, no derives) rather than inventing a duplicate: make it `pub`, derive `Serialize/Deserialize/Clone/Copy/PartialEq`, add `Materialization::from_flag(git: bool)` and replace the inline `if self.request.git {...}` branch in `ProjectMaterializer::materialize()` with it. Re-export via `materialization/mod.rs` (`pub use project::Materialization;`) rather than making all of `mod project` public.

### Hooking into `create()` — this is what makes partial/failed instances visible
`Materializer<Pending>::new()` already computes `destination` (in `manage_destination`) but never exposes it. Add a generic getter:
```rust
impl<S> Materializer<S> {
    pub fn destination(&self) -> &Path { &self.destination }
}
```
Restructure `commands/create.rs::create()` to:
1. Build `Request`, construct `Materializer::new(request)?`, read `destination` via the new getter (don't recompute).
2. **Register the instance with `InProgress` status and save the registry BEFORE calling `.materialize_project()`** — this is the key change: if the process crashes or a command fails mid-way, the registry already has a durable record pointing at a possibly-partial directory.
3. Run `.materialize_project().and_then(|m| m.materialize_environment())`, capturing the `Result` instead of using `?` immediately.
4. Look up the same entry by destination, set status to `Ready` or `Failed { reason: err.to_string() }`, save again.
5. Return the original `result` unchanged, so CLI exit-code/error-printing behavior is unaffected.

### New commands
- `dcl list` (alias `ls`): loads the registry, prints a plain `println!`-based aligned table — `NAME | MODE | REVISION | STATUS | AGE | SOURCE`. Age is a small hand-rolled "Ns/Nm/Nh/Nd ago" formatter off the stored epoch seconds (no new time crate). Use `info!("No managed instances found.")` for the empty case; use raw `println!` for table rows (not `info!`, which would prefix every row with `[INFO]` and break alignment).
- `dcl remove <target>` (alias `rm`), with `-y`/`--yes` to skip confirmation:
  1. `Registry::resolve(target)` — matches by exact `name` (basename); if `target` contains a path separator, matches by exact `destination` path instead (disambiguation escape hatch). No match → `DevCloneError::InstanceNotFound`. Multiple basename matches (rare edge case: same name+revision from different source parents) → `DevCloneError::AmbiguousTarget` listing full destination paths. **This lookup happens before any filesystem mutation — it's the sole gate that prevents `remove` from ever deleting an unmanaged path.**
  2. Unless `--yes`, prompt `Remove instance '<name>' at <path>? [y/N]` via stdin; abort on anything but y/yes.
  3. If `instance.destination.exists()`, `fs::remove_dir_all(&instance.destination)`; otherwise `warn!` and treat as no-op (covers a `Failed`/`InProgress` instance whose directory never got created, or one deleted manually outside `dcl`).
  4. Remove the entry from the registry and save, regardless of whether the directory existed.
- Both mode uniformly: no branching on `instance.mode` in removal. `git`-mode clones are full `git clone --local` checkouts, not `git worktree`s, so there's no worktree metadata to unwind — plain recursive delete is correct for both modes. `fs::remove_dir_all` on Unix `lstat`s entries, so a symlink inside a clone (e.g. symlinked `node_modules`) is unlinked, never followed/recursed into — deleting a clone can never touch the original project's real files.

### New `DevCloneError` variants (src/error.rs)
```rust
#[error("failed to parse registry: {0}")]
RegistryParse(String),
#[error("failed to write registry: {0}")]
RegistryWrite(String),
#[error("no managed instance found matching: {0}")]
InstanceNotFound(String),
#[error("multiple instances match '{target}': {candidates:?}; specify the full destination path to disambiguate")]
AmbiguousTarget { target: String, candidates: Vec<String> },
```
Existing variants (`Io`, `DestinationExists`, `ConfigParse`, etc.) are reused as-is.

## Files to change

**New:**
- `src/registry/mod.rs` — `Registry { instances: Vec<Instance> }`, `load()`/`save()` (atomic write), `add()`, `find_mut_by_destination()`, `remove()`, `resolve()`.
- `src/registry/instance.rs` — `Instance`, `InstanceStatus`, `Instance::new()`.
- `src/commands/list.rs` — `list()` + `status_label()`/`humanize_age()` helpers.
- `src/commands/remove.rs` — `remove(target, yes)` + `confirm()` helper (note: clone the `&Instance` returned by `resolve()` before calling `registry.remove()`, to end the immutable borrow first).

**Modified:**
- `src/main.rs` — add `pub mod registry;`.
- `src/commands/mod.rs` — add `pub mod list; pub mod remove;`.
- `src/cli.rs` — add `List` (alias `ls`) and `Remove { target: String, #[arg(short='y', long)] yes: bool }` (alias `rm`) variants + match arms, mirroring the existing `#[command(alias = "c")]` convention.
- `src/config/paths.rs` — add `data_dir`/`registry_file` fields, populate via `dirs::data_dir()`, add `ensure_data_dir()` (dir only, no eager file write) called from `init()`, add `registry_file()` getter, add `REGISTRY_FILE` const.
- `src/error.rs` — add the four variants above.
- `src/materialization/mod.rs` — add `pub use project::Materialization;` (keep `mod project;` private otherwise).
- `src/materialization/project.rs` — make `Materialization` `pub` with the new derives; add `from_flag`/`as_str`; use `from_flag` in `ProjectMaterializer::materialize()`.
- `src/materialization/materializer.rs` — add `impl<S> Materializer<S> { pub fn destination(&self) -> &Path }`.
- `src/commands/create.rs` — restructure per "Hooking into `create()`" above.
- `Cargo.toml` — **no changes**; everything needed is already a dependency.

## Verification
1. `cargo build` and `cargo clippy` clean.
2. `cargo test` — existing `environment.rs` test still passes; the codebase's convention is inline `#[cfg(test)]` modules, so add a small one for `Registry::resolve` (unique match / not-found / ambiguous) following that pattern.
3. Manual end-to-end in a scratch git repo:
   - `dcl create <rev>` (archive mode) → `dcl list` shows it as `ready`.
   - `dcl create <rev2> --git` → `dcl list` shows both, correct `mode` column.
   - Force a failure (e.g. bad revision name) → `dcl list` shows `failed (<reason>)` and the entry persists.
   - `dcl remove <name>` without `-y` → prompts, `n` aborts, directory untouched; `y` deletes directory and removes registry entry; verify a symlinked `node_modules` inside the clone is removed but the original project's `node_modules` is untouched.
   - `dcl remove <name> -y` on a `failed` instance whose directory doesn't exist → no-op warning, registry entry still removed.
   - `dcl remove nonexistent-name` → `InstanceNotFound` error, nothing deleted.
