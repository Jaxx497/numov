use rusqlite::{
    types::{FromSql, FromSqlResult, ValueRef},
    Result as RusqliteResult, ToSql,
};
// use serde::Serialize;
use std::fmt;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum VideoCodec {
    x264,
    x265,
    AV1,
    Other(String),
}

impl From<&str> for VideoCodec {
    fn from(s: &str) -> Self {
        match s {
            "AV1" | "V_AV1" => VideoCodec::AV1,
            "x264" | "V_MPEG4/ISO/AVC" => VideoCodec::x264,
            "x265" | "V_MPEGH/ISO/HEVC" => VideoCodec::x265,
            _ => {
                let other = s
                    .split('_')
                    .last()
                    .unwrap_or("Err")
                    .split('/')
                    .last()
                    .unwrap_or("Err");
                VideoCodec::Other(other.to_string())
            }
        }
    }
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoCodec::Other(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl ToSql for VideoCodec {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            VideoCodec::AV1 => "AV1".into(),
            VideoCodec::x265 => "x265".into(),
            VideoCodec::x264 => "x264".into(),
            VideoCodec::Other(s) => s.as_str().into(),
        })
    }
}

impl FromSql for VideoCodec {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(VideoCodec::from)
    }
}
