//! # `menv`
//! This crate provides [a macro](require_envs) for asserting the presence of a list of environment
//! variables and accessing them as types which implement [`FromStr`](std::str::FromStr).

/// Generate the following:
/// - A function which asserts the presence and well-formedness of a list of env vars
/// - A function which returns a `bool` representing whether any of the required vars are set
/// - A function which returns a `String` representing the collected help messages for the list of vars
/// - A list of functions, one for each environment variable required, which parse and return the associated env var
///
/// # Example
/// Here we fill an `env` module with required environment variables,
/// print help and exit if none of them are set, runs the asserts for them
/// if some are set, and proceeds.
///
/// Other parts of this program are now free, since the asserts were run at the start of main,
/// to access `env::server_port()` and `env::db_path()` as if they are infallible.
///
/// The getter function name can be suffixed with `?` to make an env var optional. In this example,
/// `plugin_dir`'s return type is `Option<String>`.
/// ```
/// mod env {
///     use menv::require_envs;
///     require_envs! {
///         (assert_env_vars, any_set, gen_help);
///
///         server_port, "FERRISCRAFT_USERS_PORT", u16,
///         "FERRISCRAFT_USERS_PORT should be set to the desired server port";
///
///         db_path, "FERRISCRAFT_USERS_DB", String,
///         "FERRISCRAFT_USERS_DB should be set to the path to the users database";
///
///         plugin_dir?, "XLANG_PLUGIN_DIR", String,
///         "XLANG_PLUGIN_DIR, if set, overrides the directory that lccc looks for xlang plugins";
///     }
/// }
/// fn main() {
///    if env::any_set() {
///         env::assert_env_vars();
///     } else {
///         println!("# Environment Variables Help\n{}", env::gen_help());
///         return
///     }
/// }
/// ```
#[macro_export]
macro_rules! require_envs {
    // We set a default visibility which is different from Rust's default to private.
    (@func $fname:ident ?, $ename:literal, $ty:ty, $etext:literal) => {
        $crate::require_envs! {@func pub $fname ?, $ename, $ty, $etext}
    };
    (@func $vis:vis $fname:ident ?, $ename:literal, $ty:ty, $etext:literal) => {
        $vis fn $fname() -> $crate::__private::Option<$ty> {
            let x = $crate::__private::env::var($ename).ok();
            x.and_then(|x| {
                $crate::__private::Option::Some($crate::__private::FromStr::from_str(&x).expect($etext))
            })
        }
    };
    (@func $fname:ident, $ename:literal, $ty:ty, $etext:literal) => {
        $crate::require_envs! {@func pub $fname, $ename, $ty, $etext}
    };
    (@func $vis:vis $fname:ident, $ename:literal, $ty:ty, $etext:literal) => {
        $vis fn $fname() -> $ty {
            $crate::__private::FromStr::from_str(&$crate::__private::env::var($ename).expect($etext)).expect($etext)
        }
    };
    // We do not assert the existence of optional variables.
    (@assert $vis:vis $fname:ident ?, $ename:literal, $ty:ty, $etext:literal) => {};
    (@assert $vis:vis $fname:ident, $ename:literal, $ty:ty, $etext:literal) => {
        let _ = $fname();
    };
    (@get_res $vis:vis $fname:ident $(?)?, $ename:literal, $ty:ty, $etext:literal) => {
        $crate::__private::env::var($ename)
    };
    (@etext $vis:vis $fname:ident $(?)?, $ename:literal, $ty:ty, $etext:literal) => {
        $etext
    };
    (($assert_name:ident, $any_set_name:ident, $help_name:ident); $($stream:tt)*) => {
        // Note: While I now use a proc macro for dividing the input stream into declarations,
        // the below comments still accurately describe what that proc macro generates invocations of.
        pub fn $assert_name () {
            $crate::__private::assert_var_body! {$crate $($stream)*}
            // $(
            //     $crate::require_envs! {@assert $a $b $c $d $e $f $g $($h)?}
            // )*
        }
        pub fn $any_set_name() -> bool {
            $crate::__private::any_set_body! {$crate $($stream)*}.iter().any(|x| x.is_ok())
            // [$($crate::require_envs! {@get_res $a $b $c $d $e $f $g $($h)?}),*].iter().any(|x| x.is_ok())
        }
        pub fn $help_name() -> $crate::__private::String {
            $crate::__private::help_body!{$crate $($stream)*}.iter().fold($crate::__private::String::new(), |a, x| {
                a + x + "\n"
            })
            // $crate::__private::String::new() $(+ $crate::require_envs! {@etext $a $b $c $d $e $f $g $($h)?} + "\n")*
        }
        $crate::__private::getters! {$crate $($stream)*}
        // $(
        //     $crate::require_envs! {@func $a $b $c $d $e $f $g $($h)?}
        // )*
        $crate::__private::errors! {$crate $($stream)*}
    }
}

/// This module holds private re-exports which are used by [`require_envs`]
/// to ensure it always refers to the right external items.
#[doc(hidden)]
pub mod __private {
    pub use ::menv_proc_macro::{any_set_body, assert_var_body, errors, getters, help_body};
    pub use ::std::env;
    pub use ::std::option::Option;
    pub use ::std::str::FromStr;
    pub use ::std::string::String;
}
