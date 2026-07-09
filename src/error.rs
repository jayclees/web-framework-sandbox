use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct HttpError {
    code: u16,
    message: String,
}

impl HttpError {
    pub fn new(code: u16, message: String) -> HttpError {
        HttpError { code, message }
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Http error [{}]: {}", self.code, self.message)
    }
}

impl Error for HttpError {

}
