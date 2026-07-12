use crate::action::{Action, Responsable};
use crate::app::App;
use crate::entity::user::Entity as User;
use crate::error::HttpError;
use async_trait::async_trait;
use hyper::body::Incoming;
use hyper::Request;
use sea_orm::EntityTrait;
use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct ShowLanding;

#[async_trait]
impl Action for ShowLanding {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        Ok(Box::new("Home page".to_string()))
    }
}

#[derive(Debug)]
pub struct ShowAbout;

#[async_trait]
impl Action for ShowAbout {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        Ok(Box::new("About page".to_string()))
    }
}

#[derive(Debug)]
pub struct ShowDeeplyNestedRoute;

#[async_trait]
impl Action for ShowDeeplyNestedRoute {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        Ok(Box::new("Deeply nested route".to_string()))
    }
}

#[derive(Debug)]
pub struct ShowNumberArray;

#[async_trait]
impl Action for ShowNumberArray {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let mut vec = Vec::new();
        vec.push(5);
        Ok(Box::new(vec))
    }
}

#[derive(Debug)]
pub struct ShowJson;

#[async_trait]
impl Action for ShowJson {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let full_name = "John Doe";
        let age_last_year = 42;
        let john = json!({
            "name": full_name,
            "age": age_last_year + 1,
            "phones": [
                format!("+44 {}", 123)
            ]
        });

        Ok(Box::new(john))
    }
}

#[derive(Debug)]
pub struct ShowHtml;

#[async_trait]
impl Action for ShowHtml {
    async fn handle(
        &self,
        app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let template = app.template().get_template("landing.html").unwrap();
        let result = template.render(ExampleStruct {
            app_title: "Bus Web Framework".to_string(),
            name: "John Doe".to_string(),
        });
        Ok(Box::new(result.unwrap()))
    }
}

#[derive(Serialize)]
struct ExampleStruct {
    app_title: String,
    name: String,
}

#[derive(Debug)]
pub struct ShowUser;

#[async_trait]
impl Action for ShowUser {
    async fn handle(
        &self,
        app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let connection = app.db().unwrap();
        let result = User::find_by_id(1).one(connection).await.unwrap();

        Ok(Box::new(json!(result)))
    }
}

#[derive(Debug)]
pub struct ShowErrorPage;

#[async_trait]
impl Action for ShowErrorPage {
    async fn handle(
        &self,
        _app: &App,
        request: Request<Incoming>,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let code = 400;
        Err(HttpError::new(
            code,
            format!("Error ({}). Please change request and try again.", code),
        ))
    }
}
