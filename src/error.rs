use std::fmt;

pub const ERR_INVALID_DIMENSIONS: &str = "invalid bitmap dimensions: width and height must be positive";
pub const ERR_DIMENSIONS_TOO_LARGE: &str = "bitmap dimensions too large";
pub const ERR_INVALID_TEMPLATE_INDEX: &str = "invalid template index";
pub const ERR_TOO_MANY_SYMBOLS: &str = "too many symbols";
pub const ERR_INVALID_REFERENCE_CORNER: &str = "invalid reference corner";
pub const ERR_INVALID_COMBINATION_OPERATOR: &str = "invalid combination operator";

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
