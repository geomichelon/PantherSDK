use panther_domain::ports::KeyValueStore;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct InMemoryStore {
    map: Mutex<HashMap<String, String>>,
}

impl KeyValueStore for InMemoryStore {
    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        Ok(self.map.lock().unwrap().get(key).cloned())
    }
    fn set(&self, key: &str, value: String) -> anyhow::Result<()> {
        self.map.lock().unwrap().insert(key.to_string(), value);
        Ok(())
    }
    fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.map.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_store_roundtrip() {
        let s = InMemoryStore::default();
        s.set("k", "v".into()).unwrap();
        assert_eq!(s.get("k").unwrap(), Some("v".into()));
        s.delete("k").unwrap();
        assert_eq!(s.get("k").unwrap(), None);
    }
}

