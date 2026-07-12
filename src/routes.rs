use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Response;
use crate::action::Action;
use crate::error::HttpError;
use crate::router::{Route, Router};

pub fn register_routes(router: &mut Router) -> () {
    router.add(Route::get("/", Action {
        handler: hello
    }).name("landing"));
    // router.add(Route::get("/about", Box::new(ShowAbout)));
    // router.add(Route::get(
    //     "/deeply/nested/route",
    //     Box::new(ShowDeeplyNestedRoute),
    // ));
    // router.add(Route::get("/json-array", Box::new(ShowNumberArray)));
    // router.add(Route::get("/json", Box::new(ShowJson)));
    // router.add(Route::get("/html", Box::new(ShowHtml)));
    // router.add(Route::get("/user/{user}", Box::new(ShowUser)));
    // // router.add(Route::get("/user/{user}/post/{post}", Box::new(ShowUser)));
    // router.add(Route::get("/error", Box::new(ShowErrorPage)));
    // // router.add(Route::get("/app/{wildcard}", Box::new(ShowErrorPage)).constrain("wildcard", "/regexpattern/"));
}

fn to_action(handler: fn() -> Result<Response<Full<Bytes>>, HttpError>) -> Action {
    Action { handler }
}

fn hello() -> Result<Response<Full<Bytes>>, HttpError> {
    Ok(Response::new(Full::new(Bytes::from("hello"))))
}
