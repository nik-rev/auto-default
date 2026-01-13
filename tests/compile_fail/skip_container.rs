#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

// #[auto_default(skip)] is not allowed on containers

#[auto_default]
#[auto_default(skip)]
struct X {
    field: (),
}

#[auto_default]
#[auto_default(skip)]
enum Foo {
    A { field: () },
}

fn main() {}
