#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default]
#[derive(PartialEq, Debug)]
struct X {
    no_default: i8,
    default: i8 = 10,
}

#[test]
fn test() {
    assert_eq!(
        X { .. },
        X {
            no_default: 0,
            default: 10
        }
    );
}
