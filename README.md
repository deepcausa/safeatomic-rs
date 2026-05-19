# safeatomic-rs

[![Crates.io](https://img.shields.io/crates/v/safeatomic-rs.svg)](https://crates.io/crates/safeatomic-rs)
[![Docs.rs](https://docs.rs/safeatomic-rs/badge.svg)](https://docs.rs/safeatomic-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Atomic POSIX filesystem primitives for Rust. Sibling of the Python
[`safeatomic`](https://pypi.org/project/safeatomic/) package.

**MSRV:** Rust 1.75.0

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

## Related projects

- [`safeatomic`](https://github.com/deepcausa/safeatomic) — Python package
  for plain-file persistence with explicit guarantees, runtime diagnostics,
  checksums, cooperative locks, and small TLA+ protocol models.
- [`datawal`](https://github.com/deepcausa/datawal) — Rust record store that
  uses filesystem primitives in the same spirit: framed append-only records,
  recovery, a bytes-based KV projection, tombstones, compaction, and clean
  export.

`safeatomic-rs` is the Rust sibling of `safeatomic`, not a binding and not
an API mirror. It provides low-level filesystem primitives that can be used
by higher-level crates such as `datawal`.

## Status

`v0.1.0` — useful, stable in surface, intentionally minimal. The six
primitives above are unlikely to change shape; new primitives may be
added later.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

SPDX-License-Identifier: `MIT OR Apache-2.0`

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
