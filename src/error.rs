use std::fmt;

#[derive(Debug)]
pub struct Jbig2Error {
    pub message: String,
}

impl Jbig2Error {
    pub fn new(msg: &str) -> Self {
        Jbig2Error {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for Jbig2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Jbig2Error: {}", self.message)
    }
}

impl std::error::Error for Jbig2Error {}
