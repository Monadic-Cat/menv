//! # `menv`
//! This crate provides [a macro](require_envs) for asserting the presence of a list of environment
//! variables and accessing them as types which implement [`FromStr`](std::str::FromStr).

/// Generate the following:
/// - A function which asserts the presence and well-formedness of a list of env vars
/// - A function which returns a `bool` representing whether any of the required vars are set
/// - A function which returns a `String` representing the collected help messages for the list of vars
/// - A list of functions, one for each environment variable required
///
/// # Example
/// Here we fill an `env` module with required environment variables,
/// print help and exit if none of them are set, runs the asserts for them
/// if some are set, and proceeds.
/// Other parts of this program are now free, since the asserts were run at the start of main,
/// to access `env::server_port()` and `env::db_path()` as if they are infallible.
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
