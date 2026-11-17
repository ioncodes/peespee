use std::fmt;
use std::ops::Add;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Address(pub u32);

impl Address {
    pub const fn wrapping_add(self, offset: usize) -> Self {
        Self(self.0.wrapping_add(offset as u32))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#010X}", self.0)
    }
}

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Address(value)
    }
}

impl From<Address> for u32 {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<Address> for usize {
    fn from(address: Address) -> Self {
        address.0 as usize
    }
}

impl Add<Address> for Address {
    type Output = Self;

    fn add(self, rhs: Address) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
