//! # git-graph
//!
//! Models git repositories as graphs for agent coordination.
//!
//! Provides graph-based data structures for reasoning about commits, branches,
//! agent topologies, message routing, memory indexing, and fleet status.

mod commit_graph;
mod branch_forest;
mod agent_topology;
mod message_route;
mod memory_index;
mod fleet_status;

pub use commit_graph::{CommitGraph, CommitNode};
pub use branch_forest::{BranchForest, BranchInfo, DivergencePoint};
pub use agent_topology::{AgentTopology, AgentAssignment, ConflictInfo};
pub use message_route::MessageRoute;
pub use memory_index::{MemoryIndex, MemoryEntry};
pub use fleet_status::{FleetStatus, WorkspaceStatus, Health};
