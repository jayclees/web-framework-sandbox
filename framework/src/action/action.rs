use crate::action::Responsable;
use crate::app::App;
use crate::error::HttpError;
use async_trait::async_trait;
use hyper::body::Incoming;
use hyper::Request;
use std::fmt::Debug;

#[async_trait]
pub trait Action: Send + Sync + Debug {
    async fn handle(
        &self,
        app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError>;
    async fn log(&self) -> () {
        // Do nothing by default
    }
    async fn respond(
        &self,
        responsable: Box<dyn Responsable>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        Ok(responsable)
    }
}
