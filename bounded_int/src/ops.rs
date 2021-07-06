use crate::BoundedInt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundedIntOverflow {
    overflowed_by: i32,
}

impl<const LOW: i32, const HIGH: i32> Add for BoundedInt<{ LOW }, { HIGH }> {
    type Output = Result<Self, BoundedIntOverflow>;

    fn add(self, rhs: Self) -> Self::Output {
        if self.0 + rhs.0 > HIGH {
            Err(BoundedIntOverflow {
                overflowed_by: (self.0 + rhs.0) - HIGH,
            })
        } else {
            Ok(BoundedInt(self.0 + rhs.0))
        }
    }
}

impl<const LOW: i32, const HIGH: i32> Add<u16> for BoundedInt<{ LOW }, { HIGH }> {
    type Output = Result<Self, BoundedIntOverflow>;

    fn add(self, rhs: u16) -> Self::Output {
        let rhs = rhs as i32;
        if self.0 + rhs > HIGH {
            Err(BoundedIntOverflow {
                overflowed_by: (self.0 + rhs) - HIGH,
            })
        } else {
            Ok(BoundedInt(self.0 + rhs))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundedIntUnderflow {
    underflowed_by: i32,
}

impl<const LOW: i32, const HIGH: i32> Sub for BoundedInt<{ LOW }, { HIGH }> {
    type Output = Result<Self, BoundedIntUnderflow>;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.0 - rhs.0 < LOW {
            Err(BoundedIntUnderflow {
                underflowed_by: (self.0 - rhs.0) - HIGH,
            })
        } else {
            Ok(BoundedInt(self.0 - rhs.0))
        }
    }
}

impl<const LOW: i32, const HIGH: i32> Sub<u16> for BoundedInt<{ LOW }, { HIGH }> {
    type Output = Result<Self, BoundedIntUnderflow>;

    fn sub(self, rhs: u16) -> Self::Output {
        let rhs = rhs as i32;
        if self.0 - rhs < LOW {
            Err(BoundedIntUnderflow {
                underflowed_by: (self.0 - rhs) - HIGH,
            })
        } else {
            Ok(BoundedInt(self.0 - rhs))
        }
    }
}

// impl<const LOW: i32, const HIGH: i32, const RHS_LOW: i32, const RHS_HIGH: i32>
//     Mul<BoundedInt<{ RHS_LOW }, { RHS_HIGH }>> for BoundedInt<{ LOW }, { HIGH }>
// {
//     type Output = BoundedInt<{ LOW * RHS_LOW }, { HIGH * RHS_HIGH }>;

//     fn mul(self, rhs: BoundedInt<{ RHS_LOW }, { RHS_HIGH }>) -> Self::Output {
//         if self.0 - rhs.0 < LOW {
//             Err(BoundedIntUnderflow {
//                 underflowed_by: (self.0 - rhs.0) - HIGH,
//             })
//         } else {
//             Ok(BoundedInt(self.0 - rhs.0))
//         }
//     }
// }
