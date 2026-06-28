use crate::action::Action;
use async_trait::async_trait;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use std::convert::Infallible;

pub struct ShowLanding;

#[async_trait]
impl Action for ShowLanding {
    async fn handle(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        Ok(Response::new(Full::new(Bytes::from("Home Page"))))
    }
    async fn log(&self) -> () {
        // Do nothing
        println!("Logging first time visitor...")
    }
}

pub struct ShowAbout;

#[async_trait]
impl Action for ShowAbout {
    async fn handle(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        Ok(Response::new(Full::new(Bytes::from("About Page"))))
    }
}
