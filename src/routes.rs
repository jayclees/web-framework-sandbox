use framework::routing::router::Router;
use crate::action::pages::{
    ServeApp, ShowAbout, ShowDeeplyNestedRoute, ShowErrorPage, ShowHtml, ShowJson, ShowLanding,
    ShowNumberArray, ShowUser,
};

pub fn register_routes(router: &mut Router) -> () {
    router.getm("/", ShowLanding, |route| route.name("landing"));
    router.get("/about", ShowAbout);
    router.get("/deeply/nested/route", ShowDeeplyNestedRoute);
    router.get("/json-array", ShowNumberArray);
    router.get("/json", ShowJson);
    router.get("/html", ShowHtml);
    router.get("/user/{user}", ShowUser);
    // router.get("/user/{user}/post/{post}", ShowUser);
    router.get("/error", ShowErrorPage);
    router.getm(
        "/home/app-{trailing_token}",
        ServeApp,
        |route| route.constrain("trailing_token", "[a-zA-Z-]+"),
   );
}
