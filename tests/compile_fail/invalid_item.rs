use auto_default::auto_default;

#[auto_default(arguments)]
struct X(u32);

#[auto_default(arguments)]
struct M;

#[auto_default(arguments)]
trait Z {}

#[auto_default(arguments)]
fn x() {}

#[auto_default(arguments)]
macro_rules! x {
    () => {};
}

#[auto_default(arguments)]
mod a {}

fn main() {}
