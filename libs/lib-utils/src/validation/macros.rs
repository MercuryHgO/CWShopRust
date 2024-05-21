extern crate proc_macro;


#[macro_export]
macro_rules! validate {
    ( $val:expr, $($rule:expr),+ ) => {
        {
            let mut not_validated: Vec<Box<dyn std::error::Error>> = Vec::new();
            $(
                $rule.validate($val).map_err( |e| { not_validated.push(Box::new(e)); } );
            )*
            not_validated
        }
    };
}
