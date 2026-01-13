#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]
use auto_default::auto_default;

// nested skip is allowed

#[auto_default]
#[derive(PartialEq, Eq, Debug)]
enum Z {
    #[auto_default(skip)]
    A {
        #[auto_default(skip)]
        field: (),
        not_skip: (),
    },
}

fn main() {
    assert_eq!(
        Z::A {
            field: (),
            not_skip: ()
        },
        Z::A {
            field: (),
            not_skip: ()
        }
    );
}
