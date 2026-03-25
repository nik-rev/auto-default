#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(default_field_values)]

use auto_default::auto_default;

#[auto_default(Option)]
struct X {
    value: Option<()>,
    not_option: u32,
}

// `not_option` field doesn't have a default value
fn main() {
    X { .. };
}
