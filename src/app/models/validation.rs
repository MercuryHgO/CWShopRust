use std::{collections::HashMap, error::Error, fmt};

pub type ValidtionDetails = HashMap<String,Vec<String>>;

#[derive(Debug,Clone)]
pub struct ValidationError {
    message: String,
    details: ValidtionDetails,
}

impl ValidationError {
    pub fn new(message: &str, details: ValidtionDetails) -> Self {
        ValidationError {
            message: message.into(),
            details
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let details_str: Vec<String> = self.details.iter().map(
            |(k,v)| {
                format!("{}: {:?}", k, v)
            }
        ).collect();
        write!(f, "Validation error: {}", details_str.join(", "))
    }
}

impl Error for ValidationError {}
