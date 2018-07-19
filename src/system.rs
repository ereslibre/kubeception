use std;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

pub struct System {}

pub enum SystemError {
    UnknownError,
}

impl fmt::Debug for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SystemError")
    }
}

impl From<std::io::Error> for SystemError {
    fn from(_error: std::io::Error) -> Self {
        SystemError::UnknownError
    }
}

impl System {
    pub fn hostname() -> Result<String, SystemError> {
        let mut file = File::open("/etc/hostname")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.trim().to_string())
    }
}
