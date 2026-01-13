#![feature(default_field_values)]
#![feature(const_trait_impl)]
#![feature(const_default)]

use auto_default::auto_default;

#[auto_default]
#[derive(PartialEq, Debug)]
pub enum Empty {}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum Unit {
    Unit,
    UnitNoComma
}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum Tuple {
    Tuple(u32),
    TupleNoComma(u32)
}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum Named {
    Named { a: u32 },
    NamedNoComma { a: u32 }
}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum UnitDiscriminant {
    Unit = 1,
    UnitNoComma = 2
}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum TupleDiscriminant {
    Tuple(u32) = 1,
    TupleNoComma(u32) = 2
}

#[auto_default]
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum NamedDiscriminant {
    Named { a: u32, c: u32 = 40 } = 1,
    NamedNoComma { a: u32 } = 2
}

#[test]
fn unit() {
    assert_eq!(Unit::Unit, Unit::Unit);
    assert_eq!(Unit::UnitNoComma, Unit::UnitNoComma);
    assert_eq!(UnitDiscriminant::Unit, UnitDiscriminant::Unit);
    assert_eq!(UnitDiscriminant::UnitNoComma, UnitDiscriminant::UnitNoComma);
}

#[test]
fn tuple_discriminant() {
    assert_eq!(TupleDiscriminant::Tuple(1), TupleDiscriminant::Tuple(1),);
    assert_eq!(
        TupleDiscriminant::TupleNoComma(1),
        TupleDiscriminant::TupleNoComma(1),
    );
}

#[test]
fn named_discriminant() {
    assert_eq!(
        NamedDiscriminant::Named { .. },
        NamedDiscriminant::Named { a: 0, c: 40 }
    );
}
