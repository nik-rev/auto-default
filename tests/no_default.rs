#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default]
struct Hello {
    no_default: i8,
}
