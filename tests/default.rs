#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default]
#[derive(PartialEq, Debug)]
struct X {
    default: i8 = 40,
}

#[test]
fn test() {
    assert_eq!(X { .. }, X { default: 40 });
}
