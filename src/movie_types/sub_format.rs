use rusqlite::{
    types::{FromSql, FromSqlResult, ValueRef},
    Result as RusqliteResult, ToSql,
};
// use serde::Serialize;
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
            "ASS" | "S_TEXT/ASS" => SubtitleFormat::ASS,
            "PGS" | "S_HDMV/PGS" => SubtitleFormat::PGS,
            "UTF8" | "S_TEXT/UTF8" => SubtitleFormat::SRT,
            "SSA" | "S_TEXT/SSA" => SubtitleFormat::SSA,
            "VOB" | "S_VOBSUB" => SubtitleFormat::VOB,
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

impl FromSql for SubtitleFormat {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(SubtitleFormat::from)
    }
}
