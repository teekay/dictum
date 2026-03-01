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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Principle,
    Constraint,
    Assumption,
    Choice,
    Rule,
    Goal,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Principle => write!(f, "principle"),
            Kind::Constraint => write!(f, "constraint"),
            Kind::Assumption => write!(f, "assumption"),
            Kind::Choice => write!(f, "choice"),
            Kind::Rule => write!(f, "rule"),
            Kind::Goal => write!(f, "goal"),
        }
    }
}

impl FromStr for Kind {
    type Err = DictumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "principle" => Ok(Kind::Principle),
            "constraint" => Ok(Kind::Constraint),
            "assumption" => Ok(Kind::Assumption),
            "choice" => Ok(Kind::Choice),
            "rule" => Ok(Kind::Rule),
            "goal" => Ok(Kind::Goal),
            _ => Err(DictumError::InvalidKind(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Weight {
    Must,
    Should,
    May,
}

impl fmt::Display for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Weight::Must => write!(f, "must"),
            Weight::Should => write!(f, "should"),
            Weight::May => write!(f, "may"),
        }
    }
}

impl FromStr for Weight {
    type Err = DictumError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "must" => Ok(Weight::Must),
            "should" => Ok(Weight::Should),
            "may" => Ok(Weight::May),
            _ => Err(DictumError::InvalidWeight(s.to_string())),
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
    #[serde(default = "default_kind")]
    pub kind: Kind,
    #[serde(default = "default_weight")]
    pub weight: Weight,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuttal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

fn default_kind() -> Kind {
    Kind::Choice
}

fn default_weight() -> Weight {
    Weight::Should
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_roundtrips() {
        let all = [
            Kind::Principle,
            Kind::Constraint,
            Kind::Assumption,
            Kind::Choice,
            Kind::Rule,
            Kind::Goal,
        ];
        for variant in &all {
            let s = variant.to_string();
            let parsed: Kind = s.parse().unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn weight_roundtrips() {
        let all = [Weight::Must, Weight::Should, Weight::May];
        for variant in &all {
            let s = variant.to_string();
            let parsed: Weight = s.parse().unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn invalid_kind_errors() {
        assert!("banana".parse::<Kind>().is_err());
    }

    #[test]
    fn invalid_weight_errors() {
        assert!("banana".parse::<Weight>().is_err());
    }
}
