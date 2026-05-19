//! `safeatomic-rs` — atomic POSIX filesystem primitives.
//!
//! Rust sibling of the Python `safeatomic` package. Provides the small set
//! of low-level operations needed to safely write configs, manifests,
//! checkpoints, segments, and similar files on a local POSIX/Linux
//! filesystem.
//!
//! ## Scope
//!
//! - Same-directory temp file, fsync, rename, fsync parent directory.
//! - Write-once semantics (refuse to overwrite).
//! - Raw append + fsync.
//! - Atomic rename.
//! - Directory fsync.
//!
//! ## Non-scope
//!
//! - **Not** a write-ahead log. No record framing, no checksums, no recovery
//!   protocol. [`write_append_fsync`] is a primitive: it appends bytes and
//!   fsyncs. It does not give you durable record boundaries or crash recovery
//!   semantics. For that, use a framed log layer such as `datawal-core`.
//! - **Not** a database. No keys, indices, or transactions.
//! - **Not** a file-locking library. Advisory locks may land in a follow-up
//!   crate; today this crate is lock-free.
//! - **Not** a network/object store. Local POSIX assumptions only.
//!
//! ## Dependencies
//!
//! `std` + `anyhow`. No async runtime. No domain crates.
//!
//! ## Functions
//!
//! - [`write_atomic`]            tmp + fsync + rename + fsync parent dir
//! - [`write_once`]              same as `write_atomic` but only if absent
//! - [`write_append_fsync`]      `OpenOptions` create+append + fsync (raw)
//! - [`rename_atomic`]           rename + fsync parent dir
//! - [`fsync_dir`]               fsync a directory inode
//! - [`write_once_with_parents`] `mkdir -p` then [`write_once`]

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Write content atomically: write to `.tmp`, fsync, rename, fsync parent dir.
pub fn write_atomic(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp");
    let mut f = File::create(&tmp)?;
    f.write_all(content)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, path)?;
    if let Some(parent) = path.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

/// Write content atomically ONLY if file does not exist (write-once).
///
/// Returns `Ok(true)` if written, `Ok(false)` if file already exists.
pub fn write_once(path: &Path, content: &[u8]) -> anyhow::Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(path, content)?;
    Ok(true)
}

/// Append content and fsync.
///
/// This is a **primitive**: it appends bytes and calls `sync_all`. It does
/// **not** provide record framing, checksums, recovery, or transactional
/// semantics. For framed append-only logs use a higher-level layer such as
/// `datawal-core`.
pub fn write_append_fsync(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(content)?;
    f.sync_all()?;
    Ok(())
}

/// Atomic rename: rename then fsync the destination parent directory.
pub fn rename_atomic(from: &Path, to: &Path) -> anyhow::Result<()> {
    fs::rename(from, to)?;
    if let Some(parent) = to.parent() {
        fsync_dir(parent)?;
    }
    Ok(())
}

/// fsync a directory inode (POSIX).
pub fn fsync_dir(dir: &Path) -> anyhow::Result<()> {
    let f = File::open(dir)?;
    #[cfg(unix)]
    f.sync_all()?;
    #[cfg(not(unix))]
    let _ = f;
    Ok(())
}

/// Create parent directories if needed, then [`write_once`].
pub fn write_once_with_parents(path: &Path, content: &[u8]) -> anyhow::Result<bool> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_once(path, content)
}
