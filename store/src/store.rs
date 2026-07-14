use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Write};
use std::str::FromStr;

use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::constants::Operation;

pub struct KeyValueStore {
    store: Arc<RwLock<HashMap<String, Value>>>,
    flushed_lsn: u64,
    path: String,
    writer: BufWriter<File>,
}

impl KeyValueStore {
    pub fn new(path: String) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.clone())
            .unwrap();

        Self {
            path: path,
            flushed_lsn: 0,
            store: Arc::new(RwLock::new(HashMap::new())),
            writer: BufWriter::new(file),
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let store = self.store.read().unwrap();
        store.get(key).cloned()
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.write_wal(Operation::Set, &key, &value);
        self._set(key, value);
    }

    fn _set(&mut self, key: String, value: Value) {
        let mut store = self.store.write().unwrap();
        store.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) -> Option<Value> {
        self.write_wal(Operation::Delete, &key, &Value::Null);
        self._delete(key)
    }

    fn _delete(&mut self, key: &str) -> Option<Value> {
        let mut store = self.store.write().unwrap();
        store.remove(key)
    }

    fn write_wal(&mut self, operation: Operation, key: &str, value: &Value) {
        let line = self.wal_line(operation, key, value);
        writeln!(self.writer, "{}", line).unwrap();
        self.flushed_lsn += 1;
    }

    fn wal_line(&self, operation: Operation, key: &str, value: &Value) -> String {
        format!(
            "{},{},{},{},",
            self.flushed_lsn + 1,
            operation.to_string(),
            key,
            json!(value)
        )
    }

    pub fn replay_wal(&mut self) {
        let f = File::open(self.path.clone()).unwrap();
        let buf_reader = BufReader::new(f);

        for line in buf_reader.lines() {
            let wal_line = WalLine::new(line.unwrap());
            match wal_line.operation {
                Operation::Get => {}
                Operation::Set => {
                    self._set(wal_line.key, wal_line.value);
                    self.flushed_lsn = wal_line.lsn;
                }
                Operation::Delete => {
                    self._delete(&wal_line.key);
                    self.flushed_lsn = wal_line.lsn;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WalLine {
    pub lsn: u64,
    pub operation: Operation,
    pub key: String,
    pub value: Value,
}

impl WalLine {
    pub fn new(line: String) -> Self {
        let parts = line.split(",");
        let parts = parts.collect::<Vec<&str>>();
        let operation = Operation::from_str(&parts[1].to_lowercase()).unwrap();
        Self {
            lsn: parts[0].parse::<u64>().unwrap(),
            operation: operation,
            key: parts[2].to_string(),
            value: json!(parts[3]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut kv = KeyValueStore::new("./src/test_wals/test_basic.txt".to_string());
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

    #[test]
    fn test_replay_log() {
        let mut kv = KeyValueStore::new("./src/test_wals/test_wal.txt".to_string());
        kv.replay_wal();

        assert_eq!(kv.get("key_name").unwrap(), "value".to_string());
        assert_eq!(kv.get("key_name_2").unwrap(), "another_value".to_string());
    }
}
