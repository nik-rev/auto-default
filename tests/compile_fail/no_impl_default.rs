#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

// struct does not implement default, we need
// to ensure that span of the error points to the correct place

use auto_default::auto_default;

struct DoesNotImplDefault;

#[auto_default]
struct X {
    default: DoesNotImplDefault,
}

fn main() {}
