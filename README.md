# git-graph: Model Git Repositories as Graphs for Agent Coordination

A Rust library that models git commit history as directed acyclic graphs (DAGs), branches as forests, agent assignments as topologies, and commit-graph paths as message routes. Built on `petgraph` for multi-agent coordination systems where git is the shared state.

## Why It Matters

When multiple AI agents work in the same repository — each on their own branch — the commit graph becomes the **shared coordination substrate**. Reasoning about this graph lets you answer:

- **Where did branches diverge?** (find common ancestors for merge planning)
- **Who's working on what?** (agent-to-branch topology)
- **Are two agents going to conflict?** (divergence detection)
- **How do messages route between agents?** (shortest path through commit DAG)
- **Is the fleet healthy?** (staleness, dirty workspaces, health aggregation)

This library provides all of these as composable graph algorithms.

## How It Works

### Commit Graph (DAG)

Commits are nodes; edges point from child → parent (matching git's direction). This is a **DAG** because git history is acyclic.

```
    C --- D
   /         \
  A --- B --- E (merge)
```

**Operations**:

| Operation | Algorithm | Complexity |
|-----------|-----------|------------|
| `insert(commit)` | Hash map lookup/insert | O(1) avg |
| `add_parent(child, parent)` | Edge insert + dedup | O(1) avg |
| `parents(hash)` | Outgoing neighbors | O(d) |
| `children(hash)` | Incoming neighbors | O(d) |
| `roots()` / `leaves()` | Scan for degree-0 nodes | O(V) |
| `lca(a, b)` | BFS from b through ancestor set of a | O(V + E) |
| `topological_sort()` | Kahn's algorithm (via petgraph) | O(V + E) |

### Branch Forest

Tracks named branches over the commit DAG with divergence detection:

```
find_divergence("feature", "main") → DivergencePoint {
    common_ancestor: "abc123",
    branch_a: "feature",
    branch_b: "main",
}
```

Uses `lca` on the commit DAG to find the merge base.

### Agent Topology

Maps agents to branches and detects conflicts:

```
conflict exists if:
    agent_a.status == "active" AND agent_b.status == "active"
    AND their branches have diverged from a common ancestor
```

Checks all O(N²) agent pairs, each requiring an LCA query.

### Message Routing

BFS through the commit graph treated as **undirected** (traverse both parent→child and child→parent edges), so messages can route up through common ancestors and down to sibling branches:

```
route(B → C) = B → A → C  (through common ancestor A)
```

**Complexity**: O(V + E) per BFS query.

### Fleet Status

Aggregate health across all agent workspaces. Uses a priority ordering:

```
Healthy (0) < Degraded (1) < Unhealthy (2) < Unknown (3)
overall_health = max(health_i)  // worst-case wins
```

Also tracks dirty workspaces (uncommitted changes > 0) and stale workspaces (last commit timestamp < threshold).

## Quick Start

```rust
use git_graph::{CommitGraph, CommitNode};

let mut g = CommitGraph::new();
g.insert(CommitNode { hash: "aaa".into(), message: "root".into(), author: "alice".into(), timestamp: 100 });
g.insert(CommitNode { hash: "bbb".into(), message: "child".into(), author: "bob".into(), timestamp: 200 });
g.add_parent("bbb", "aaa");

assert_eq!(g.parents("bbb"), vec!["aaa"]);
assert_eq!(g.children("aaa"), vec!["bbb"]);
assert_eq!(g.lca("aaa", "bbb"), Some("aaa".into()));
```

## API

| Module | Key Types | Purpose |
|--------|-----------|---------|
| `commit_graph` | `CommitGraph`, `CommitNode` | DAG of commits |
| `branch_forest` | `BranchForest`, `BranchInfo`, `DivergencePoint` | Branch tracking |
| `agent_topology` | `AgentTopology`, `AgentAssignment`, `ConflictInfo` | Agent ↔ branch mapping |
| `message_route` | `MessageRoute` | Shortest-path routing |
| `memory_index` | `MemoryIndex`, `MemoryEntry` | Git-tag-backed KV store |
| `fleet_status` | `FleetStatus`, `WorkspaceStatus`, `Health` | Fleet health aggregation |

## Architecture Notes

This is an **η (eta)** module — orchestration built on top of git's **γ** (the commit graph data structure). In the γ + η = C framework, git itself is the γ (deterministic content-addressed storage), and this crate is the η that adds coordination semantics: divergence detection, conflict prediction, message routing, and fleet health. The composition γ + η = C means "coordinated agent work backed by git history."

## References

- Eppstein, D. (2004). *Finding the k Shortest Paths*. SIAM J. Computing 28(2).
- petgraph: [docs.rs/petgraph](https://docs.rs/petgraph)
- Chacon, S. & Straub, B. (2014). *Pro Git* (2nd ed.). Apress.
- Hewitt, C. (1977). *Actor Model of Computation*. MIT AI Memo 410.

## License

MIT
