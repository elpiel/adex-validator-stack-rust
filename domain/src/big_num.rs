use std::convert::TryFrom;
use std::error::Error;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Sub};
use std::str::FromStr;

use num::rational::Ratio;
use num::Integer;
use num_bigint::BigUint;
use num_derive::{Num, NumOps, One, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, NumOps, One, Zero, Num,
)]
pub struct BigNum(
    #[serde(
        deserialize_with = "biguint_from_str",
        serialize_with = "biguint_to_str"
    )]
    BigUint,
);

impl BigNum {
    pub fn new(num: BigUint) -> Result<Self, super::DomainError> {
        Ok(Self(num))
    }

    pub fn div_floor(&self, other: &Self) -> Self {
        Self(self.0.div_floor(&other.0))
    }

    pub fn to_f64(&self) -> Option<f64> {
        use num::traits::cast::ToPrimitive;

        self.0.to_f64()
    }

    pub fn to_u64(&self) -> Option<u64> {
        use num::traits::cast::ToPrimitive;

        self.0.to_u64()
    }
}

impl Integer for BigNum {
    fn div_floor(&self, other: &Self) -> Self {
        self.0.div_floor(&other.0).into()
    }

    fn mod_floor(&self, other: &Self) -> Self {
        self.0.mod_floor(&other.0).into()
    }

    fn gcd(&self, other: &Self) -> Self {
        self.0.gcd(&other.0).into()
    }

    fn lcm(&self, other: &Self) -> Self {
        self.0.lcm(&other.0).into()
    }

    fn divides(&self, other: &Self) -> bool {
        self.0.divides(&other.0)
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        self.0.is_multiple_of(&other.0)
    }

    fn is_even(&self) -> bool {
        self.0.is_even()
    }

    fn is_odd(&self) -> bool {
        !self.is_even()
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        let (quotient, remainder) = self.0.div_rem(&other.0);

        (quotient.into(), remainder.into())
    }
}

impl Add<&BigNum> for &BigNum {
    type Output = BigNum;

    fn add(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 + &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl AddAssign<&BigNum> for BigNum {
    fn add_assign(&mut self, rhs: &BigNum) {
        self.0 += &rhs.0
    }
}

impl Sub<&BigNum> for &BigNum {
    type Output = BigNum;

    fn sub(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 - &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl Div<&BigNum> for &BigNum {
    type Output = BigNum;

    fn div(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 / &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl Div<&BigNum> for BigNum {
    type Output = BigNum;

    fn div(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 / &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl Mul<&BigNum> for &BigNum {
    type Output = BigNum;

    fn mul(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 * &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl Mul<&BigNum> for BigNum {
    type Output = BigNum;

    fn mul(self, rhs: &BigNum) -> Self::Output {
        let big_uint = &self.0 * &rhs.0;
        BigNum(big_uint.to_owned())
    }
}

impl<'a> Sum<&'a BigNum> for BigNum {
    fn sum<I: Iterator<Item = &'a BigNum>>(iter: I) -> Self {
        let sum_uint = iter.map(|big_num| &big_num.0).sum();

        Self(sum_uint)
    }
}

impl Mul<&Ratio<BigNum>> for &BigNum {
    type Output = BigNum;

    fn mul(self, rhs: &Ratio<BigNum>) -> Self::Output {
        // perform multiplication first!
        (self * rhs.numer()) / rhs.denom()
    }
}

impl Mul<&Ratio<BigNum>> for BigNum {
    type Output = BigNum;

    fn mul(self, rhs: &Ratio<BigNum>) -> Self::Output {
        // perform multiplication first!
        (self * rhs.numer()) / rhs.denom()
    }
}

impl TryFrom<&str> for BigNum {
    type Error = super::DomainError;

    fn try_from(num: &str) -> Result<Self, Self::Error> {
        let big_uint = BigUint::from_str(&num)
            .map_err(|err| super::DomainError::InvalidArgument(err.description().to_string()))?;

        Ok(Self(big_uint))
    }
}

impl ToString for BigNum {
    fn to_string(&self) -> String {
        self.0.to_str_radix(10)
    }
}

impl From<u64> for BigNum {
    fn from(value: u64) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<BigUint> for BigNum {
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

fn biguint_from_str<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let num = String::deserialize(deserializer)?;
    Ok(BigUint::from_str(&num).map_err(serde::de::Error::custom)?)
}

fn biguint_to_str<S>(num: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&num.to_str_radix(10))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bignum_mul_by_ratio() {
        let big_num: BigNum = 50.into();
        let ratio: Ratio<BigNum> = (23.into(), 100.into()).into();

        let expected: BigNum = 11.into();
        assert_eq!(expected, &big_num * &ratio);
    }
}
