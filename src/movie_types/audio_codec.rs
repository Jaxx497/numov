use rusqlite::{types::ToSql, Result as RusqliteResult};
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
    Other(String),
}

impl From<&str> for AudioCodec {
    fn from(s: &str) -> Self {
        match s {
            "A_AAC" => AudioCodec::AAC,
            "A_AC3" => AudioCodec::AC3,
            "A_DTS" => AudioCodec::DTS,
            "A_EAC3" => AudioCodec::EAC3,
            "A_FLAC" => AudioCodec::FLAC,
            "A_OPUS" => AudioCodec::OPUS,
            "A_TRUEHD" => AudioCodec::Atmos,
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
