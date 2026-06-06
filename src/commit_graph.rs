use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// A single commit in the graph.
#[derive(Debug, Clone)]
pub struct CommitNode {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: u64,
}

/// DAG of commits with parent tracking.
///
/// Each node is a `CommitNode`. Edges point from child → parent,
/// matching git's direction (you traverse backwards through history).
#[derive(Debug, Clone)]
pub struct CommitGraph {
    graph: DiGraph<CommitNode, ()>,
    index: HashMap<String, NodeIndex>,
}

impl CommitGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    /// Insert a commit. Returns its node index.
    /// If already present, returns the existing index.
    pub fn insert(&mut self, commit: CommitNode) -> NodeIndex {
        if let Some(&idx) = self.index.get(&commit.hash) {
            return idx;
        }
        let idx = self.graph.add_node(commit.clone());
        self.index.insert(commit.hash.clone(), idx);
        idx
    }

    /// Add a parent edge: `child_hash` → `parent_hash`.
    /// Inserts placeholder commits if they don't exist.
    pub fn add_parent(&mut self, child_hash: &str, parent_hash: &str) {
        let child = if let Some(&idx) = self.index.get(child_hash) {
            idx
        } else {
            self.insert(CommitNode {
                hash: child_hash.into(),
                message: String::new(),
                author: String::new(),
                timestamp: 0,
            })
        };

        let parent = if let Some(&idx) = self.index.get(parent_hash) {
            idx
        } else {
            self.insert(CommitNode {
                hash: parent_hash.into(),
                message: String::new(),
                author: String::new(),
                timestamp: 0,
            })
        };

        if !self.graph.edges_connecting(child, parent).any(|_| true) {
            self.graph.add_edge(child, parent, ());
        }
    }

    /// Get a commit by hash.
    pub fn get(&self, hash: &str) -> Option<&CommitNode> {
        self.index.get(hash).map(|&idx| &self.graph[idx])
    }

    /// Get the node index for a hash.
    pub fn node_index(&self, hash: &str) -> Option<NodeIndex> {
        self.index.get(hash).copied()
    }

    /// Get parent hashes of a commit.
    pub fn parents(&self, hash: &str) -> Vec<String> {
        let Some(&idx) = self.index.get(hash) else {
            return vec![];
        };
        self.graph
            .neighbors_directed(idx, petgraph::Direction::Outgoing)
            .filter_map(|n| self.graph.node_weight(n).map(|c| c.hash.clone()))
            .collect()
    }

    /// Get children (descendants) of a commit.
    pub fn children(&self, hash: &str) -> Vec<String> {
        let Some(&idx) = self.index.get(hash) else {
            return vec![];
        };
        self.graph
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .filter_map(|n| self.graph.node_weight(n).map(|c| c.hash.clone()))
            .collect()
    }

    /// Return all root commits (no parents).
    pub fn roots(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .neighbors_directed(idx, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx).map(|c| c.hash.clone()))
            .collect()
    }

    /// Return all leaf commits (no children).
    pub fn leaves(&self) -> Vec<String> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx).map(|c| c.hash.clone()))
            .collect()
    }

    /// Number of commits.
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Find the lowest common ancestor of two commits.
    pub fn lca(&self, a: &str, b: &str) -> Option<String> {
        let ancestors_a = self.ancestor_set(a)?;
        let Some(&b_idx) = self.index.get(b) else {
            return None;
        };
        let mut queue = vec![b_idx];
        let mut visited = std::collections::HashSet::new();
        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            if let Some(weight) = self.graph.node_weight(current) {
                if ancestors_a.contains(&weight.hash) {
                    return Some(weight.hash.clone());
                }
            }
            for neighbor in self
                .graph
                .neighbors_directed(current, petgraph::Direction::Outgoing)
            {
                queue.push(neighbor);
            }
        }
        None
    }

    /// Collect all ancestors of a commit (inclusive).
    fn ancestor_set(&self, hash: &str) -> Option<std::collections::HashSet<String>> {
        let &idx = self.index.get(hash)?;
        let mut set = std::collections::HashSet::new();
        let mut stack = vec![idx];
        while let Some(current) = stack.pop() {
            if let Some(w) = self.graph.node_weight(current) {
                if set.contains(&w.hash) {
                    continue;
                }
                set.insert(w.hash.clone());
            }
            for n in self
                .graph
                .neighbors_directed(current, petgraph::Direction::Outgoing)
            {
                stack.push(n);
            }
        }
        Some(set)
    }

    /// Topological sort (roots first for a child→parent DAG).
    pub fn topological_sort(&self) -> Vec<String> {
        // petgraph toposort returns leaves-first for child→parent edges
        // We reverse to get roots-first (parents before children)
        let sorted = petgraph::algo::toposort(&self.graph, None).unwrap_or_default();
        sorted
            .into_iter()
            .rev()
            .filter_map(|idx| self.graph.node_weight(idx).map(|c| c.hash.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(hash: &str, msg: &str) -> CommitNode {
        CommitNode {
            hash: hash.into(),
            message: msg.into(),
            author: "test".into(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "first"));
        assert_eq!(g.get("aaa").unwrap().message, "first");
        assert!(g.get("bbb").is_none());
    }

    #[test]
    fn test_parents_and_children() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "root"));
        g.insert(make_commit("bbb", "child"));
        g.add_parent("bbb", "aaa");
        assert_eq!(g.parents("bbb"), vec!["aaa"]);
        assert_eq!(g.children("aaa"), vec!["bbb"]);
    }

    #[test]
    fn test_roots_and_leaves() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "root"));
        g.insert(make_commit("bbb", "mid"));
        g.insert(make_commit("ccc", "tip"));
        g.add_parent("bbb", "aaa");
        g.add_parent("ccc", "bbb");
        let mut roots = g.roots();
        roots.sort();
        assert_eq!(roots, vec!["aaa"]);
        let mut leaves = g.leaves();
        leaves.sort();
        assert_eq!(leaves, vec!["ccc"]);
    }

    #[test]
    fn test_lca_simple() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "root"));
        g.insert(make_commit("bbb", "mid"));
        g.insert(make_commit("ccc", "left"));
        g.insert(make_commit("ddd", "right"));
        g.add_parent("bbb", "aaa");
        g.add_parent("ccc", "bbb");
        g.add_parent("ddd", "bbb");
        assert_eq!(g.lca("ccc", "ddd"), Some("bbb".into()));
    }

    #[test]
    fn test_duplicate_insert() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "first"));
        g.insert(make_commit("aaa", "second"));
        assert_eq!(g.len(), 1);
        assert_eq!(g.get("aaa").unwrap().message, "first");
    }

    #[test]
    fn test_topological_sort() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa", "root"));
        g.insert(make_commit("bbb", "mid"));
        g.insert(make_commit("ccc", "tip"));
        g.add_parent("bbb", "aaa");
        g.add_parent("ccc", "bbb");
        let topo = g.topological_sort();
        // toposort returns root-first order for a child→parent DAG
        // aaa (root) should come before ccc (leaf)
        let pos_aaa = topo.iter().position(|h| h == "aaa").unwrap();
        let pos_ccc = topo.iter().position(|h| h == "ccc").unwrap();
        assert!(pos_aaa < pos_ccc, "aaa ({}) should come before ccc ({})", pos_aaa, pos_ccc);
    }
}
