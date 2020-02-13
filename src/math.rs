
// calculations inspired by
// https://github.com/AcademySoftwareFoundation/openexr/blob/master/OpenEXR/IlmImf/ImfTiledMisc.cpp

//! Simple math utilities.

use std::convert::TryFrom;
use crate::error::{i32_to_usize};
use crate::error::Result;

/// Simple two-dimensional vector of any numerical type.
/// Supports only few mathematical operations
/// as this is used mainly as data struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2<T> (pub T, pub T);

impl<T> Vec2<T> {
    /// Maps all components of this vector to a new type, yielding a vector of that new type.
    pub fn map<B>(self, map: impl Fn(T) -> B) -> Vec2<B> {
        Vec2(map(self.0), map(self.1))
    }

    /// Try to convert all components of this vector to a new type,
    /// yielding either a vector of that new type, or an error.
    pub fn try_from<S>(value: Vec2<S>) -> std::result::Result<Self, T::Error> where T: TryFrom<S> {
        let x = T::try_from(value.0)?;
        let y = T::try_from(value.1)?;
        Ok(Vec2(x, y))
    }

    /// Seeing this vector as a dimension or size (width and height),
    /// this returns the area that this dimensions contains (`width * height`).
    pub fn area(self) -> T where T: std::ops::Mul<T, Output = T> {
        self.0 * self.1
    }
}



impl Vec2<i32> {

    /// Try to convert to `Vec2<usize>`, returning an error on negative numbers.
    pub fn to_usize(self, error_message: &'static str) -> Result<Vec2<usize>> {
        let x = i32_to_usize(self.0, error_message)?;
        let y = i32_to_usize(self.1, error_message)?;
        Ok(Vec2(x, y))
    }

    // Try to convert to `Vec2<u32>`, returning an error on negative numbers.
    /*pub fn to_u32(self, error_message: &'static str) -> Result<Vec2<u32>> {
        let x = i32_to_u32(self.0, error_message)?;
        let y = i32_to_u32(self.1, error_message)?;
        Ok(Vec2(x, y))
    }*/
}

impl Vec2<usize> {

    /// Panics for too large values
    pub fn to_i32(self) -> Vec2<i32> {
        let x = i32::try_from(self.0).expect("vector x coordinate too large");
        let y = i32::try_from(self.1).expect("vector y coordinate too large");
        Vec2(x, y)
    }

}

/*impl Vec2<u32> {

    /// Panics on too large value
    pub fn to_usize(self) -> Vec2<usize> {
        let x = usize::try_from(self.0).expect("max value overflow");
        let y = usize::try_from(self.1).expect("max value overflow");
        Vec2(x, y)
    }

    /// Panics on too large value
    pub fn to_i32(self) -> Vec2<i32> {
        let x = i32::try_from(self.0).expect("max value overflow");
        let y = i32::try_from(self.1).expect("max value overflow");
        Vec2(x, y)
    }
}*/

/*impl Vec2<usize> {

    /// Panics on too large value
    pub fn to_u32(self) -> Vec2<u32> {
        let x = u32::try_from(self.0).expect("max value overflow");
        let y = u32::try_from(self.1).expect("max value overflow");
        Vec2(x, y)
    }
}*/

impl<T: std::ops::Add<T>> std::ops::Add<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn add(self, other: Vec2<T>) -> Self::Output {
        Vec2(self.0 + other.0, self.1 + other.1)
    }
}

impl<T: std::ops::Sub<T>> std::ops::Sub<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn sub(self, other: Vec2<T>) -> Self::Output {
        Vec2(self.0 - other.0, self.1 - other.1)
    }
}

impl<T: std::ops::Div<T>> std::ops::Div<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn div(self, other: Vec2<T>) -> Self::Output {
        Vec2(self.0 / other.0, self.1 / other.1)
    }
}

impl<T: std::ops::Mul<T>> std::ops::Mul<Vec2<T>> for Vec2<T> {
    type Output = Vec2<T::Output>;
    fn mul(self, other: Vec2<T>) -> Self::Output {
        Vec2(self.0 * other.0, self.1 * other.1)
    }
}


/// Computes `floor(log(x)/log(2))`. Returns 0 where argument is 0.
// TODO does rust std not provide this?
pub(crate) fn floor_log_2(mut number: u32) -> u32 {
    let mut log = 0;

    // TODO check if this unrolls properly?
    while number > 1 {
        log += 1;
        number >>= 1;
    }

    log
}


/// Computes `ceil(log(x)/log(2))`. Returns 0 where argument is 0.
// taken from https://github.com/openexr/openexr/blob/master/OpenEXR/IlmImf/ImfTiledMisc.cpp
// TODO does rust std not provide this?
pub(crate) fn ceil_log_2(mut number: u32) -> u32 {
    let mut log = 0;
    let mut round_up = 0;

    // TODO check if this unrolls properly
    while number > 1 {
        if number & 1 != 0 {
            round_up = 1;
        }

        log +=  1;
        number >>= 1;
    }

    log + round_up
}


/// Round up or down in specific calculations.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RoundingMode {
    Down, Up,
}

impl RoundingMode {
    pub(crate) fn log2(self, number: usize) -> usize {
        match self {
            RoundingMode::Down => self::floor_log_2(number as u32) as usize,
            RoundingMode::Up => self::ceil_log_2(number as u32) as usize,
        }
    }

    pub(crate) fn divide(self, dividend: usize, divisor: usize) -> usize {
        match self {
            RoundingMode::Up => (dividend + divisor - 1) / divisor, // only works for positive numbers
            RoundingMode::Down => dividend / divisor,
        }
    }
}

// TODO log2 tests
