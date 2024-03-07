use matroska::{self, Matroska};
use regex::Regex;
use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Movie {
    pub title: String,
    pub year: i16,
    pub hash: u32,
    pub size: f32,
}

impl Movie {
    pub fn new(path: &PathBuf) -> Self {
        let (byte_count, hash) = numov::read_metadata(path);
        let matroska = Matroska::open(std::fs::File::open(path).unwrap()).unwrap();

        let (title, year) = match &matroska.info.title {
            Some(t) if Regex::new(r".*\(\d{4}\)$").unwrap().is_match(t) => {
                Self::unwrap_title_year(t)
            }
            // _ => _set_title_year(path),
            _ => (String::from("TITLE"), 2000),
        };

        Movie {
            title,
            year,
            hash,
            size: numov::make_gb(byte_count),
        }
    }

    fn unwrap_title_year(parent: &str) -> (String, i16) {
        let title = Self::extract_match(parent, r"^(.*?) \(").unwrap();

        let year = Self::extract_match(parent, r"\((.*?)\)")
            .unwrap()
            .parse::<i16>()
            .expect("Year could not be parsed.");

        (title, year)
    }

    fn extract_match(text: &str, pattern: &str) -> Option<String> {
        let regex = Regex::new(pattern).unwrap();
        regex
            .captures(text)
            .and_then(|capture| capture.get(1).map(|m| m.as_str().to_string()))
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
