pub const CREATE_DECISIONS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS decisions (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT,
    level TEXT NOT NULL CHECK(level IN ('strategic', 'tactical', 'operational')),
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'superseded', 'deprecated', 'draft')),
    superseded_by TEXT,
    author TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)";

pub const CREATE_LINKS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS links (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('refines', 'supports', 'supersedes', 'conflicts', 'requires')),
    created_at TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id, kind),
    FOREIGN KEY (source_id) REFERENCES decisions(id),
    FOREIGN KEY (target_id) REFERENCES decisions(id)
)";

pub const CREATE_LABELS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS labels (
    decision_id TEXT NOT NULL,
    label TEXT NOT NULL,
    PRIMARY KEY (decision_id, label),
    FOREIGN KEY (decision_id) REFERENCES decisions(id)
)";
