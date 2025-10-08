use panther_domain::ports::KeyValueStore;

pub struct SledStore {
    db: sled::Db,
}

impl SledStore {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }
}

impl KeyValueStore for SledStore {
    fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        Ok(self
            .db
            .get(key.as_bytes())?
            .map(|ivec| String::from_utf8_lossy(&ivec).to_string()))
    }

    fn set(&self, key: &str, value: String) -> anyhow::Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.db.remove(key.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sled_store_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("db");
        let store = SledStore::open(path.to_str().unwrap()).unwrap();
        store.set("k", "v".into()).unwrap();
        assert_eq!(store.get("k").unwrap(), Some("v".into()));
        store.delete("k").unwrap();
        assert_eq!(store.get("k").unwrap(), None);
    }
}

