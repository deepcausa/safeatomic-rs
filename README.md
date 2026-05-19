# safeatomic-rs

Atomic POSIX filesystem primitives for Rust. Sibling of the Python
[`safeatomic`](https://pypi.org/project/safeatomic/) package.

`safeatomic-rs` is a small, focused crate: it provides the low-level
operations needed to write configs, manifests, checkpoints, segments, and
similar files **safely** on a local POSIX/Linux filesystem.

## What it does

```rust
use safeatomic_rs::{
    write_atomic, write_once, write_once_with_parents,
    write_append_fsync, rename_atomic, fsync_dir,
};
```

- `write_atomic(path, bytes)` — same-dir tmp + fsync + rename + fsync parent
- `write_once(path, bytes) -> bool` — refuses to overwrite
- `write_once_with_parents(path, bytes) -> bool` — `mkdir -p` + `write_once`
- `write_append_fsync(path, bytes)` — raw append + fsync (**primitive**)
- `rename_atomic(src, dst)` — rename + fsync parent
- `fsync_dir(path)` — fsync a directory inode

## What it is not

- **Not a write-ahead log.** `write_append_fsync` is a primitive. It does
  not provide record framing, checksums, or recovery semantics. For framed
  append-only logs see [`datawal`](../datawal/).
- **Not a database.** No keys, no indices, no transactions.
- **Not a file-locking library.** Advisory locks may land in a follow-up
  crate; this one is lock-free today.
- **Not portable to non-POSIX targets.** Linux/macOS/BSD assumptions.

## Relationship to Python `safeatomic`

`safeatomic-rs` is the Rust sibling of the Python `safeatomic` package.
The two share intent and surface (same six primitives, same guarantees on
POSIX) but the Rust side stands on its own — it is not a binding, not an
FFI wrapper.

## Status

`v0.1.0` — useful, stable in surface, intentionally minimal. The six
primitives above are unlikely to change shape; new primitives may be
added later.

## License

Apache-2.0.
