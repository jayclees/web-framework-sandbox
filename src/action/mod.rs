use async_trait::async_trait;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use std::convert::Infallible;

pub mod pages;

#[async_trait]
pub trait Action {
    async fn handle(&self) -> Result<Response<Full<Bytes>>, Infallible>;
    async fn log(&self) -> () {
        // Do nothing
        println!("Doing nothing...")
    }
}
