use async_trait::async_trait;
use minijinja::context;
use sturdy::action::{Action, Responsable};
use sturdy::app::App;
use sturdy::http::error::HttpError;
use sturdy::http::request::HttpRequest;

#[derive(Debug)]
pub struct Welcome {
    message: &'static str,
}

#[async_trait]
impl Action for Welcome {
    async fn handle(
        &self,
        app: &App,
        _request: HttpRequest,
    ) -> Result<Box<dyn Responsable>, HttpError> {
        let result = app.template("welcome.html", context! {
            message => self.message,
        });

        match result {
            Ok(rendered) => sturdy::action::text(rendered),
            Err(error) => Err(HttpError::new(500, error.to_string())),
        }
    }
}

impl Welcome {
    pub fn new(message: &'static str) -> Welcome {
        Welcome { message }
    }
}
