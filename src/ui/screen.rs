use num_derive::FromPrimitive;

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum Screen {
    Upper = 1,
    Lower = 0,
}
