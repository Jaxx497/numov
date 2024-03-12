use rusqlite::{Result as RusqliteResult, ToSql};
use std::fmt;

#[derive(Debug)]
pub enum BitDepth {
    Bit10,
    Bit8,
    Other(i8),
}

impl ToSql for BitDepth {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            BitDepth::Bit10 => "10bit".into(),
            BitDepth::Bit8 => "8bit".into(),
            BitDepth::Other(s) => format!("{}bit", s).into(),
        })
    }
}

impl fmt::Display for BitDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BitDepth::Bit10 => write!(f, "10bit"),
            BitDepth::Bit8 => write!(f, "8bit"),
            _ => write!(f, "ERR"),
        }
    }
}

impl From<&i8> for BitDepth {
    fn from(bits: &i8) -> Self {
        match bits {
            10 => BitDepth::Bit10,
            8 => BitDepth::Bit8,
            i => BitDepth::Other(*i),
        }
    }
}
