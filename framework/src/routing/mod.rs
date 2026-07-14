pub mod route;
pub mod router;
mod tokenizer;
pub mod action;

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_segments(path: &str) -> Vec<&str> {
    // Special case for single slash
    if path == "/" {
        return vec![""];
    }

    path.split("/").collect()
}
