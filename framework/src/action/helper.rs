use super::Responsable;
use crate::error::HttpError;

pub fn text(string: String) -> Result<Box<dyn Responsable>, HttpError> {
    Ok(Box::new(string))
}
