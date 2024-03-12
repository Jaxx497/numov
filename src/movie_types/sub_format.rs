use rusqlite::{Result as RusqliteResult, ToSql};
use std::fmt;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum SubtitleFormat {
    ASS,
    PGS,
    SRT,
    SSA,
    VOB,
    Other(String),
}

impl From<&str> for SubtitleFormat {
    fn from(s: &str) -> Self {
        match s {
            "S_TEXT/ASS" => SubtitleFormat::ASS,
            "S_HDMV/PGS" => SubtitleFormat::PGS,
            "S_TEXT/UTF8" => SubtitleFormat::SRT,
            "S_TEXT/SSA" => SubtitleFormat::SSA,
            "S_VOBSUB" => SubtitleFormat::VOB,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .next()
                    .unwrap_or("Err");
                SubtitleFormat::Other(other.to_string())
            }
        }
    }
}

impl ToSql for SubtitleFormat {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            SubtitleFormat::ASS => "ASS".into(),
            SubtitleFormat::PGS => "PGS".into(),
            SubtitleFormat::SRT => "SRT".into(),
            SubtitleFormat::SSA => "SSA".into(),
            SubtitleFormat::VOB => "VOB".into(),
            SubtitleFormat::Other(s) => s.as_str().into(),
        })
    }
}

impl fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubtitleFormat::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}
