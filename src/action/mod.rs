use async_trait::async_trait;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use std::convert::Infallible;

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
        let builder1 = Response::builder().header("Content-Type", "application/json");
        let result1 = serde_json::to_string(&self).unwrap();
        let result = builder1.body(Full::new(Bytes::from(result1)));
        Ok(result.unwrap())
    }
}
