#[macro_export]
macro_rules! require_envs {
    (($assert_name:ident, $any_set_name:ident, $help_name:ident); $($fname:ident, $ename:literal, $ty:ty, $etext:literal);* $(;)?) => {
        pub fn $assert_name () {
            // TODO: comment out this body to check if vars are unused
            $(
                let _ = $fname();
            )*
        }
        pub fn $any_set_name() -> bool {
            [$(::std::env::var($ename)),*].iter().any(|x| x.is_ok())
        }
        pub fn $help_name() -> String {
            String::new() $(+ $etext + "\n")*
        }
        $(
            pub fn $fname() -> $ty {
                ::std::str::FromStr::from_str(&::std::env::var($ename).expect($etext)).expect($etext)
            }
        )*
    }
}
