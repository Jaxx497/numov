use lazy_static::lazy_static;
use matroska::{self, Matroska};
use regex::Regex;
use std::fmt::{Display, Formatter, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Movie {
    pub title: String,
    pub year: i16,
    pub duration: String,
    pub hash: u32,
    pub size: f32,
}

impl Movie {
    pub fn new(path: &PathBuf) -> Self {
        let matroska = Matroska::open(std::fs::File::open(path).unwrap()).unwrap();

        Self::collection(&matroska, path)
    }

    fn collection(matroska: &Matroska, path: &Path) -> Self {
        let (title, year) = Self::get_title_year(matroska, path).unwrap();

        let (byte_count, hash) = numov::read_metadata(path);

        let duration_raw = &matroska.info.duration.unwrap();
        let hours = duration_raw.as_secs() / 3600;
        let minutes = (duration_raw.as_secs() % 3600) / 60;

        let duration = format!("{}h {:02}min", hours, minutes);

        let size = numov::make_gb(byte_count);

        Movie {
            title,
            year,
            duration,
            hash,
            size,
        }
    }

    fn get_title_year(matroska: &Matroska, path: &Path) -> Option<(String, i16)> {
        let metadata_title = matroska.info.title.clone().unwrap_or_default();

        let parent = path
            .parent()
            .expect("Could not unwrap parent contents.")
            .file_name()
            .expect("Could not read parent folder name.")
            .to_str()?;

        Self::extract(&metadata_title)
            .or_else(|| {
                Self::extract(parent).map(|(title, year)| {
                    println!("TODO UPDATE METADATA FOR {}", title);
                    (title, year)
                })
            })
            .or_else(|| {
                println!(
                    "UNABLE TO PARSE TITLE INFO FOR {{ {:?} }}",
                    &path.file_name().unwrap()
                );
                None
            })
    }

    fn extract(str: &str) -> Option<(String, i16)> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?P<title>.*) \((?P<year>\d{4})\)").unwrap();
        }

        RE.captures(str).map(|captures| {
            let title = captures.get(1).unwrap().as_str().to_string();
            let year: i16 = captures.get(2).unwrap().as_str().parse().unwrap();
            (title, year)
        })
    }
}

impl Display for Movie {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{0} ({1}) [{4:x}]\n\t {3} | {2:.2} GB\n",
            self.title, self.year, self.size, self.duration, self.hash
        )
    }
}
