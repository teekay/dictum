use thiserror::Error;

#[derive(Error, Debug)]
pub enum DictumError {
    #[error("not initialized — run `dictum init` first")]
    NotInitialized,

    #[error("already initialized in {0}")]
    AlreadyInitialized(String),

    #[error("decision not found: {0}")]
    DecisionNotFound(String),

    #[error("invalid level: {0} (expected strategic, tactical, or operational)")]
    InvalidLevel(String),

    #[error("invalid status: {0} (expected active, superseded, deprecated, or draft)")]
    InvalidStatus(String),

    #[error("invalid link kind: {0}")]
    InvalidLinkKind(String),

    #[error("link already exists")]
    LinkAlreadyExists,

    #[error("link not found")]
    LinkNotFound,

    #[error("cannot link a decision to itself")]
    SelfLink,

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, DictumError>;
