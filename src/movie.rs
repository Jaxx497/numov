use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;
use xxhash_rust::const_xxh32::xxh32;

#[derive(Debug)]
pub struct Movie {
    pub title: PathBuf,
    // pub year: i32,
    pub hash: usize,
    pub size: f32,
}

impl Movie {
    pub fn new(path: PathBuf) -> Self {
        let (size, hash) = Self::_get_size_hash(&path);

        Movie {
            title: path,
            hash,
            size,
        }
    }

    fn _get_size_hash(path: &PathBuf) -> (f32, usize) {
        let bytes = std::fs::metadata(path)
            .expect("Could not read file's metadata.")
            .len();

        let readable =
            ["B", "KB", "MB", "GB"]
                .iter()
                .fold(bytes as f32, |acc, _| match acc > 1024.0 {
                    true => acc / 1024.0,
                    false => acc,
                });

        let last_mod = path
            .metadata()
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Could not convert to timestamp.")
            .as_nanos();

        let hash = xxh32(&(bytes as u128 + last_mod).to_be_bytes(), 0);

        (readable, hash as usize)
    }
}

impl Display for Movie {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Title: {:?}\nSize: {:.2} GB\nHash: {:x}\n",
            self.title, self.size, self.hash
        )
    }
}
