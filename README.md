# `menv`
This crate provides a macro (`require_envs!`) for asserting the presence of a list of environment
variables and accessing them as types which implement [`FromStr`](https://doc.rust-lang.org/stable/std/str/trait.FromStr.html).

Generate the following:
- A function which asserts the presence and well-formedness of a list of env vars
- A function which returns a `bool` representing whether any of the required vars are set
- A function which returns a `String` representing the collected help messages for the list of vars
- A list of functions, one for each environment variable required, which parse and return the associated env var

# Example
Here we fill an `env` module with required environment variables,
print help and exit if none of them are set, runs the asserts for them
if some are set, and proceeds.

Other parts of this program are now free, since the asserts were run at the start of main,
to access `env::server_port()` and `env::db_path()` as if they are infallible.

The getter function name can be suffixed with `?` to make an env var optional. In this example,
`plugin_dir`'s return type is `Option<String>`.
```rust
mod env {
    use menv::require_envs;
    require_envs! {
        (assert_env_vars, any_set, gen_help);

        server_port, "FERRISCRAFT_USERS_PORT", u16,
        "FERRISCRAFT_USERS_PORT should be set to the desired server port";

        db_path, "FERRISCRAFT_USERS_DB", String,
        "FERRISCRAFT_USERS_DB should be set to the path to the users database";

        plugin_dir?, "XLANG_PLUGIN_DIR", String,
        "XLANG_PLUGIN_DIR, if set, overrides the directory that lccc looks for xlang plugins";
    }
}
fn main() {
   if env::any_set() {
        env::assert_env_vars();
    } else {
        println!("# Environment Variables Help\n{}", env::gen_help());
        return
    }
}
```

# MSRV
This crate is tested with the latest stable version of Rust.
It probably works with many earlier ones, but I do not promise that it will in perpetuity.

I will revisit this policy at 1.0.
