use crate::CommitGraph;
use std::collections::HashMap;

/// Metadata about a branch.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub tip_hash: String,
    pub agent: Option<String>,
}

/// A point where two branches diverge.
#[derive(Debug, Clone)]
pub struct DivergencePoint {
    pub branch_a: String,
    pub branch_b: String,
    pub common_ancestor: String,
}

/// Track branches and their divergence points.
#[derive(Debug, Clone)]
pub struct BranchForest {
    branches: HashMap<String, BranchInfo>,
    graph: CommitGraph,
}

impl BranchForest {
    pub fn new(graph: CommitGraph) -> Self {
        Self {
            branches: HashMap::new(),
            graph,
        }
    }

    /// Register a branch pointing at `tip_hash`.
    pub fn add_branch(&mut self, name: &str, tip_hash: &str, agent: Option<String>) {
        self.branches.insert(
            name.to_string(),
            BranchInfo {
                name: name.to_string(),
                tip_hash: tip_hash.to_string(),
                agent,
            },
        );
    }

    /// Remove a branch.
    pub fn remove_branch(&mut self, name: &str) -> bool {
        self.branches.remove(name).is_some()
    }

    /// Get branch info.
    pub fn get_branch(&self, name: &str) -> Option<&BranchInfo> {
        self.branches.get(name)
    }

    /// List all branch names.
    pub fn branch_names(&self) -> Vec<String> {
        self.branches.keys().cloned().collect()
    }

    /// Number of branches.
    pub fn len(&self) -> usize {
        self.branches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }

    /// Access the underlying commit graph.
    pub fn graph(&self) -> &CommitGraph {
        &self.graph
    }

    /// Mutable access to the commit graph.
    pub fn graph_mut(&mut self) -> &mut CommitGraph {
        &mut self.graph
    }

    /// Find the divergence point between two branches.
    pub fn find_divergence(&self, a: &str, b: &str) -> Option<DivergencePoint> {
        let tip_a = self.branches.get(a)?.tip_hash.clone();
        let tip_b = self.branches.get(b)?.tip_hash.clone();
        let ancestor = self.graph.lca(&tip_a, &tip_b)?;
        Some(DivergencePoint {
            branch_a: a.to_string(),
            branch_b: b.to_string(),
            common_ancestor: ancestor,
        })
    }

    /// Find all pairwise divergence points.
    pub fn all_divergences(&self) -> Vec<DivergencePoint> {
        let names: Vec<_> = self.branches.keys().cloned().collect();
        let mut result = Vec::new();
        for i in 0..names.len() {
            for j in (i + 1)..names.len() {
                if let Some(dp) = self.find_divergence(&names[i], &names[j]) {
                    result.push(dp);
                }
            }
        }
        result
    }

    /// Find branches belonging to an agent.
    pub fn branches_for_agent(&self, agent: &str) -> Vec<String> {
        self.branches
            .values()
            .filter(|b| b.agent.as_deref() == Some(agent))
            .map(|b| b.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommitNode;

    fn make_commit(hash: &str) -> CommitNode {
        CommitNode {
            hash: hash.into(),
            message: String::new(),
            author: "test".into(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_branch_crud() {
        let g = CommitGraph::new();
        let mut f = BranchForest::new(g);
        f.add_branch("main", "aaa", None);
        f.add_branch("feat", "bbb", Some("agent-1".into()));
        assert_eq!(f.len(), 2);
        assert!(f.get_branch("main").is_some());
        assert!(f.remove_branch("feat"));
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn test_find_divergence() {
        let mut g = CommitGraph::new();
        g.insert(make_commit("aaa"));
        g.insert(make_commit("bbb"));
        g.insert(make_commit("ccc"));
        g.insert(make_commit("ddd"));
        g.add_parent("bbb", "aaa");
        g.add_parent("ccc", "bbb");
        g.add_parent("ddd", "bbb");

        let mut f = BranchForest::new(g);
        f.add_branch("left", "ccc", None);
        f.add_branch("right", "ddd", None);
        let div = f.find_divergence("left", "right").unwrap();
        assert_eq!(div.common_ancestor, "bbb");
    }
}
