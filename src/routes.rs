use framework::routing::router::Router;
use crate::action::pages::{
    ServeApp, ShowAbout, ShowDeeplyNestedRoute, ShowErrorPage, ShowHtml, ShowJson, ShowLanding,
    ShowNumberArray, ShowUser,
};

pub fn register_routes(router: &mut Router) -> () {
    router.getm("/".to_owned(), ShowLanding, |route| route.name("landing".to_owned()));
    router.get("/about".to_owned(), ShowAbout);
    // router.get("/deeply/nested/route", ShowDeeplyNestedRoute);
    // router.get("/json-array", ShowNumberArray);
    // router.get("/json", ShowJson);
    // router.get("/html", ShowHtml);
    // router.get("/user/{user}", ShowUser);
    // // router.get("/user/{user}/post/{post}", ShowUser);
    // router.get("/error", ShowErrorPage);
    // router.getm(
    //     "/home/app-{wildcard}",
    //     ServeApp,
    //     |route| route.constrain("wildcard", "asdf"),
    // );
}
