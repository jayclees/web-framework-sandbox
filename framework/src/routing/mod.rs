pub mod route;
pub mod router;
mod tokenizer;

/// Since we want to split the route definition path and the request
/// instance path the same way we will extract it into a helper fn
fn split_segments(path: String) -> Vec<String> {
    // Special case for single slash
    if path == "/" {
        return vec!["".to_string()];
    }

    path.split("/").map(|segment| segment.to_string()).collect()
}
