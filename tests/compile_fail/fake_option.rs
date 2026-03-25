#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

type Option<T, K> = (T, K);

// `fake_option` field is not a real `Option`

#[auto_default(Option)]
#[derive(PartialEq, Debug)]
struct X {
    fake_option: Option<u8, u8>,
}

fn main() {
    X { .. };
}
