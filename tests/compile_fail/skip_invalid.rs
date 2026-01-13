#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]
use auto_default::auto_default;

// invalid syntax of the skip attribute (#[auto_default(skip)])
#[auto_default]
struct X {
    #[auto_default(skip a)]
    a: (),
    #[auto_default(not_skip)]
    b: (),
    #[auto_default{skip}]
    c: (),
    #[auto_default()]
    d: (),
    #[auto_default]
    e: (),
}

fn main() {}
