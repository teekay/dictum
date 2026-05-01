use std::collections::HashSet;
use std::path::Path;

use crate::db;
use crate::db::{ListFilter, Store};
use crate::error::Result;
use crate::format::tree::build_tree;
use crate::model::decision::{Kind, Level, Status, Weight};
use crate::model::{Decision, Link};

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    List,
    Detail,
    Tree,
    Search,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub title: String,
    pub depth: usize,
    pub has_children: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FilterState {
    pub kind: Option<Kind>,
    pub weight: Option<Weight>,
    pub status: Option<Status>,
    pub level: Option<Level>,
    pub scope: Option<String>,
}

impl FilterState {
    pub fn is_empty(&self) -> bool {
        self.kind.is_none()
            && self.weight.is_none()
            && self.status.is_none()
            && self.level.is_none()
            && self.scope.is_none()
    }

    pub fn to_list_filter(&self) -> ListFilter {
        ListFilter {
            level: self.level.clone(),
            status: self.status.clone(),
            label: None,
            kind: self.kind.clone(),
            weight: self.weight.clone(),
            scope: self.scope.clone(),
        }
    }

    pub fn cycle_field(&mut self, field: usize) {
        match field {
            0 => {
                self.kind = match &self.kind {
                    None => Some(Kind::Principle),
                    Some(Kind::Principle) => Some(Kind::Constraint),
                    Some(Kind::Constraint) => Some(Kind::Assumption),
                    Some(Kind::Assumption) => Some(Kind::Choice),
                    Some(Kind::Choice) => Some(Kind::Rule),
                    Some(Kind::Rule) => Some(Kind::Goal),
                    Some(Kind::Goal) => None,
                };
            }
            1 => {
                self.weight = match &self.weight {
                    None => Some(Weight::Must),
                    Some(Weight::Must) => Some(Weight::Should),
                    Some(Weight::Should) => Some(Weight::May),
                    Some(Weight::May) => None,
                };
            }
            2 => {
                self.status = match &self.status {
                    None => Some(Status::Active),
                    Some(Status::Active) => Some(Status::Superseded),
                    Some(Status::Superseded) => Some(Status::Deprecated),
                    Some(Status::Deprecated) => Some(Status::Draft),
                    Some(Status::Draft) => None,
                };
            }
            3 => {
                self.level = match &self.level {
                    None => Some(Level::Strategic),
                    Some(Level::Strategic) => Some(Level::Tactical),
                    Some(Level::Tactical) => Some(Level::Operational),
                    Some(Level::Operational) => None,
                };
            }
            _ => {}
        }
    }
}

pub struct App {
    pub store: Box<dyn Store>,
    pub view: View,
    pub decisions: Vec<Decision>,
    pub selected_index: usize,
    pub detail_scroll: u16,
    pub filter: FilterState,
    pub filter_panel_open: bool,
    pub tree_nodes: Vec<TreeNode>,
    pub expanded_nodes: HashSet<String>,
    pub search_query: String,
    pub selected_decision: Option<Decision>,
    pub selected_links: Vec<Link>,
    pub refines_links: Vec<(String, String)>,
}

impl App {
    pub fn new(cwd: &Path) -> Result<Self> {
        let dictum_dir = cwd.join(".dictum");
        crate::cli::ensure_init(&dictum_dir)?;
        let store = db::open(&dictum_dir)?;

        let decisions = store.decision_get_all()?;
        let refines_links = store.links_of_kind(&crate::model::LinkKind::Refines)?;

        let mut app = App {
            store,
            view: View::List,
            decisions,
            selected_index: 0,
            detail_scroll: 0,
            filter: FilterState::default(),
            filter_panel_open: false,
            tree_nodes: Vec::new(),
            expanded_nodes: HashSet::new(),
            search_query: String::new(),
            selected_decision: None,
            selected_links: Vec::new(),
            refines_links,
        };

        app.refresh_tree();
        app.load_selected_decision();

        Ok(app)
    }

