#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]
use auto_default::auto_default;

// you cannot apply 2 `skip` attributes to a single field or variant

#[auto_default]
struct X {
    #[auto_default(skip)]
    #[auto_default(skip)]
    field: (),
}

#[auto_default]
enum Z {
    A {
        #[auto_default(skip)]
        #[auto_default(skip)]
        field: (),
    },
    #[auto_default(skip)]
    #[auto_default(skip)]
    B { field: () },
}

fn main() {}
