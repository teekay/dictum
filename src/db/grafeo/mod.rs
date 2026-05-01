use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use grafeo::{Config, GrafeoDB, Value};

use crate::db::store::{ListFilter, Neighborhood, Store};
use crate::error::{DictumError, Result};
use crate::model::{Decision, Kind, Level, Link, LinkKind, Status, Weight};

pub struct GrafeoStore {
    db: GrafeoDB,
}

impl GrafeoStore {
    pub fn open(dictum_dir: &Path) -> Result<Self> {
        let db_path = dictum_dir.join("dictum.grafeo");
        let config = Config::persistent(&db_path);
        let db = GrafeoDB::with_config(config)?;
        let store = GrafeoStore { db };
        store.ensure_text_indexes();
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let db = GrafeoDB::new_in_memory();
        let store = GrafeoStore { db };
        store.ensure_text_indexes();
        Ok(store)
    }

    fn ensure_text_indexes(&self) {
        // Best-effort: create text indexes for decision_search
        let _ = self.db.create_text_index("Decision", "title");
        let _ = self.db.create_text_index("Decision", "body");
        let _ = self.db.create_text_index("Decision", "rebuttal");
        let _ = self.db.create_text_index("Decision", "scope");
    }

    fn session(&self) -> grafeo::Session {
        self.db.session()
    }

    fn load_labels(&self, decision_id: &str) -> Result<Vec<String>> {
        let session = self.session();
        let result = session.execute_with_params(
            "MATCH (:Decision {id: $id})-[:HAS_LABEL]->(l:Label) RETURN l.name ORDER BY l.name",
            params(&[("id", Value::from(decision_id))]),
        )?;
        let mut labels = Vec::new();
        for row in result.iter() {
            if let Some(name) = row[0].as_str() {
                labels.push(name.to_string());
            }
        }
        Ok(labels)
    }

    fn row_to_decision(&self, row: &[Value]) -> Result<Decision> {
        let id = row[0].as_str().unwrap_or("").to_string();
        let labels = self.load_labels(&id)?;
        Ok(Decision {
            id,
            title: row[1].as_str().unwrap_or("").to_string(),
            body: row[2].as_str().map(|s| s.to_string()),
            level: row[3]
                .as_str()
                .unwrap_or("tactical")
                .parse()
                .unwrap_or(Level::Tactical),
            status: row[4]
                .as_str()
                .unwrap_or("active")
                .parse()
                .unwrap_or(Status::Active),
            superseded_by: row[5].as_str().map(|s| s.to_string()),
            author: row[6].as_str().unwrap_or("").to_string(),
            created_at: row[7].as_str().unwrap_or("").to_string(),
            updated_at: row[8].as_str().unwrap_or("").to_string(),
            kind: row[9]
                .as_str()
                .unwrap_or("choice")
                .parse()
                .unwrap_or(Kind::Choice),
            weight: row[10]
                .as_str()
                .unwrap_or("should")
                .parse()
                .unwrap_or(Weight::Should),
            rebuttal: row[11].as_str().map(|s| s.to_string()),
            scope: row[12].as_str().map(|s| s.to_string()),
            labels,
        })
    }
}

const DECISION_COLS: &str =
    "d.id, d.title, d.body, d.level, d.status, d.superseded_by, \
     d.author, d.created_at, d.updated_at, d.kind, d.weight, d.rebuttal, d.scope";

