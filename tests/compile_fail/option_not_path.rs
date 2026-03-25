#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(default_field_values)]

use auto_default::auto_default;

// we don't expect the Path to an option, just the ident

#[auto_default(::core::option::Option)]
struct X {
    value: Option<()>,
}

fn main() {}
