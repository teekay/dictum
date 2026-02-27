use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DictumError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    Refines,
    Supports,
    Supersedes,
    Conflicts,
    Requires,
    Entails,
    Excludes,
}

impl fmt::Display for LinkKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkKind::Refines => write!(f, "refines"),
            LinkKind::Supports => write!(f, "supports"),
            LinkKind::Supersedes => write!(f, "supersedes"),
            LinkKind::Conflicts => write!(f, "conflicts"),
            LinkKind::Requires => write!(f, "requires"),
            LinkKind::Entails => write!(f, "entails"),
            LinkKind::Excludes => write!(f, "excludes"),
        }
    }
}

impl FromStr for LinkKind {
    type Err = DictumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "refines" => Ok(LinkKind::Refines),
            "supports" => Ok(LinkKind::Supports),
            "supersedes" => Ok(LinkKind::Supersedes),
            "conflicts" => Ok(LinkKind::Conflicts),
            "requires" => Ok(LinkKind::Requires),
            "entails" => Ok(LinkKind::Entails),
            "excludes" => Ok(LinkKind::Excludes),
            _ => Err(DictumError::InvalidLinkKind(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub source_id: String,
    pub target_id: String,
    pub kind: LinkKind,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_kind_roundtrips() {
        let all = [
            LinkKind::Refines,
            LinkKind::Supports,
            LinkKind::Supersedes,
            LinkKind::Conflicts,
            LinkKind::Requires,
            LinkKind::Entails,
            LinkKind::Excludes,
        ];
        for variant in &all {
            let s = variant.to_string();
            let parsed: LinkKind = s.parse().unwrap();
            assert_eq!(&parsed, variant);
        }
    }
}
