use num_derive::FromPrimitive;

#[derive(FromPrimitive, PartialEq, Eq, Debug, Copy, Clone)]
pub enum Screen {
    Upper = 1,
    Lower = 0,
}
