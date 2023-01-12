#[macro_export]
macro_rules! require_envs {
    ($assert_name:ident; $($fname:ident, $ename:literal, $ty:ty, $etext:literal);* $(;)?) => {
        pub fn $assert_name () {
            // TODO: comment out this body to check if vars are unused
            $(
                let _ = $fname();
            )*
        }
        $(
            pub fn $fname() -> $ty {
                ::std::str::FromStr::from_str(&::std::env::var($ename).expect($etext)).expect($etext)
            }
        )*
    }
}
