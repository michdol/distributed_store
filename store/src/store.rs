use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct KeyValueStore {
    store: Arc<RwLock<HashMap<String, Value>>>,
}

impl KeyValueStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: Value) {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) -> Option<Value> {
        let mut store = self.store.write().unwrap();
        store.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let mut kv = KeyValueStore::new();
        let key = "1".to_string();
        kv.set(key, json!(1));
        if let Some(value) = kv.get("1") {
            assert_eq!(value, 1);
        } else {
            panic!("Expected to get '1'");
        }

        kv.set("asd".to_string(), json!({"age": 100, "name": "Me"}));
        if let Some(value) = kv.get("asd") {
            assert_eq!(value.get("age").unwrap(), 100);
            assert_eq!(value.get("name").unwrap(), "Me");
        } else {
            panic!("panicking");
        }

        if let Some(value) = kv.delete("asd") {
            assert_eq!(value.get("age").unwrap(), 100);
            assert_eq!(value.get("name").unwrap(), "Me");
        }

        if let Some(value) = kv.delete("asd") {
            panic!(
                "Store returned value that is not in the store anymore {:?}",
                value
            );
        }
    }
}
