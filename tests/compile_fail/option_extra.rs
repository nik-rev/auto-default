#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(default_field_values)]

use auto_default::auto_default;

// extra token "extra"

#[auto_default(Option extra)]
struct X {
    value: Option<()>,
}

fn main() {}
