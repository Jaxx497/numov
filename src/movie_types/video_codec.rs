use rusqlite::{
    types::{FromSql, ToSql},
    Result as RusqliteResult,
};
use std::fmt;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum VideoCodec {
    x264,
    x265,
    Other(String),
}

impl From<&str> for VideoCodec {
    fn from(s: &str) -> Self {
        match s {
            "V_MPEGH/ISO/HEVC" => VideoCodec::x265,
            "V_MPEG4/ISO/AVC" => VideoCodec::x264,
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
            VideoCodec::x265 => "x265".into(),
            VideoCodec::x264 => "x264".into(),
            VideoCodec::Other(s) => s.as_str().into(),
        })
    }
}
//
// impl FromSql for VideoCodec {
//     fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
//             value.as_str().and_then(|s| {
//             VideoCodec::from_str(s) )
//         })
//     }
// }
