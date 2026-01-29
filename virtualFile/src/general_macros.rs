
#[macro_export]
macro_rules! test_init {
    ($($var:ident),+) => {{
        let mut missing = Vec::new();
        $(
            if $var.is_none() {
                missing.push(stringify!($var).to_string());
            }
        )+
        if missing.is_empty() {
            Ok(())
        } else {
            Err(Box::new(GlobalError::UninitializedVariable(missing.join(", "))))
        }
    }};
}
#[macro_export]
macro_rules! iter_tuple {
    (($($item:expr),*), $var:ident, $body:block) => {
        $(let $var = $item; $body)*
    };
    ($tuple:ident,$var:ident,$body:block) => {
        let ($($tuple)*,) = $tuple;
    };
}

