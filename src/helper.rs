#[macro_export]
macro_rules! dd {
    ( $( $x:expr ),* ) => {
        $(
            dbg!($x);
        )*
        std::process::exit(1);
    };
}
