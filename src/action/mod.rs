use async_trait::async_trait;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use std::convert::Infallible;
use serde_json::Value;

pub mod pages;

#[async_trait]
pub trait Action {
    async fn handle(&self) -> Box<dyn Responsable>;
    async fn log(&self) -> () {
        // Do nothing
        println!("Doing nothing...")
    }
}

pub trait Responsable {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, Infallible>;
}

impl Responsable for String {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        let slice = self.to_string();
        Ok(Response::new(Full::new(Bytes::from(slice))))
    }
}

impl Responsable for Vec<usize> {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        let json = serde_json::to_string(&self).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}

impl<const N: usize> Responsable for [usize; N] {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        let json = serde_json::to_string(&self.to_vec()).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}

impl Responsable for Value {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        let json = serde_json::to_string(&self).unwrap();

        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap())
    }
}
