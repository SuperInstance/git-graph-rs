use crate::CommitGraph;
use std::collections::HashMap;

/// Find shortest paths between agents through the commit graph.
///
/// Treats the DAG as undirected (can traverse parent → child and child → parent)
/// so messages can route up through common ancestors and down to other branches.
pub struct MessageRoute<'a> {
    graph: &'a CommitGraph,
}

impl<'a> MessageRoute<'a> {
    pub fn new(graph: &'a CommitGraph) -> Self {
        Self { graph }
    }

    /// Find the shortest path from `from_hash` to `to_hash` through the commit graph.
    /// Returns the sequence of commit hashes forming the path, or `None` if disconnected.
    pub fn find_route(&self, from_hash: &str, to_hash: &str) -> Option<Vec<String>> {
        if from_hash == to_hash {
            return Some(vec![from_hash.to_string()]);
        }

        let mut prev: HashMap<String, Option<String>> = HashMap::new();
        let mut queue = vec![from_hash.to_string()];
        prev.insert(from_hash.to_string(), None);

        while let Some(current) = queue.pop() {
            if current == to_hash {
                let mut path = vec![];
                let mut node: Option<String> = Some(to_hash.to_string());
                while let Some(n) = node {
                    path.push(n.clone());
                    node = prev.get(&n).cloned().flatten();
                }
                path.reverse();
                return Some(path);
            }

            let mut neighbors = self.graph.parents(&current);
            neighbors.extend(self.graph.children(&current));

            for neighbor in neighbors {
                if prev.contains_key(&neighbor) {
                    continue;
                }
                prev.insert(neighbor.clone(), Some(current.clone()));
                queue.push(neighbor);
            }
        }

        None
    }

    /// Return the distance (number of edges) between two commits.
    pub fn distance(&self, from_hash: &str, to_hash: &str) -> Option<usize> {
        self.find_route(from_hash, to_hash)
            .map(|p| p.len().saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommitNode;

    fn build_graph() -> CommitGraph {
        let mut g = CommitGraph::new();
        let c = |h: &str| CommitNode {
            hash: h.into(),
            message: String::new(),
            author: "t".into(),
            timestamp: 0,
        };
        g.insert(c("root"));
        g.insert(c("a"));
        g.insert(c("b"));
        g.insert(c("c"));
        g.add_parent("a", "root");
        g.add_parent("b", "a");
        g.add_parent("c", "a");
        g
    }

    #[test]
    fn test_route_same_node() {
        let g = build_graph();
        let mr = MessageRoute::new(&g);
        let path = mr.find_route("root", "root").unwrap();
        assert_eq!(path, vec!["root"]);
    }

    #[test]
    fn test_route_linear() {
        let g = build_graph();
        let mr = MessageRoute::new(&g);
        let path = mr.find_route("root", "b").unwrap();
        assert_eq!(path, vec!["root", "a", "b"]);
        assert_eq!(mr.distance("root", "b"), Some(2));
    }

    #[test]
    fn test_route_cross_branch() {
        let g = build_graph();
        let mr = MessageRoute::new(&g);
        let path = mr.find_route("b", "c").unwrap();
        assert_eq!(path, vec!["b", "a", "c"]);
    }

    #[test]
    fn test_route_disconnected() {
        let mut g = CommitGraph::new();
        let c = |h: &str| CommitNode {
            hash: h.into(),
            message: String::new(),
            author: "t".into(),
            timestamp: 0,
        };
        g.insert(c("x"));
        g.insert(c("y"));
        let mr = MessageRoute::new(&g);
        assert!(mr.find_route("x", "y").is_none());
    }
}
