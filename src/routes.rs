use crate::action::pages::{
    ServeApp, ShowAbout, ShowDeeplyNestedRoute, ShowErrorPage, ShowHtml, ShowJson, ShowLanding,
    ShowNumberArray, ShowUser,
};
use crate::routing::router::Router;

pub fn register_routes(router: &mut Router) -> () {
    router.get("/", ShowLanding, Some(|route| route.name("landing")));
    router.get("/about", ShowAbout, None);
    router.get("/deeply/nested/route", ShowDeeplyNestedRoute, None);
    router.get("/json-array", ShowNumberArray, None);
    router.get("/json", ShowJson, None);
    router.get("/html", ShowHtml, None);
    router.get("/user/{user}", ShowUser, None);
    // router.get("/user/{user}/post/{post}", ShowUser, None);
    router.get("/error", ShowErrorPage, None);
    router.get(
        "/home/app-{wildcard}",
        ServeApp,
        Some(|route| route.wildcard("wildcard", true)),
    );
}
