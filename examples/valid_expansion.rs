use std::str::FromStr;
use menv::Flag;
menv::require_envs! {
    (assert_env_vars, any_set, gen_help);

    hm, "HM", Hm<String>,
    "HM should be set to something.";

    lol, "LMAO", String,
    "LMAO should be set to something.";

    huh?, "HUH", String,
    "HUH can be set, but does not have to be.";

    flag~, "FLAG", bool,
    "FLAG can be set to a boolean, but defaults to true if unset.";

    better_flag~, "BETTER_FLAG", Flag,
    "BETTER_FLAG is true when set, and false when not.";
}

/// This type is just an excuse to be able to write a generic type in the above macro invocation.
pub struct Hm<T> {
    _x: T,
}
impl<T: FromStr> FromStr for Hm<T> {
    type Err = T::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            _x: s.parse()?,
        })
    }
}


fn main() {}
