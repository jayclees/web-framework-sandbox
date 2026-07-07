use crate::action::{Action, Responsable};
use async_trait::async_trait;
use serde_json::json;

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

pub struct ShowNumberArray;

#[async_trait]
impl Action for ShowNumberArray {
    async fn handle(&self) -> Box<dyn Responsable> {
        let mut vec = Vec::new();
        vec.push(5);
        Box::new(vec)
        // Box::new([1, 2, 3, 4, 5])
    }
}

pub struct ShowJson;

#[async_trait]
impl Action for ShowJson {
    async fn handle(&self) -> Box<dyn Responsable> {
        let full_name = "John Doe";
        let age_last_year = 42;
        let john = json!({
            "name": full_name,
            "age": age_last_year + 1,
            "phones": [
                format!("+44 {}", 123)
            ]
        });

        Box::new(john)
    }
}
