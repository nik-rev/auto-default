#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(default_field_values)]

use auto_default::auto_default;

// needs to be an Option:

#[auto_default(invalid)]
struct X {
    value: Option<()>,
}

fn main() {}
