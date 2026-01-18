#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default]
struct X {
    #[auto_default(skip)]
    skipped: (),
    not_skipped: (),
}

#[auto_default]
enum Foo {
    A {
        #[auto_default(skip)]
        skipped: (),
        not_skipped: (),
    },
    // test that it works on enum variants
    #[auto_default(skip)]
    B { skipped: () },
}

// each of these constructors have a `skipped` field that has no default field value.
fn main() {
    X { .. };
    Foo::A { .. };
    Foo::B { .. };
}
