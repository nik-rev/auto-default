#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default(Option)]
#[derive(PartialEq, Debug)]
struct X {
    a: ::core::prelude::v1::Option<String>,
    b: ::std::prelude::v1::Option<String>,
    c: core::prelude::v1::Option<String>,
    d: std::prelude::v1::Option<String>,
    a2: ::core::option::Option<String>,
    b2: ::std::option::Option<String>,
    c2: core::option::Option<String>,
    d2: std::option::Option<String>,
}

#[test]
fn test() {
    assert_eq!(X { .. }, X {
        a: None,
        b: None,
        c: None,
        d: None,
        a2: None,
        b2: None,
        c2: None,
        d2: None,
    });
}