impl Store for GrafeoStore {
    fn decision_insert(&mut self, decision: &Decision) -> Result<()> {
        let session = self.session();

        // Check for duplicate (no unique constraint in LPG)
        let check = session.execute_with_params(
            "MATCH (d:Decision {id: $id}) RETURN d.id",
            params(&[("id", Value::from(decision.id.as_str()))]),
        )?;
        if check.row_count() > 0 {
            return Err(DictumError::DecisionAlreadyExists);
        }

        session.execute_with_params(
            "INSERT (:Decision {
                id: $id, title: $title, body: $body, level: $level,
                status: $status, superseded_by: $superseded_by, author: $author,
                created_at: $created_at, updated_at: $updated_at,
                kind: $kind, weight: $weight, rebuttal: $rebuttal, scope: $scope
            })",
            params(&[
                ("id", Value::from(decision.id.as_str())),
                ("title", Value::from(decision.title.as_str())),
                ("body", opt_value(&decision.body)),
                ("level", Value::from(decision.level.to_string().as_str())),
                ("status", Value::from(decision.status.to_string().as_str())),
                ("superseded_by", opt_value(&decision.superseded_by)),
                ("author", Value::from(decision.author.as_str())),
                ("created_at", Value::from(decision.created_at.as_str())),
                ("updated_at", Value::from(decision.updated_at.as_str())),
                ("kind", Value::from(decision.kind.to_string().as_str())),
                ("weight", Value::from(decision.weight.to_string().as_str())),
                ("rebuttal", opt_value(&decision.rebuttal)),
                ("scope", opt_value(&decision.scope)),
            ]),
        )?;
        Ok(())
    }

    fn decision_get(&self, id: &str) -> Result<Decision> {
        let session = self.session();
        let query = format!("MATCH (d:Decision {{id: $id}}) RETURN {}", DECISION_COLS);
        let result = session.execute_with_params(
            &query,
            params(&[("id", Value::from(id))]),
        )?;
        if result.row_count() == 0 {
            return Err(DictumError::DecisionNotFound(id.to_string()));
        }
        self.row_to_decision(&result.rows[0])
    }

    fn decision_list(&self, filter: &ListFilter) -> Result<Vec<Decision>> {
        let session = self.session();

        let mut conditions = Vec::new();
        let mut param_pairs: Vec<(String, Value)> = Vec::new();

        if let Some(ref level) = filter.level {
            conditions.push("d.level = $f_level".to_string());
            param_pairs.push(("f_level".to_string(), Value::from(level.to_string().as_str())));
        }
        if let Some(ref status) = filter.status {
            conditions.push("d.status = $f_status".to_string());
            param_pairs.push(("f_status".to_string(), Value::from(status.to_string().as_str())));
        }
        if let Some(ref kind) = filter.kind {
            conditions.push("d.kind = $f_kind".to_string());
            param_pairs.push(("f_kind".to_string(), Value::from(kind.to_string().as_str())));
        }
        if let Some(ref weight) = filter.weight {
            conditions.push("d.weight = $f_weight".to_string());
            param_pairs.push(("f_weight".to_string(), Value::from(weight.to_string().as_str())));
        }
        if let Some(ref scope) = filter.scope {
            conditions.push("d.scope = $f_scope".to_string());
            param_pairs.push(("f_scope".to_string(), Value::from(scope.as_str())));
        }

        let match_clause = if let Some(ref label) = filter.label {
            param_pairs.push(("f_label".to_string(), Value::from(label.as_str())));
            "MATCH (d:Decision)-[:HAS_LABEL]->(l:Label {name: $f_label})".to_string()
        } else {
            "MATCH (d:Decision)".to_string()
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            "{}{} RETURN {} ORDER BY d.created_at DESC",
            match_clause, where_clause, DECISION_COLS,
        );

        let param_map: HashMap<String, Value> = param_pairs.into_iter().collect();
        let result = session.execute_with_params(&query, param_map)?;

        let mut decisions = Vec::new();
        for row in result.iter() {
            decisions.push(self.row_to_decision(row)?);
        }
        Ok(decisions)
    }

    fn decision_update_status(
        &mut self,
        id: &str,
        status: &Status,
        superseded_by: Option<&str>,
    ) -> Result<()> {
        let session = self.session();

        // Verify existence
        let check = session.execute_with_params(
            "MATCH (d:Decision {id: $id}) RETURN d.id",
            params(&[("id", Value::from(id))]),
        )?;
        if check.row_count() == 0 {
            return Err(DictumError::DecisionNotFound(id.to_string()));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let superseded_val = match superseded_by {
            Some(s) => Value::from(s),
            None => Value::Null,
        };

        session.execute_with_params(
            "MATCH (d:Decision {id: $id}) \
             SET d.status = $status, d.superseded_by = $superseded_by, d.updated_at = $updated_at",
            params(&[
                ("id", Value::from(id)),
                ("status", Value::from(status.to_string().as_str())),
                ("superseded_by", superseded_val),
                ("updated_at", Value::from(now.as_str())),
            ]),
        )?;
        Ok(())
    }

    fn decision_search(&self, query: &str) -> Result<Vec<Decision>> {
        // Use text_search API across indexed properties, merge results by ID
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut decisions = Vec::new();

        for property in &["title", "body", "rebuttal", "scope"] {
            if let Ok(hits) = self.db.text_search("Decision", property, query, 1000) {
                for (node_id, _score) in hits {
                    // Resolve the node to get its id property
                    let session = self.session();
                    let result = session.execute_with_params(
                        &format!("MATCH (d:Decision) WHERE id(d) = $nid RETURN {}", DECISION_COLS),
                        params(&[("nid", Value::from(node_id.0 as i64))]),
                    );
                    if let Ok(r) = result {
                        for row in r.iter() {
                            if let Some(id) = row[0].as_str() {
                                if seen_ids.insert(id.to_string()) {
                                    if let Ok(d) = self.row_to_decision(row) {
                                        decisions.push(d);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback if no text indexes exist yet or no results: property scan
        if decisions.is_empty() {
            let session = self.session();
            let result = session.execute_with_params(
                &format!(
                    "MATCH (d:Decision) \
                     WHERE d.title CONTAINS $q OR d.body CONTAINS $q \
                     OR d.rebuttal CONTAINS $q OR d.scope CONTAINS $q \
                     RETURN {} ORDER BY d.created_at DESC",
                    DECISION_COLS
                ),
                params(&[("q", Value::from(query))]),
            )?;
            for row in result.iter() {
                decisions.push(self.row_to_decision(row)?);
            }
        }

        Ok(decisions)
    }

    fn label_add(&mut self, decision_id: &str, label: &str) -> Result<()> {
        let session = self.session();

        // Ensure Label node exists
        let check = session.execute_with_params(
            "MATCH (l:Label {name: $name}) RETURN l.name",
            params(&[("name", Value::from(label))]),
        )?;
        if check.row_count() == 0 {
            session.execute_with_params(
                "INSERT (:Label {name: $name})",
                params(&[("name", Value::from(label))]),
            )?;
        }

        // Ensure HAS_LABEL edge doesn't already exist
        let p = params(&[
            ("did", Value::from(decision_id)),
            ("name", Value::from(label)),
        ]);
        let check = session.execute_with_params(
            "MATCH (:Decision {id: $did})-[:HAS_LABEL]->(:Label {name: $name}) RETURN 1",
            p.clone(),
        )?;
        if check.row_count() == 0 {
            session.execute_with_params(
                "MATCH (d:Decision {id: $did}), (l:Label {name: $name}) \
                 INSERT (d)-[:HAS_LABEL]->(l)",
                p,
            )?;
        }
        Ok(())
    }

    fn link_insert(&mut self, link: &Link) -> Result<()> {
        if link.source_id == link.target_id {
            return Err(DictumError::SelfLink);
        }

        let session = self.session();

        // Check for duplicate
        let check = session.execute_with_params(
            "MATCH (:Decision {id: $src})-[r:LINK]->(:Decision {id: $tgt}) \
             WHERE r.kind = $kind RETURN 1",
            params(&[
                ("src", Value::from(link.source_id.as_str())),
                ("tgt", Value::from(link.target_id.as_str())),
                ("kind", Value::from(link.kind.to_string().as_str())),
            ]),
        )?;
        if check.row_count() > 0 {
            return Err(DictumError::LinkAlreadyExists);
        }

        session.execute_with_params(
            "MATCH (s:Decision {id: $src}), (t:Decision {id: $tgt}) \
             INSERT (s)-[:LINK {kind: $kind, created_at: $created_at, reason: $reason}]->(t)",
            params(&[
                ("src", Value::from(link.source_id.as_str())),
                ("tgt", Value::from(link.target_id.as_str())),
                ("kind", Value::from(link.kind.to_string().as_str())),
                ("created_at", Value::from(link.created_at.as_str())),
                ("reason", opt_value(&link.reason)),
            ]),
        )?;
        Ok(())
    }

    fn link_delete(&mut self, source_id: &str, kind: &LinkKind, target_id: &str) -> Result<()> {
        let session = self.session();

        let check = session.execute_with_params(
            "MATCH (:Decision {id: $src})-[r:LINK]->(:Decision {id: $tgt}) \
             WHERE r.kind = $kind RETURN 1",
            params(&[
                ("src", Value::from(source_id)),
                ("tgt", Value::from(target_id)),
                ("kind", Value::from(kind.to_string().as_str())),
            ]),
        )?;
        if check.row_count() == 0 {
            return Err(DictumError::LinkNotFound);
        }

        session.execute_with_params(
            "MATCH (:Decision {id: $src})-[r:LINK]->(:Decision {id: $tgt}) \
             WHERE r.kind = $kind DELETE r",
            params(&[
                ("src", Value::from(source_id)),
                ("tgt", Value::from(target_id)),
                ("kind", Value::from(kind.to_string().as_str())),
            ]),
        )?;
        Ok(())
    }

    fn links_for_decision(&self, decision_id: &str) -> Result<Vec<Link>> {
        let session = self.session();
        let mut links = Vec::new();

        // Outbound
        let result = session.execute_with_params(
            "MATCH (d:Decision {id: $id})-[r:LINK]->(t:Decision) \
             RETURN d.id, t.id, r.kind, r.created_at, r.reason \
             ORDER BY r.created_at",
            params(&[("id", Value::from(decision_id))]),
        )?;
        for row in result.iter() {
            links.push(row_to_link(row));
        }

        // Inbound
        let result = session.execute_with_params(
            "MATCH (s:Decision)-[r:LINK]->(d:Decision {id: $id}) \
             RETURN s.id, d.id, r.kind, r.created_at, r.reason \
             ORDER BY r.created_at",
            params(&[("id", Value::from(decision_id))]),
        )?;
        for row in result.iter() {
            links.push(row_to_link(row));
        }

        links.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(links)
    }

    fn links_of_kind(&self, kind: &LinkKind) -> Result<Vec<(String, String)>> {
        let session = self.session();
        let result = session.execute_with_params(
            "MATCH (s:Decision)-[r:LINK]->(t:Decision) \
             WHERE r.kind = $kind \
             RETURN s.id, t.id ORDER BY r.created_at",
            params(&[("kind", Value::from(kind.to_string().as_str()))]),
        )?;
        let mut out = Vec::new();
        for row in result.iter() {
            out.push((
                row[0].as_str().unwrap_or("").to_string(),
                row[1].as_str().unwrap_or("").to_string(),
            ));
        }
        Ok(out)
    }

    fn neighborhood(&self, id: &str, depth: u32) -> Result<Neighborhood> {
        // BFS — same algorithm as SQLite backend, just using trait methods
        let mut visited_ids: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        visited_ids.insert(id.to_string());
        queue.push_back((id.to_string(), 0));

        let mut all_links: Vec<Link> = Vec::new();

        while let Some((current_id, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }
            let node_links = self.links_for_decision(&current_id)?;
            for link in &node_links {
                let neighbor = if link.source_id == current_id {
                    &link.target_id
                } else {
                    &link.source_id
                };
                if visited_ids.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), current_depth + 1));
                }
            }
            all_links.extend(node_links);
        }

        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        all_links.retain(|l| {
            seen.insert((l.source_id.clone(), l.target_id.clone(), l.kind.to_string()))
        });

        let mut decisions = Vec::new();
        for node_id in &visited_ids {
            decisions.push(self.decision_get(node_id)?);
        }
        Ok(Neighborhood {
            decisions,
            links: all_links,
        })
    }

    fn reachable(&self, id: &str, kinds: &[LinkKind]) -> Result<Vec<String>> {
        let mut all_edges: Vec<(String, String)> = Vec::new();
        for kind in kinds {
            all_edges.extend(self.links_of_kind(kind)?);
        }

        let mut visited: Vec<String> = Vec::new();
        let mut visited_set: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(id.to_string());
        visited_set.insert(id.to_string());

        while let Some(current) = queue.pop_front() {
            visited.push(current.clone());
            for (src, tgt) in &all_edges {
                if src == &current && visited_set.insert(tgt.clone()) {
                    queue.push_back(tgt.clone());
                }
            }
        }

        Ok(visited.into_iter().skip(1).collect())
    }
}

// --- Helpers ---

fn opt_value(opt: &Option<String>) -> Value {
    match opt {
        Some(s) => Value::from(s.as_str()),
        None => Value::Null,
    }
}

fn params(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

fn row_to_link(row: &[Value]) -> Link {
    Link {
        source_id: row[0].as_str().unwrap_or("").to_string(),
        target_id: row[1].as_str().unwrap_or("").to_string(),
        kind: row[2]
            .as_str()
            .unwrap_or("supports")
            .parse()
            .unwrap_or(LinkKind::Supports),
        created_at: row[3].as_str().unwrap_or("").to_string(),
        reason: row[4].as_str().map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::store::Store;

    fn make_store() -> GrafeoStore {
        GrafeoStore::in_memory().unwrap()
    }

    fn make_decision(id: &str, kind: Kind, weight: Weight, scope: Option<&str>) -> Decision {
        Decision {
            id: id.to_string(),
            title: format!("Decision {}", id),
            body: None,
            level: Level::Tactical,
            status: Status::Active,
            superseded_by: None,
            author: "test".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            labels: Vec::new(),
            kind,
            weight,
            rebuttal: None,
            scope: scope.map(|s| s.to_string()),
        }
    }

    #[test]
    fn insert_and_get() {
        let mut store = make_store();
        let d = Decision {
            body: Some("test body".to_string()),
            rebuttal: Some("unless X".to_string()),
            scope: Some("auth".to_string()),
            ..make_decision("d-1", Kind::Rule, Weight::Must, None)
        };
        store.decision_insert(&d).unwrap();
        let got = store.decision_get("d-1").unwrap();
        assert_eq!(got.id, "d-1");
        assert_eq!(got.title, "Decision d-1");
        assert_eq!(got.body.as_deref(), Some("test body"));
        assert_eq!(got.kind, Kind::Rule);
        assert_eq!(got.weight, Weight::Must);
        assert_eq!(got.rebuttal.as_deref(), Some("unless X"));
        assert_eq!(got.scope.as_deref(), Some("auth"));
    }

    #[test]
    fn duplicate_insert_returns_already_exists() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        let result = store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None));
        assert!(matches!(result, Err(DictumError::DecisionAlreadyExists)));
    }

    #[test]
    fn get_nonexistent_returns_not_found() {
        let store = make_store();
        let result = store.decision_get("nope");
        assert!(matches!(result, Err(DictumError::DecisionNotFound(_))));
    }

    #[test]
    fn filter_by_kind() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();
        store.decision_insert(&make_decision("d-3", Kind::Rule, Weight::Should, None)).unwrap();

        let results = store.decision_list(&ListFilter {
            kind: Some(Kind::Rule), ..Default::default()
        }).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|d| d.kind == Kind::Rule));
    }

    #[test]
    fn filter_by_scope() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, Some("auth"))).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Rule, Weight::Must, Some("logging"))).unwrap();
        store.decision_insert(&make_decision("d-3", Kind::Rule, Weight::Must, None)).unwrap();

        let results = store.decision_list(&ListFilter {
            scope: Some("auth".to_string()), ..Default::default()
        }).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d-1");
    }

    #[test]
    fn labels_round_trip() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.label_add("d-1", "backend").unwrap();
        store.label_add("d-1", "auth").unwrap();
        store.label_add("d-1", "auth").unwrap(); // idempotent

        let d = store.decision_get("d-1").unwrap();
        assert_eq!(d.labels, vec!["auth", "backend"]);
    }

    #[test]
    fn link_insert_and_query() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();

        store.link_insert(&Link {
            source_id: "d-1".to_string(), target_id: "d-2".to_string(),
            kind: LinkKind::Refines, created_at: "2025-01-01T00:00:00Z".to_string(),
            reason: Some("specializes".to_string()),
        }).unwrap();

        let links = store.links_for_decision("d-1").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target_id, "d-2");
        assert_eq!(links[0].kind, LinkKind::Refines);
        assert_eq!(links[0].reason.as_deref(), Some("specializes"));

        // Inbound
        let links = store.links_for_decision("d-2").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].source_id, "d-1");
    }

    #[test]
    fn link_self_link_rejected() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        let result = store.link_insert(&Link {
            source_id: "d-1".to_string(), target_id: "d-1".to_string(),
            kind: LinkKind::Refines, created_at: "2025-01-01T00:00:00Z".to_string(),
            reason: None,
        });
        assert!(matches!(result, Err(DictumError::SelfLink)));
    }

    #[test]
    fn link_duplicate_rejected() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();
        let link = Link {
            source_id: "d-1".to_string(), target_id: "d-2".to_string(),
            kind: LinkKind::Refines, created_at: "2025-01-01T00:00:00Z".to_string(),
            reason: None,
        };
        store.link_insert(&link).unwrap();
        assert!(matches!(store.link_insert(&link), Err(DictumError::LinkAlreadyExists)));
    }

    #[test]
    fn links_of_kind_query() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();
        store.decision_insert(&make_decision("d-3", Kind::Goal, Weight::Must, None)).unwrap();

        store.link_insert(&Link {
            source_id: "d-1".to_string(), target_id: "d-2".to_string(),
            kind: LinkKind::Refines, created_at: "2025-01-01T00:00:00Z".to_string(),
            reason: None,
        }).unwrap();
        store.link_insert(&Link {
            source_id: "d-2".to_string(), target_id: "d-3".to_string(),
            kind: LinkKind::Requires, created_at: "2025-01-01T00:00:01Z".to_string(),
            reason: None,
        }).unwrap();

        let refines = store.links_of_kind(&LinkKind::Refines).unwrap();
        assert_eq!(refines.len(), 1);
        assert_eq!(refines[0], ("d-1".to_string(), "d-2".to_string()));
    }

    #[test]
    fn update_status() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_update_status("d-1", &Status::Superseded, Some("d-2")).unwrap();
        let d = store.decision_get("d-1").unwrap();
        assert_eq!(d.status, Status::Superseded);
        assert_eq!(d.superseded_by.as_deref(), Some("d-2"));
    }

    #[test]
    fn link_delete_works() {
        let mut store = make_store();
        store.decision_insert(&make_decision("d-1", Kind::Rule, Weight::Must, None)).unwrap();
        store.decision_insert(&make_decision("d-2", Kind::Choice, Weight::Should, None)).unwrap();
        store.link_insert(&Link {
            source_id: "d-1".to_string(), target_id: "d-2".to_string(),
            kind: LinkKind::Refines, created_at: "2025-01-01T00:00:00Z".to_string(),
            reason: None,
        }).unwrap();
        store.link_delete("d-1", &LinkKind::Refines, "d-2").unwrap();
        let links = store.links_for_decision("d-1").unwrap();
        assert!(links.is_empty());
    }

    #[test]
    fn search_finds_by_title() {
        let mut store = make_store();
        store.decision_insert(&Decision {
            title: "Use graph storage for decisions".to_string(),
            ..make_decision("d-1", Kind::Choice, Weight::Should, None)
        }).unwrap();
        store.decision_insert(&Decision {
            title: "Prefer SQLite for simple data".to_string(),
            ..make_decision("d-2", Kind::Choice, Weight::Should, None)
        }).unwrap();

        let results = store.decision_search("graph").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d-1");
    }
}
