use crate::error::Result;
use crate::model::{Decision, Kind, Level, Link, LinkKind, Status, Weight};

#[derive(Default)]
pub struct ListFilter {
    pub level: Option<Level>,
    pub status: Option<Status>,
    pub label: Option<String>,
    pub kind: Option<Kind>,
    pub weight: Option<Weight>,
    pub scope: Option<String>,
}

#[allow(dead_code)]
pub struct Neighborhood {
    pub decisions: Vec<Decision>,
    pub links: Vec<Link>,
}

pub trait Store {
    // --- Decision CRUD ---
    fn decision_insert(&mut self, decision: &Decision) -> Result<()>;
    fn decision_get(&self, id: &str) -> Result<Decision>;
    fn decision_list(&self, filter: &ListFilter) -> Result<Vec<Decision>>;
    fn decision_update_status(
        &mut self,
        id: &str,
        status: &Status,
        superseded_by: Option<&str>,
    ) -> Result<()>;
    fn decision_search(&self, query: &str) -> Result<Vec<Decision>>;
    fn decision_get_all(&self) -> Result<Vec<Decision>> {
        self.decision_list(&ListFilter::default())
    }

    // --- Label operations ---
    fn label_add(&mut self, decision_id: &str, label: &str) -> Result<()>;

    // --- Link operations ---
    fn link_insert(&mut self, link: &Link) -> Result<()>;
    fn link_delete(&mut self, source_id: &str, kind: &LinkKind, target_id: &str) -> Result<()>;
    fn links_for_decision(&self, decision_id: &str) -> Result<Vec<Link>>;
    fn links_of_kind(&self, kind: &LinkKind) -> Result<Vec<(String, String)>>;

    // --- Graph traversal (used by Grafeo backend; available to all) ---
    #[allow(dead_code)]
    fn neighborhood(&self, id: &str, depth: u32) -> Result<Neighborhood>;
    #[allow(dead_code)]
    fn reachable(&self, id: &str, kinds: &[LinkKind]) -> Result<Vec<String>>;
}
