#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use std::collections::HashMap;

use auto_default::auto_default;

#[auto_default]
#[derive(PartialEq, Debug)]
struct X<'a, 'b> {
    a: Option<HashMap<&'a str, &'b str>> = None,
    b: Option<HashMap<&'a str, &'b str>>
}

#[test]
fn test() {
    assert_eq!(X { .. }, X { a: None, b: None });
}
