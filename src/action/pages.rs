use crate::action::{Action, Responsable};
use async_trait::async_trait;

pub struct ShowLanding;

#[async_trait]
impl Action for ShowLanding {
    async fn handle(&self) -> Box<dyn Responsable> {
        Box::new("Home page".to_string())
    }

    async fn log(&self) -> () {
        println!("Logging first time visitor...")
    }
}

pub struct ShowAbout;

#[async_trait]
impl Action for ShowAbout {
    async fn handle(&self) -> Box<dyn Responsable> {
        Box::new("About page".to_string())
    }
}
