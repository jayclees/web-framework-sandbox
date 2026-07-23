use crate::action::pages::Welcome;
use sturdy::routing::router::Router;

pub fn register_routes(router: &mut Router) -> () {
    router.get("/", Welcome::new("Welcome to Sturdy Framework"));
}
