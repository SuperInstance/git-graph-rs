use crate::BranchForest;
use std::collections::HashMap;

/// Which agent owns which branch (and what they're doing).
#[derive(Debug, Clone)]
pub struct AgentAssignment {
    pub agent_id: String,
    pub branch: String,
    pub task: String,
    pub status: String,
}

/// A detected conflict between agent branches.
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub agent_a: String,
    pub agent_b: String,
    pub branch_a: String,
    pub branch_b: String,
    pub common_ancestor: String,
}

/// Map agents to branches, detect merge conflicts between agent thought branches.
#[derive(Debug, Clone)]
pub struct AgentTopology {
    agents: HashMap<String, AgentAssignment>,
    forest: BranchForest,
}

impl AgentTopology {
    pub fn new(forest: BranchForest) -> Self {
        Self {
            agents: HashMap::new(),
            forest,
        }
    }

    /// Register an agent on a branch.
    pub fn assign(&mut self, agent_id: &str, branch: &str, task: &str, status: &str) {
        self.agents.insert(
            agent_id.to_string(),
            AgentAssignment {
                agent_id: agent_id.to_string(),
                branch: branch.to_string(),
                task: task.to_string(),
                status: status.to_string(),
            },
        );
    }

    /// Remove an agent.
    pub fn unassign(&mut self, agent_id: &str) -> bool {
        self.agents.remove(agent_id).is_some()
    }

    /// Get assignment for an agent.
    pub fn get(&self, agent_id: &str) -> Option<&AgentAssignment> {
        self.agents.get(agent_id)
    }

    /// List all agents.
    pub fn agents(&self) -> Vec<&AgentAssignment> {
        self.agents.values().collect()
    }

    /// Number of agents.
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Access the branch forest.
    pub fn forest(&self) -> &BranchForest {
        &self.forest
    }

    pub fn forest_mut(&mut self) -> &mut BranchForest {
        &mut self.forest
    }

    /// Detect conflicts: pairs of agents whose branches have diverged from a
    /// common ancestor and are both in "active" status.
    pub fn detect_conflicts(&self) -> Vec<ConflictInfo> {
        let agents: Vec<_> = self.agents.values().collect();
        let mut conflicts = Vec::new();
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let a = &agents[i];
                let b = &agents[j];
                if a.status != "active" || b.status != "active" {
                    continue;
                }
                if let Some(div) = self.forest.find_divergence(&a.branch, &b.branch) {
                    conflicts.push(ConflictInfo {
                        agent_a: a.agent_id.clone(),
                        agent_b: b.agent_id.clone(),
                        branch_a: a.branch.clone(),
                        branch_b: b.branch.clone(),
                        common_ancestor: div.common_ancestor,
                    });
                }
            }
        }
        conflicts
    }

    /// Find all agents on the same branch.
    pub fn agents_on_branch(&self, branch: &str) -> Vec<String> {
        self.agents
            .values()
            .filter(|a| a.branch == branch)
            .map(|a| a.agent_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommitGraph, CommitNode};

    fn setup() -> AgentTopology {
        let mut g = CommitGraph::new();
        let c = |h: &str| CommitNode {
            hash: h.into(),
            message: String::new(),
            author: "t".into(),
            timestamp: 0,
        };
        g.insert(c("root"));
        g.insert(c("a1"));
        g.insert(c("b1"));
        g.add_parent("a1", "root");
        g.add_parent("b1", "root");

        let mut f = BranchForest::new(g);
        f.add_branch("br-a", "a1", None);
        f.add_branch("br-b", "b1", None);

        let mut topo = AgentTopology::new(f);
        topo.assign("agent-a", "br-a", "task-a", "active");
        topo.assign("agent-b", "br-b", "task-b", "active");
        topo
    }

    #[test]
    fn test_assign_and_get() {
        let topo = setup();
        assert_eq!(topo.get("agent-a").unwrap().branch, "br-a");
        assert_eq!(topo.len(), 2);
    }

    #[test]
    fn test_detect_conflicts() {
        let topo = setup();
        let conflicts = topo.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].common_ancestor, "root");
    }

    #[test]
    fn test_no_conflict_if_inactive() {
        let mut topo = setup();
        topo.assign("agent-a", "br-a", "task-a", "idle");
        assert!(topo.detect_conflicts().is_empty());
    }
}
