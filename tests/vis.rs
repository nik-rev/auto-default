#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![allow(unused)]

use auto_default::auto_default;

#[auto_default]
#[derive(PartialEq, Debug)]
pub struct A {}

#[auto_default]
#[derive(PartialEq, Debug)]
pub(crate) struct B {}

#[rustfmt::skip]
#[auto_default]
#[derive(PartialEq, Debug)]
pub(in crate) struct C {}
