use serde_json::{Value, json};
use std::str::FromStr;

use crate::constants::Operation;

#[derive(Debug)]
pub struct WalLine {
    pub lsn: u64,
    pub operation: Operation,
    pub key: String,
    pub value: Value,
}

impl WalLine {
    pub fn from_string(line: String) -> Self {
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

    pub fn to_string(&self) -> String {
        format!(
            "{},{},{},{},",
            self.lsn,
            self.operation.to_string(),
            self.key,
            json!(self.value)
        )
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut buff = Vec::new();

        // Fixed 8 bytes
        buff.extend_from_slice(&self.lsn.to_be_bytes());

        // 1 byte
        buff.push(self.operation.as_byte());

        // Append key
        let key_bytes = self.key.as_bytes();
        // 4 bytes describing the lenght of the actual key (key is stored in bytes)
        buff.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());
        // The key as bytes
        buff.extend_from_slice(key_bytes);

        // Append value
        let value_bytes = serde_json::to_vec(&self.value).unwrap();
        // 4 bytes describing the lenght of the actual value (value is stored in bytes)
        buff.extend_from_slice(&(value_bytes.len() as u32).to_be_bytes());
        // The value as bytes
        buff.extend_from_slice(&value_bytes);

        Ok(buff)
    }

    pub fn from_bytes(buff: &[u8]) -> Result<Self, serde_json::Error> {
        let mut pos = 0;
        let lsn = u64::from_be_bytes(buff[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let operation = Operation::from_byte(buff[pos]).unwrap();
        pos += 1;
        let key_length = u32::from_be_bytes(buff[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let key = String::from_utf8(buff[pos..pos + key_length].to_vec()).unwrap();
        pos += key_length;
        let value_length = u32::from_be_bytes(buff[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let value = serde_json::from_slice(&buff[pos..pos + value_length]).unwrap();

        Ok(Self {
            lsn,
            operation,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_bytes() {
        let line = String::from("1,SET,key_name,value,");
        let wl = WalLine::from_string(line);
        let wl_bytes = wl.to_bytes().expect("Serialization should have succeeded");

        let mut expected = Vec::new();
        expected.extend_from_slice(&1u64.to_be_bytes());
        expected.push(Operation::Set.as_byte());
        expected.extend_from_slice(&8u32.to_be_bytes());
        expected.extend_from_slice(b"key_name");
        let value_json = serde_json::to_vec(&json!("value")).unwrap();
        expected.extend_from_slice(&(value_json.len() as u32).to_be_bytes());
        expected.extend_from_slice(&value_json);

        assert_eq!(wl_bytes, expected);
    }

    #[test]
    fn test_to_bytes_and_back() {
        let line = String::from("1,SET,key_name,value,");
        let wl = WalLine::from_string(line);
        let wl_bytes = wl.to_bytes().expect("Serialization should have succeeded");

        let wl_from_bytes = WalLine::from_bytes(&wl_bytes).unwrap();

        assert_eq!(wl_from_bytes.lsn, 1);
        assert_eq!(wl_from_bytes.operation, Operation::Set);
        assert_eq!(wl_from_bytes.key, String::from("key_name"));
        assert_eq!(wl_from_bytes.value, json!("value"));
    }

    #[test]
    fn test_to_string() {
        let line = String::from("1,Set,key_name,value,");
        let wl = WalLine::from_string(line.clone());

        let string_line = wl.to_string();
        assert_eq!(string_line, "1,Set,key_name,\"value\",".to_string());
    }
}
