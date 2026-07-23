use crate::action::pages::Welcome;
use sturdy::routing::router::Router;

pub fn register_routes(router: &mut Router) -> () {
    router.getm("/", Welcome::new("Welcome to Sturdy Framework."), |route| route.name("landing"));
}
