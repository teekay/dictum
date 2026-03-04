use std::collections::{HashMap, HashSet};

use crate::model::Decision;

pub struct TreeStructure<'a> {
    pub children_map: HashMap<&'a str, Vec<&'a str>>,
    pub roots: Vec<&'a str>,
    pub decision_map: HashMap<&'a str, &'a Decision>,
}

pub fn build_tree<'a>(
    decisions: &'a [Decision],
    refines_links: &'a [(String, String)],
) -> TreeStructure<'a> {
    let decision_map: HashMap<&str, &Decision> =
        decisions.iter().map(|d| (d.id.as_str(), d)).collect();

    let mut children_map: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut has_parent: HashSet<&str> = HashSet::new();

    for (source, target) in refines_links {
        children_map
            .entry(target.as_str())
            .or_default()
            .push(source.as_str());
        has_parent.insert(source.as_str());
    }

    let mut roots: Vec<&str> = decisions
        .iter()
        .filter(|d| !has_parent.contains(d.id.as_str()))
        .map(|d| d.id.as_str())
        .collect();
    roots.sort();

    TreeStructure {
        children_map,
        roots,
        decision_map,
    }
}

pub fn format_tree(decisions: &[Decision], refines_links: &[(String, String)]) -> String {
    if decisions.is_empty() {
        return "No decisions found.\n".to_string();
    }

    let tree = build_tree(decisions, refines_links);

    let mut out = String::new();
    for (i, root) in tree.roots.iter().enumerate() {
        let is_last = i == tree.roots.len() - 1;
        format_node(
            &mut out,
            root,
            "",
            is_last,
            &tree.decision_map,
            &tree.children_map,
            true,
        );
    }

    out
}

fn format_node(
    out: &mut String,
    id: &str,
    prefix: &str,
    is_last: bool,
    decisions: &HashMap<&str, &Decision>,
    children: &HashMap<&str, Vec<&str>>,
    is_root: bool,
) {
    let connector = if is_root && prefix.is_empty() {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    if let Some(d) = decisions.get(id) {
        out.push_str(&format!(
            "{}{}{} {}\n",
            prefix, connector, d.id, d.title
        ));
    } else {
        out.push_str(&format!("{}{}{} (unknown)\n", prefix, connector, id));
    }

    if let Some(kids) = children.get(id) {
        let child_prefix = if is_root && prefix.is_empty() {
            "".to_string()
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        for (i, child) in kids.iter().enumerate() {
            let child_is_last = i == kids.len() - 1;
            format_node(out, child, &child_prefix, child_is_last, decisions, children, false);
        }
    }
}
