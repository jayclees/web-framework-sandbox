#[macro_export]
macro_rules! dd {
    ( $( $x:expr ),* ) => {
        $(
            dbg!($x);
        )*
        std::process::exit(1);
    };
}

pub use dd;

#[macro_export]
macro_rules! get_line {
    () => {
        format!("{}:{}:{}", file!(), line!(), column!())
    };
}

pub use get_line;
