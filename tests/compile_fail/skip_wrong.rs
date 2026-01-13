#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]
use auto_default::auto_default;

// Unit and Tuple variants cannot have #[auto_default(skip)] applied to them

#[auto_default]
enum Enum {
    #[auto_default(skip)]
    Unit,
    #[auto_default(skip)]
    Tuple(u32),
}

fn main() {}
