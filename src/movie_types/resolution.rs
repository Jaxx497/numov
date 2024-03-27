use rusqlite::{
    types::{FromSql, FromSqlResult, ValueRef},
    Result as RusqliteResult, ToSql,
};
// use serde::Serialize;
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Default)]
pub enum Resolution {
    SD,
    HD720,
    HD1080,
    UHD4K,
    UHD8K,
    #[default]
    Err,
}

impl From<u64> for Resolution {
    fn from(height: u64) -> Self {
        match height {
            h if h < 480 => Resolution::SD,
            h if h <= 720 => Resolution::HD720,
            h if h <= 1080 => Resolution::HD1080,
            h if h <= 2160 => Resolution::UHD4K,
            _ => Resolution::UHD8K,
        }
    }
}

impl FromStr for Resolution {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "SD" => Ok(Resolution::SD),
            "720p" => Ok(Resolution::HD720),
            "1080p" => Ok(Resolution::HD1080),
            "2160p" => Ok(Resolution::UHD4K),
            "8K" => Ok(Resolution::UHD8K),
            _ => Ok(Resolution::Err),
        }
    }
}

impl From<&str> for Resolution {
    fn from(s: &str) -> Self {
        match s {
            "SD" => Resolution::SD,
            "720p" => Resolution::HD720,
            "1080p" => Resolution::HD1080,
            "2160p" => Resolution::UHD4K,
            "8K" => Resolution::UHD8K,
            _ => Resolution::Err,
        }
    }
}

// impl ToString for Resolution {
//     fn to_string(&self) -> String {
//         match self {
//             Resolution::SD => String::from("SD"),
//             Resolution::HD720 => String::from("720p"),
//             Resolution::HD1080 => String::from("1080p"),
//             Resolution::UHD4K => String::from("4K"),
//             Resolution::UHD8K => String::from("8K"),
//             Resolution::Err => String::from("Err"),
//         }
//     }
// }

impl Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolution::SD => write!(f, "SD"),
            Resolution::HD720 => write!(f, "720p"),
            Resolution::HD1080 => write!(f, "1080p"),
            Resolution::UHD4K => write!(f, "2160p"),
            Resolution::UHD8K => write!(f, "8K"),
            Resolution::Err => write!(f, "Err"),
        }
    }
}

impl ToSql for Resolution {
    fn to_sql(&self) -> RusqliteResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.to_string().into())
    }
}

impl FromSql for Resolution {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().map(Resolution::from)
    }
}