    pub fn refresh_list(&mut self) -> Result<()> {
        if self.filter.is_empty() {
            self.decisions = self.store.decision_get_all()?;
        } else {
            self.decisions = self.store.decision_list(&self.filter.to_list_filter())?;
        }
        self.refines_links = self.store.links_of_kind(&crate::model::LinkKind::Refines)?;
        if self.selected_index >= self.decisions.len() && !self.decisions.is_empty() {
            self.selected_index = self.decisions.len() - 1;
        }
        self.refresh_tree();
        self.load_selected_decision();
        Ok(())
    }

    pub fn load_selected_decision(&mut self) {
        let selected_id = match self.view {
            View::Tree => {
                self.tree_nodes
                    .get(self.selected_index)
                    .map(|n| n.id.clone())
            }
            _ => self
                .decisions
                .get(self.selected_index)
                .map(|d| d.id.clone()),
        };

        if let Some(id) = selected_id {
            if let Ok(d) = self.store.decision_get(&id) {
                self.selected_links =
                    self.store.links_for_decision(&id).unwrap_or_default();
                self.selected_decision = Some(d);
                self.detail_scroll = 0;
                return;
            }
        }
        self.selected_decision = None;
        self.selected_links = Vec::new();
    }

    pub fn refresh_tree(&mut self) {
        let tree = build_tree(&self.decisions, &self.refines_links);
        let mut nodes = Vec::new();

        for root in &tree.roots {
            flatten_tree_node(
                root,
                0,
                &tree.children_map,
                &tree.decision_map,
                &self.expanded_nodes,
                &mut nodes,
            );
        }
        self.tree_nodes = nodes;
    }

    pub fn toggle_tree_node(&mut self) {
        if let Some(node) = self.tree_nodes.get(self.selected_index) {
            if node.has_children {
                let id = node.id.clone();
                if self.expanded_nodes.contains(&id) {
                    self.expanded_nodes.remove(&id);
                } else {
                    self.expanded_nodes.insert(id);
                }
                self.refresh_tree();
            }
        }
    }

    pub fn search(&mut self) -> Result<()> {
        if self.search_query.is_empty() {
            self.decisions = self.store.decision_get_all()?;
        } else {
            self.decisions = self.store.decision_search(&self.search_query)?;
        }
        self.selected_index = 0;
        self.load_selected_decision();
        Ok(())
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = match self.view {
            View::Tree => self.tree_nodes.len(),
            _ => self.decisions.len(),
        };
        if len == 0 {
            return;
        }
        let new_index = if delta < 0 {
            self.selected_index.saturating_sub((-delta) as usize)
        } else {
            (self.selected_index + delta as usize).min(len - 1)
        };
        self.selected_index = new_index;
        self.load_selected_decision();
    }

    pub fn item_count(&self) -> usize {
        match self.view {
            View::Tree => self.tree_nodes.len(),
            _ => self.decisions.len(),
        }
    }
}

fn flatten_tree_node(
    id: &str,
    depth: usize,
    children_map: &std::collections::HashMap<&str, Vec<&str>>,
    decision_map: &std::collections::HashMap<&str, &Decision>,
    expanded: &HashSet<String>,
    nodes: &mut Vec<TreeNode>,
) {
    let has_children = children_map.get(id).map_or(false, |c| !c.is_empty());
    let title = decision_map
        .get(id)
        .map(|d| d.title.clone())
        .unwrap_or_else(|| "(unknown)".to_string());

    nodes.push(TreeNode {
        id: id.to_string(),
        title,
        depth,
        has_children,
    });

    if has_children && expanded.contains(id) {
        if let Some(kids) = children_map.get(id) {
            for child in kids {
                flatten_tree_node(child, depth + 1, children_map, decision_map, expanded, nodes);
            }
        }
    }
}
