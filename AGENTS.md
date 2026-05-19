# AGENTS.md

A focused orientation file for any agent (LLM or human) opening this
repository for the first time. It complements `README.md` and the
inline rustdoc without duplicating them.

If you only read one file before touching anything, read this one.

## What this project is

`safeatomic-rs` is a **small, deliberately bounded** Rust crate
providing atomic POSIX filesystem primitives. It is the Rust sibling
of the Python [`safeatomic`](https://github.com/deepcausa/safeatomic)
package: **not** a binding, **not** a 1:1 port. Different surface,
same engineering values.

Scope is one whole file at a time, on a local POSIX filesystem
(Linux, macOS). It is not a WAL, not a database, not a distributed
coordination primitive. For an append-only frame log with recovery,
see the sibling crate [`datawal`](https://github.com/deepcausa/datawal).

## Hard invariants — do not break these

These are contractual. Violating any of them is a major version
event and must be reflected in `CHANGELOG.md` plus a brief design
note in `dev/decisions.md` (private).

1. **`write_atomic` always fsyncs the target file _and_ its parent
   directory.** There is no `fsync=false` shortcut. Opting out
   would defeat the core promise.
2. **No silent copy+delete fallback under any function whose name
   promises atomicity.** The crate calls `fs::rename`, which is
   atomic on the same filesystem and returns `EXDEV` across
   filesystems. The error is surfaced verbatim; cross-device moves
   are the caller's problem to express explicitly.
3. **`write_once` is the O_CREAT|O_EXCL primitive.** It returns
   `Ok(true)` on creation, `Ok(false)` if the target already exists,
   `Err` only on I/O failure. Adding `force` / `overwrite` flags is
   out of scope; callers compose with `write_atomic` instead.
4. **Public surface is six free functions, all in `lib.rs`.** No
   builder types, no trait objects, no async. Each function does one
   thing and returns `anyhow::Result<...>`. Growth happens via new
   crates in the family, not by enlarging this one.
5. **MSRV is Rust 1.75.0.** Bumping it is a minor-version event.
   Track via `rust-version` in `Cargo.toml` and the CI matrix.

## What "honest" looks like here

The crate prefers **failing loudly** over silent best-effort:

- Parent directory missing in `write_atomic` / `rename_atomic`?
  `Err` from the underlying `File::create` / `fs::rename`,
  propagated. Use `write_once_with_parents` if you want
  `mkdir -p` behavior explicitly. (Note: `write_once` itself
  currently does `create_dir_all(parent)` before delegating to
  `write_atomic`; that behaviour is a known wart, see
  `dev/decisions.md`.)
- Cross-device rename? `Err` from `fs::rename` (`EXDEV`); we
  propagate verbatim. No silent copy fallback.
- fsync on the parent fails? Re-raise. The file is visible but
  crash-durability is **not** confirmed and the caller learns that.
- Concurrent writers racing on `write_once`? Exactly one returns
  `Ok(true)`; the others return `Ok(false)`. No partial-state file
  ever appears on disk.

## Layout

```
src/lib.rs                    # six pub fn, ~120 LOC of source
tests/                        # (empty placeholder; integration tests pending — see dev/decisions.md)
examples/atomic_write.rs      # runnable demo
.github/workflows/ci.yml      # rust matrix (stable + MSRV) + dry-run + release
Cargo.toml                    # single-crate flat
LICENSE LICENSE-MIT LICENSE-APACHE  # dual-license
README.md                     # user-facing intro + API tour
AGENTS.md                     # this file
```

There is also a **private** companion directory at `dev/`
(gitignored) containing design notes, dogfood call-site catalogs,
and decisions. It is referenced by `dev/README.md` and is **not**
part of the published artefact.

## Toolchain pinning

- Rust: MSRV `1.75.0` in `Cargo.toml`. CI matrix runs `stable` and
  `1.75.0`.
- `clippy`, `rustfmt`, `cargo doc`: run only on `stable`. Clippy
  diagnostics shift between Rust releases; gating MSRV on
  `clippy -D warnings` would create noise unrelated to the MSRV
  contract. The MSRV job runs `cargo check` and `cargo test`.
- All GitHub Actions reference tagged releases via
  `dtolnay/rust-toolchain` / `actions/checkout` / `actions/cache`.

## The release flow

Short version:

1. Bump `version` in `Cargo.toml`. If MSRV changed, also bump
   `rust-version`.
2. Update `README.md` "MSRV" line if relevant.
3. Commit, push to `main`. Wait for CI to go green.
4. Tag: `git tag vX.Y.Z && git push origin vX.Y.Z`. The tag name
   minus the `v` prefix **must** match `Cargo.toml`'s version; the
   `release` job verifies this and fails loudly otherwise.
5. The tag triggers a fresh CI run including the `release` job,
   which runs `cargo publish` with `secrets.CARGO_REGISTRY_TOKEN`.
6. Verify: `cargo add safeatomic-rs@X.Y.Z` in a scratch crate,
   smoke-test the public API, check rendered docs on `docs.rs`.

No `release: published` event is needed; pushing the tag is the
trigger. To add manual approval before publish, create a
`crates-io` GitHub Environment and reference it from the `release`
job. The token continues to be a repo secret either way.

## Branch policy on `main`

There is **no enforced branch protection** on `main` (this is a
small one-author repo). Convention:

- Push regular work direct to `main`. CI runs on every push.
- Cut tags only from green commits.
- Force-pushes to `main` are not permitted by convention; recovery
  requires an explicit instruction from the owner.

If protection is added later (PR-required, status-checks gating),
update this section.

## Family of repos

`safeatomic-rs` is part of a small family of local persistence
primitives:

- [`safeatomic`](https://github.com/deepcausa/safeatomic) — Python
  sibling, broader surface (43 public names), with a fully
  documented eight-cell guarantee matrix and an ADR set. The
  primary reference for the design philosophy.
- [`datawal`](https://github.com/deepcausa/datawal) — Rust record
  store: framed append-only log with CRC32C, recovery semantics,
  and a last-write-wins KV projection. Uses `safeatomic-rs`
  internally for one-shot atomic writes.

Cross-linked from `README.md` § "Related projects".

## Don'ts

- Do not add `fsync=false` / `durable=false` / `lazy=true` flags to
  any function whose name promises durability.
- Do not introduce silent best-effort fallbacks (cross-device,
  mkdir, overwrite-on-conflict) under existing function names.
  Compose new explicit functions instead.
- Do not add async variants. The crate is synchronous by design;
  callers wrap with `tokio::task::spawn_blocking` when needed.
- Do not pull in heavyweight runtime dependencies (no `tokio`, no
  `serde`, no `tracing`). The dependency tree is intentionally tiny.
- Do not commit `dev/` or any local scratch directory. `.gitignore`
  already excludes it.

## Where to ask "is this in scope?"

- For new behaviour: open a GitHub issue with a one-paragraph
  rationale and the proposed function signature.
- For "I broke the build, how do I unbreak it?": the CI workflow at
  `.github/workflows/ci.yml` is the canonical local reproduction
  (`cargo fmt --check`, `cargo check --all-targets`, `cargo test
  --all-targets`).
