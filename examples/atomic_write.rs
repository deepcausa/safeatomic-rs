//! Demonstrates the six primitives exposed by `safeatomic-rs`.
//!
//! Run with:
//!     cargo run --example atomic_write

use std::fs;
use std::path::PathBuf;

use safeatomic_rs::{
    fsync_dir, rename_atomic, write_append_fsync, write_atomic, write_once, write_once_with_parents,
};

fn main() -> anyhow::Result<()> {
    // Isolated playground.
    let root = std::env::temp_dir().join("safeatomic-rs-demo");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    println!("[demo] root = {}", root.display());

    // 1. write_atomic: tmp + fsync + rename + fsync(parent).
    let a: PathBuf = root.join("a.bin");
    write_atomic(&a, b"hello\n")?;
    println!(
        "[1] write_atomic   -> {} ({} bytes)",
        a.display(),
        fs::metadata(&a)?.len()
    );

    // 2. write_once: succeeds first time, refuses second time.
    let b = root.join("b.bin");
    let first = write_once(&b, b"first\n")?;
    let second = write_once(&b, b"second\n")?;
    println!(
        "[2] write_once     -> first={first} second={second} content={:?}",
        String::from_utf8(fs::read(&b)?)?
    );

    // 3. write_append_fsync: raw durable append. NOT a framed log.
    let log = root.join("c.log");
    write_append_fsync(&log, b"line-1\n")?;
    write_append_fsync(&log, b"line-2\n")?;
    write_append_fsync(&log, b"line-3\n")?;
    println!(
        "[3] append_fsync   -> {} bytes:\n{}",
        fs::metadata(&log)?.len(),
        String::from_utf8(fs::read(&log)?)?.trim_end()
    );

    // 4. rename_atomic: rename + fsync(parent dir).
    let renamed = root.join("a.renamed.bin");
    rename_atomic(&a, &renamed)?;
    println!(
        "[4] rename_atomic  -> {} (exists={})",
        renamed.display(),
        renamed.exists()
    );

    // 5. fsync_dir: just flush the directory inode.
    fsync_dir(&root)?;
    println!("[5] fsync_dir      -> ok");

    // 6. write_once_with_parents: mkdir -p then write_once.
    let deep = root.join("nested/dir/d.bin");
    let wrote = write_once_with_parents(&deep, b"deep\n")?;
    println!(
        "[6] write_once_with_parents -> wrote={wrote} path={}",
        deep.display()
    );

    println!("[demo] OK");
    Ok(())
}
