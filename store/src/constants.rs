use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Operation {
    Get,
    Set,
    Delete,
}

impl Operation {
    pub fn as_byte(&self) -> u8 {
        match self {
            Operation::Get => 0,
            Operation::Set => 1,
            Operation::Delete => 2,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, String> {
        match byte {
            0 => Ok(Self::Get),
            1 => Ok(Self::Set),
            2 => Ok(Self::Delete),
            other => Err(format!("Unknown operation {}", other)),
        }
    }
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Operation::Get),
            "set" => Ok(Operation::Set),
            "delete" => Ok(Operation::Delete),
            _ => Err(format!("Unknown operation: {}", s)),
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
