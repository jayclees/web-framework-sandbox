use crate::app::App;
use crate::error::HttpError;
use async_trait::async_trait;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use serde_json::Value;
use std::fmt::{Debug, Formatter};

pub mod pages;

#[async_trait]
pub trait Action {
    async fn handle(&self, app: &App) -> Result<Box<dyn Responsable>, HttpError>;
    async fn log(&self) -> () {
        // Do nothing
        println!("Doing nothing...")
    }
}

impl Debug for dyn Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::write(f, format_args!(""))
    }
}

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
