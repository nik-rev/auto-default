#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default(Option)]
#[derive(PartialEq, Debug)]
struct X {
    #[auto_default(skip)]
    not_option: u32,
}

fn main() {}
