use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DictumError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Strategic,
    Tactical,
    Operational,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Strategic => write!(f, "strategic"),
            Level::Tactical => write!(f, "tactical"),
            Level::Operational => write!(f, "operational"),
        }
    }
}

impl FromStr for Level {
    type Err = DictumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strategic" => Ok(Level::Strategic),
            "tactical" => Ok(Level::Tactical),
            "operational" => Ok(Level::Operational),
            _ => Err(DictumError::InvalidLevel(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Superseded,
    Deprecated,
    Draft,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Active => write!(f, "active"),
            Status::Superseded => write!(f, "superseded"),
            Status::Deprecated => write!(f, "deprecated"),
            Status::Draft => write!(f, "draft"),
        }
    }
}

impl FromStr for Status {
    type Err = DictumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Status::Active),
            "superseded" => Ok(Status::Superseded),
            "deprecated" => Ok(Status::Deprecated),
            "draft" => Ok(Status::Draft),
            _ => Err(DictumError::InvalidStatus(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub level: Level,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}
