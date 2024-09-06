use anyhow::Result;
use serde_json;

#[derive(Default)]
pub struct Node {
    #[allow(dead_code)]
    id: String, // include it as the src of any message it sends.
    #[allow(dead_code)]
    node_ids: Vec<String>,
    // I'm not sure I love the idea of storing the entire JSON object
    store: Vec<serde_json::Value>,
}

impl Node {
    pub fn init(&mut self, node_id: String, node_ids: Vec<String>) -> Self {
        Self {
            id: node_id,
            node_ids,
            store: Vec::<serde_json::Value>::new(),
        }
    }

    pub fn store(&mut self, v: serde_json::Value) -> Result<()> {
        self.store.push(v);
        Ok(())
    }

    pub fn retreive_seen_messages(&mut self) -> Result<&Vec<serde_json::Value>> {
        Ok(&self.store)
    }
}
