use crate::error::HttpError;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use serde_json::Value;

pub trait Responsable: Send {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, HttpError>;
}

impl Responsable for String {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, HttpError> {
        let slice = self.to_string();
        Ok(Response::new(Full::new(Bytes::from(slice))))
    }
}

impl Responsable for Vec<usize> {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, HttpError> {
        let json = serde_json::to_string(&self).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}

impl<const N: usize> Responsable for [usize; N] {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, HttpError> {
        let json = serde_json::to_string(&self.to_vec()).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}

impl Responsable for Value {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, HttpError> {
        let json = serde_json::to_string(&self).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}
