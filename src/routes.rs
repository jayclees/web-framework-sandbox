use framework::routing::router::Router;
use crate::action::pages::{
    ServeApp, ShowAbout, ShowDeeplyNestedRoute, ShowErrorPage, ShowHtml, ShowJson, ShowLanding,
    ShowNumberArray, ShowUser,
};

pub fn register_routes(router: &mut Router) -> () {
    router.getm("/".to_owned(), ShowLanding, |route| route.name("landing".to_owned()));
    router.get("/about".to_owned(), ShowAbout);
    router.get("/deeply/nested/route".to_owned(), ShowDeeplyNestedRoute);
    router.get("/json-array".to_owned(), ShowNumberArray);
    router.get("/json".to_owned(), ShowJson);
    router.get("/html".to_owned(), ShowHtml);
    router.get("/user/{user}".to_owned(), ShowUser);
    // router.get("/user/{user}/post/{post}".to_owned(), ShowUser);
    router.get("/error".to_owned(), ShowErrorPage);
    router.getm(
        "/home/app-{wildcard}".to_owned(),
        ServeApp,
        |route| route.constrain("wildcard", "asdf"),
    );
}
