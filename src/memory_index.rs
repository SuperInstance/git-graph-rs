use std::collections::HashMap;

/// A memory entry stored as a git tag.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub tag_ref: String,
    pub timestamp: u64,
}

/// Index git tags as a key-value memory store.
///
/// Agents can store and retrieve memories using tags as the backing store.
/// Keys are derived from tag names (e.g., `mem/foo` → key `foo`).
#[derive(Debug, Clone)]
pub struct MemoryIndex {
    entries: HashMap<String, MemoryEntry>,
    prefix: String,
}

impl MemoryIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            prefix: "mem/".to_string(),
        }
    }

    /// Create with a custom tag prefix.
    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            entries: HashMap::new(),
            prefix: prefix.to_string(),
        }
    }

    /// Store a key-value pair. Returns the tag reference.
    pub fn put(&mut self, key: &str, value: &str, timestamp: u64) -> String {
        let tag_ref = format!("{}{}", self.prefix, key);
        self.entries.insert(
            key.to_string(),
            MemoryEntry {
                key: key.to_string(),
                value: value.to_string(),
                tag_ref: tag_ref.clone(),
                timestamp,
            },
        );
        tag_ref
    }

    /// Retrieve a value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|e| e.value.as_str())
    }

    /// Get the full entry.
    pub fn get_entry(&self, key: &str) -> Option<&MemoryEntry> {
        self.entries.get(key)
    }

    /// Remove a key.
    pub fn remove(&mut self, key: &str) -> Option<MemoryEntry> {
        self.entries.remove(key)
    }

    /// List all keys.
    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Search for entries whose value contains the query.
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.value.contains(query))
            .collect()
    }

    /// Get entries updated after a given timestamp.
    pub fn since(&self, timestamp: u64) -> Vec<&MemoryEntry> {
        self.entries
            .values()
            .filter(|e| e.timestamp > timestamp)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut idx = MemoryIndex::new();
        idx.put("color", "blue", 100);
        assert_eq!(idx.get("color"), Some("blue"));
        assert!(idx.get("size").is_none());
    }

    #[test]
    fn test_overwrite() {
        let mut idx = MemoryIndex::new();
        idx.put("color", "blue", 100);
        idx.put("color", "red", 200);
        assert_eq!(idx.get("color"), Some("red"));
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn test_search() {
        let mut idx = MemoryIndex::new();
        idx.put("note1", "hello world", 100);
        idx.put("note2", "goodbye world", 200);
        idx.put("note3", "hello universe", 300);
        let results = idx.search("hello");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_since() {
        let mut idx = MemoryIndex::new();
        idx.put("a", "old", 50);
        idx.put("b", "new", 150);
        let results = idx.since(100);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "b");
    }
}
