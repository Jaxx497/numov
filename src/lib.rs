use std::path::Path;
use xxhash_rust::const_xxh32::xxh32;

/// Given a path, will return a tuple of BYTE COUNT `(u64)` and HASH `(u32)`
pub fn read_metadata(path: &Path) -> (u64, u32) {
    let bytes = std::fs::metadata(path)
        .expect("Could not read files metadata.")
        .len();

    let last_mod = path
        .metadata()
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Could not convert to timestamp.")
        .as_nanos();

    (bytes, xxh32(&(bytes as u128 + last_mod).to_be_bytes(), 0))
}

pub fn make_gb(bytes: u64) -> f32 {
    ["B", "KB", "MB", "GB"]
        .iter()
        .fold(bytes as f32, |acc, _| match acc > 1024.0 {
            true => acc / 1024.0,
            false => acc,
        })
}
