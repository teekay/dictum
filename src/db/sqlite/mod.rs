mod decisions;
mod labels;
mod links;
mod schema;

use std::collections::{HashSet, VecDeque};
use std::path::Path;

use rusqlite::Connection;

use crate::db::store::{ListFilter, Neighborhood, Store};
use crate::error::Result;
use crate::model::{Decision, Link, LinkKind, Status};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(dictum_dir: &Path) -> Result<Self> {
        let db_path = dictum_dir.join("dictum.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        migrate(&conn)?;
        initialize(&conn)?;
        Ok(SqliteStore { conn })
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        initialize(&conn)?;
        Ok(SqliteStore { conn })
    }
}

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(schema::CREATE_DECISIONS_TABLE)?;
    conn.execute_batch(schema::CREATE_LINKS_TABLE)?;
    conn.execute_batch(schema::CREATE_LABELS_TABLE)?;
    Ok(())
}

fn needs_migration(conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(decisions)")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(!columns.iter().any(|c| c == "kind"))
}

fn migrate(conn: &Connection) -> Result<()> {
    let table_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='decisions'",
        [],
        |row| row.get(0),
    )?;
    if !table_exists {
        return Ok(());
    }
    if !needs_migration(conn)? {
        return Ok(());
    }
    for sql in schema::MIGRATE_DECISIONS_V2 {
        conn.execute_batch(sql)?;
    }
    conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
    for sql in schema::MIGRATE_LINKS_V2 {
        conn.execute_batch(sql)?;
    }
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    Ok(())
}

impl Store for SqliteStore {
    fn decision_insert(&mut self, decision: &Decision) -> Result<()> {
        decisions::insert(&self.conn, decision)
    }

    fn decision_get(&self, id: &str) -> Result<Decision> {
        decisions::get(&self.conn, id)
    }

    fn decision_list(&self, filter: &ListFilter) -> Result<Vec<Decision>> {
        decisions::list(&self.conn, filter)
    }

    fn decision_update_status(
        &mut self,
        id: &str,
        status: &Status,
        superseded_by: Option<&str>,
    ) -> Result<()> {
        decisions::update_status(&self.conn, id, status, superseded_by)
    }

    fn decision_search(&self, query: &str) -> Result<Vec<Decision>> {
        decisions::search(&self.conn, query)
    }

    fn label_add(&mut self, decision_id: &str, label: &str) -> Result<()> {
        labels::add(&self.conn, decision_id, label)
    }

    fn link_insert(&mut self, link: &Link) -> Result<()> {
        links::insert(&self.conn, link)
    }

    fn link_delete(&mut self, source_id: &str, kind: &LinkKind, target_id: &str) -> Result<()> {
        links::delete(&self.conn, source_id, kind, target_id)
    }

    fn links_for_decision(&self, decision_id: &str) -> Result<Vec<Link>> {
        links::get_for_decision(&self.conn, decision_id)
    }

    fn links_of_kind(&self, kind: &LinkKind) -> Result<Vec<(String, String)>> {
        links::get_of_kind(&self.conn, kind)
    }

    fn neighborhood(&self, id: &str, depth: u32) -> Result<Neighborhood> {
        let mut visited_ids: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, u32)> = VecDeque::new();
        visited_ids.insert(id.to_string());
        queue.push_back((id.to_string(), 0));

        let mut all_links: Vec<Link> = Vec::new();

        while let Some((current_id, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }
            let node_links = links::get_for_decision(&self.conn, &current_id)?;
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

        // Deduplicate links by (source, target, kind)
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        all_links.retain(|l| {
            seen.insert((l.source_id.clone(), l.target_id.clone(), l.kind.to_string()))
        });

        let mut result_decisions = Vec::new();
        for node_id in &visited_ids {
            result_decisions.push(decisions::get(&self.conn, node_id)?);
        }

        Ok(Neighborhood { decisions: result_decisions, links: all_links })
    }

    fn reachable(&self, id: &str, kinds: &[LinkKind]) -> Result<Vec<String>> {
        let mut all_edges: Vec<(String, String)> = Vec::new();
        for kind in kinds {
            all_edges.extend(links::get_of_kind(&self.conn, kind)?);
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

        // Skip the starting node itself
        Ok(visited.into_iter().skip(1).collect())
    }
}
