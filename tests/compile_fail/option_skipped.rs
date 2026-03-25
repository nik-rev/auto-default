#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(default_field_values)]

use auto_default::auto_default;

#[auto_default(Option)]
struct X {
    #[auto_default(skip)]
    value: Option<()>,
}

// `value` field doesn't have a default value
fn main() {
    X { .. };
}
