use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Operation {
    Get,
    Set,
    Delete,
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
