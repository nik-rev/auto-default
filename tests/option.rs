#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default(Option)]
#[derive(PartialEq, Debug)]
struct X {
    not_option: u32 = 8,
    no_default: Option<i8>,
    default: Option<i8> = Some(10),
}

#[test]
fn test() {
    assert_eq!(X { .. }, X {
        not_option: 8,
        no_default: None,
        default: Some(10)
    });
}
