use rusqlite::{
    types::{FromSql, FromSqlResult, ValueRef},
    Result as RusqliteResult, ToSql,
};
use std::fmt;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum AudioCodec {
    AAC,
    AC3,
    Atmos, // same as TrueHD
    EAC3,
    DTS,
    FLAC,
    OPUS,
    PCM,
    Other(String),
}

impl From<&str> for AudioCodec {
    fn from(s: &str) -> Self {
        match s.trim_start_matches("A_") {
            "AAC" => AudioCodec::AAC,
            "AC3" => AudioCodec::AC3,
            "DTS" => AudioCodec::DTS,
            "EAC3" => AudioCodec::EAC3,
            "FLAC" => AudioCodec::FLAC,
            "OPUS" => AudioCodec::OPUS,
            "PCM" => AudioCodec::PCM,
            "Atmos" | "TRUEHD" => AudioCodec::Atmos,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .next()
                    .unwrap_or("Err");
                AudioCodec::Other(other.to_string())
            }
        }
    }
}

impl ToSql for AudioCodec {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            AudioCodec::AAC => "AAC".into(),
            AudioCodec::AC3 => "AC3".into(),
            AudioCodec::DTS => "DTS".into(),
            AudioCodec::EAC3 => "EAC3".into(),
            AudioCodec::FLAC => "FLAC".into(),
            AudioCodec::OPUS => "OPUS".into(),
            AudioCodec::PCM => "PCM".into(),
            AudioCodec::Atmos => "Atmos".into(),
            AudioCodec::Other(s) => s.as_str().into(),
        })
    }
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioCodec::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl FromSql for AudioCodec {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(AudioCodec::from)
    }
}
