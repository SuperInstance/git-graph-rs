# git-graph

Models git repositories as graphs for agent coordination.

Built on [petgraph](https://crates.io/crates/petgraph), this crate provides graph-based data structures for reasoning about commits, branches, agent topologies, message routing, memory indexing, and fleet status across multi-agent systems.

## Components

### `CommitGraph`

A directed acyclic graph (DAG) of commits with parent tracking. Edges point from child → parent, matching git's direction.

```rust
use git_graph::{CommitGraph, CommitNode};

let mut graph = CommitGraph::new();
graph.insert(CommitNode {
    hash: "abc123".into(),
    message: "initial commit".into(),
    author: "agent-1".into(),
    timestamp: 1717700000,
});
graph.add_parent("def456", "abc123");

// Find lowest common ancestor
let lca = graph.lca("branch-a-tip", "branch-b-tip");

// Topological sort
let order = graph.topological_sort();
```

### `BranchForest`

Tracks branches and their divergence points. Wraps a `CommitGraph` and maps branch names to tip commits.

```rust
use git_graph::{CommitGraph, BranchForest};

let mut forest = BranchForest::new(graph);
forest.add_branch("main", "abc123", None);
forest.add_branch("feature", "def456", Some("agent-1".into()));

let divergence = forest.find_divergence("main", "feature");
```

### `AgentTopology`

Maps agents to branches and detects merge conflicts between active agent thought branches.

```rust
use git_graph::{CommitGraph, BranchForest, AgentTopology};

let mut topo = AgentTopology::new(forest);
topo.assign("agent-1", "feature-a", "refactor", "active");
topo.assign("agent-2", "feature-b", "new-feature", "active");

let conflicts = topo.detect_conflicts();
```

### `MessageRoute`

Finds the shortest path between two commits through the DAG, treating it as undirected. This enables message routing between agents through common ancestors.

```rust
use git_graph::{CommitGraph, MessageRoute};

let router = MessageRoute::new(&graph);
let path = router.find_route("agent-a-commit", "agent-b-commit");
let distance = router.distance("commit-x", "commit-y");
```

### `MemoryIndex`

Indexes git tags as a key-value memory store. Agents can store and retrieve memories using tags as the backing store.

```rust
use git_graph::MemoryIndex;

let mut memory = MemoryIndex::new();
memory.put("config", "model=gpt-4", 1717700000);
memory.put("context", "working on auth", 1717700100);

let value = memory.get("config");
let recent = memory.since(1717700050);
let results = memory.search("auth");
```

### `FleetStatus`

Aggregates status across multiple agent workspaces — health, uncommitted changes, staleness.

```rust
use git_graph::{FleetStatus, WorkspaceStatus, Health};

let mut fleet = FleetStatus::new();
fleet.update(WorkspaceStatus {
    agent_id: "agent-1".into(),
    branch: "main".into(),
    uncommitted_changes: 0,
    last_commit_ts: 1717700000,
    health: Health::Healthy,
    active_tasks: 3,
});

let overall = fleet.overall_health();
let dirty = fleet.dirty_workspaces();
let stale = fleet.stale_since(1717600000);
```

## Installation

```toml
[dependencies]
git-graph = "0.1"
```

## License

MIT
