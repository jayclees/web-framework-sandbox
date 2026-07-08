use crate::action::{Action, Responsable};
use crate::app::App;
use async_trait::async_trait;
use sea_orm::EntityTrait;
use serde::Serialize;
use serde_json::json;
use crate::entity::user::Entity;

pub struct ShowLanding;

#[async_trait]
impl Action for ShowLanding {
    async fn handle(&self, _app: &App) -> Box<dyn Responsable> {
        Box::new("Home page".to_string())
    }

    async fn log(&self) -> () {
        println!("Logging first time visitor...")
    }
}

pub struct ShowAbout;

#[async_trait]
impl Action for ShowAbout {
    async fn handle(&self, _app: &App) -> Box<dyn Responsable> {
        Box::new("About page".to_string())
    }
}

pub struct ShowNumberArray;

#[async_trait]
impl Action for ShowNumberArray {
    async fn handle(&self, _app: &App) -> Box<dyn Responsable> {
        let mut vec = Vec::new();
        vec.push(5);
        Box::new(vec)
        // Box::new([1, 2, 3, 4, 5])
    }
}

pub struct ShowJson;

#[async_trait]
impl Action for ShowJson {
    async fn handle(&self, _app: &App) -> Box<dyn Responsable> {
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

pub struct ShowHtml;

#[async_trait]
impl Action for ShowHtml {
    async fn handle(&self, app: &App) -> Box<dyn Responsable> {
        let template = app.template().get_template("landing.html").unwrap();
        let result = template.render(ExampleStruct {
            app_title: "Bus Web Framework".to_string(),
            name: "John Doe".to_string(),
        });
        Box::new(result.unwrap())
    }
}

#[derive(Serialize)]
struct ExampleStruct {
    app_title: String,
    name: String,
}


pub struct ShowDatabaseModel;

#[async_trait]
impl Action for ShowDatabaseModel {
    async fn handle(&self, app: &App) -> Box<dyn Responsable> {
        let result = Entity::find_by_id(1).one(app.db()).await.unwrap().unwrap();
        Box::new(json!(result))
    }
}
