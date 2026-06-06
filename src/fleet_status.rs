use std::collections::HashMap;

/// Health status of a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Health {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Health::Healthy => write!(f, "healthy"),
            Health::Degraded => write!(f, "degraded"),
            Health::Unhealthy => write!(f, "unhealthy"),
            Health::Unknown => write!(f, "unknown"),
        }
    }
}

/// Status of a single agent workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceStatus {
    pub agent_id: String,
    pub branch: String,
    pub uncommitted_changes: u32,
    pub last_commit_ts: u64,
    pub health: Health,
    pub active_tasks: u32,
}

/// Aggregate status across multiple agent workspaces.
#[derive(Debug, Clone)]
pub struct FleetStatus {
    workspaces: HashMap<String, WorkspaceStatus>,
}

impl FleetStatus {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
        }
    }

    /// Register or update a workspace status.
    pub fn update(&mut self, status: WorkspaceStatus) {
        self.workspaces.insert(status.agent_id.clone(), status);
    }

    /// Remove a workspace.
    pub fn remove(&mut self, agent_id: &str) -> bool {
        self.workspaces.remove(agent_id).is_some()
    }

    /// Get a workspace status.
    pub fn get(&self, agent_id: &str) -> Option<&WorkspaceStatus> {
        self.workspaces.get(agent_id)
    }

    /// List all workspace statuses.
    pub fn all(&self) -> Vec<&WorkspaceStatus> {
        self.workspaces.values().collect()
    }

    /// Number of workspaces.
    pub fn len(&self) -> usize {
        self.workspaces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.workspaces.is_empty()
    }

    /// Count workspaces by health.
    pub fn count_by_health(&self) -> HashMap<Health, usize> {
        let mut counts = HashMap::new();
        for ws in self.workspaces.values() {
            *counts.entry(ws.health.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Overall fleet health: worst individual health.
    pub fn overall_health(&self) -> Health {
        if self.workspaces.is_empty() {
            return Health::Unknown;
        }
        let priority = |h: &Health| -> u8 {
            match h {
                Health::Healthy => 0,
                Health::Degraded => 1,
                Health::Unhealthy => 2,
                Health::Unknown => 3,
            }
        };
        let worst = self
            .workspaces
            .values()
            .map(|ws| &ws.health)
            .max_by_key(|h| priority(h))
            .unwrap();
        worst.clone()
    }

    /// Workspaces with uncommitted changes.
    pub fn dirty_workspaces(&self) -> Vec<&WorkspaceStatus> {
        self.workspaces
            .values()
            .filter(|ws| ws.uncommitted_changes > 0)
            .collect()
    }

    /// Workspaces that haven't committed since `since_ts`.
    pub fn stale_since(&self, since_ts: u64) -> Vec<&WorkspaceStatus> {
        self.workspaces
            .values()
            .filter(|ws| ws.last_commit_ts < since_ts)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws(id: &str, health: Health, changes: u32, ts: u64) -> WorkspaceStatus {
        WorkspaceStatus {
            agent_id: id.into(),
            branch: "main".into(),
            uncommitted_changes: changes,
            last_commit_ts: ts,
            health,
            active_tasks: 1,
        }
    }

    #[test]
    fn test_fleet_basic() {
        let mut fleet = FleetStatus::new();
        fleet.update(ws("a1", Health::Healthy, 0, 100));
        fleet.update(ws("a2", Health::Healthy, 0, 200));
        assert_eq!(fleet.len(), 2);
        assert!(fleet.get("a1").is_some());
        fleet.remove("a1");
        assert_eq!(fleet.len(), 1);
    }

    #[test]
    fn test_overall_health() {
        let mut fleet = FleetStatus::new();
        fleet.update(ws("a1", Health::Healthy, 0, 100));
        fleet.update(ws("a2", Health::Degraded, 0, 100));
        assert_eq!(fleet.overall_health(), Health::Degraded);

        fleet.update(ws("a3", Health::Unhealthy, 0, 100));
        assert_eq!(fleet.overall_health(), Health::Unhealthy);
    }

    #[test]
    fn test_dirty_workspaces() {
        let mut fleet = FleetStatus::new();
        fleet.update(ws("a1", Health::Healthy, 0, 100));
        fleet.update(ws("a2", Health::Healthy, 3, 100));
        assert_eq!(fleet.dirty_workspaces().len(), 1);
    }

    #[test]
    fn test_stale_since() {
        let mut fleet = FleetStatus::new();
        fleet.update(ws("a1", Health::Healthy, 0, 50));
        fleet.update(ws("a2", Health::Healthy, 0, 200));
        assert_eq!(fleet.stale_since(100).len(), 1);
        assert_eq!(fleet.stale_since(100)[0].agent_id, "a1");
    }
}
