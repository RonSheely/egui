use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::Vec2b;

/// A vector has a direction and length.
/// A [`Vec2`] is often used to represent a size.
///
/// emath represents positions using [`crate::Pos2`].
///
/// Normally the units are points (logical pixels).
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub struct Vec2 {
    /// Rightwards. Width.
    pub x: f32,

    /// Downwards. Height.
    pub y: f32,
}

/// `vec2(x, y) == Vec2::new(x, y)`
#[inline(always)]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

// ----------------------------------------------------------------------------
// Compatibility and convenience conversions to and from [f32; 2]:

impl From<[f32; 2]> for Vec2 {
    #[inline(always)]
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl From<&[f32; 2]> for Vec2 {
    #[inline(always)]
    fn from(v: &[f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl From<Vec2> for [f32; 2] {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl From<&Vec2> for [f32; 2] {
    #[inline(always)]
    fn from(v: &Vec2) -> Self {
        [v.x, v.y]
    }
}

// ----------------------------------------------------------------------------
// Compatibility and convenience conversions to and from (f32, f32):

impl From<(f32, f32)> for Vec2 {
    #[inline(always)]
    fn from(v: (f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<&(f32, f32)> for Vec2 {
    #[inline(always)]
    fn from(v: &(f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<Vec2> for (f32, f32) {
    #[inline(always)]
    fn from(v: Vec2) -> Self {
        (v.x, v.y)
    }
}

impl From<&Vec2> for (f32, f32) {
    #[inline(always)]
    fn from(v: &Vec2) -> Self {
        (v.x, v.y)
    }
}

impl From<Vec2b> for Vec2 {
    #[inline(always)]
    fn from(v: Vec2b) -> Self {
        Self {
            x: v.x as i32 as f32,
            y: v.y as i32 as f32,
        }
    }
}

// ----------------------------------------------------------------------------
// Mint compatibility and convenience conversions

#[cfg(feature = "mint")]
impl From<mint::Vector2<f32>> for Vec2 {
    #[inline]
    fn from(v: mint::Vector2<f32>) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "mint")]
impl From<Vec2> for mint::Vector2<f32> {
    #[inline]
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

// ----------------------------------------------------------------------------

impl Vec2 {
    /// Right
    pub const X: Self = Self { x: 1.0, y: 0.0 };

    /// Down
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    /// +X
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0 };

    /// -X
    pub const LEFT: Self = Self { x: -1.0, y: 0.0 };

    /// -Y
    pub const UP: Self = Self { x: 0.0, y: -1.0 };

    /// +Y
    pub const DOWN: Self = Self { x: 0.0, y: 1.0 };

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub const INFINITY: Self = Self::splat(f32::INFINITY);
    pub const NAN: Self = Self::splat(f32::NAN);

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Set both `x` and `y` to the same value.
    #[inline(always)]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// Treat this vector as a position.
    /// `v.to_pos2()` is equivalent to `Pos2::default() + v`.
    #[inline(always)]
    pub fn to_pos2(self) -> crate::Pos2 {
        crate::Pos2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Safe normalize: returns zero if input is zero.
    #[must_use]
    #[inline(always)]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len <= 0.0 { self } else { self / len }
    }

    /// Checks if `self` has length `1.0` up to a precision of `1e-6`.
    #[inline(always)]
    pub fn is_normalized(self) -> bool {
        (self.length_sq() - 1.0).abs() < 2e-6
    }

    /// Rotates the vector by 90°, i.e positive X to positive Y
    /// (clockwise in egui coordinates).
    #[inline(always)]
    pub fn rot90(self) -> Self {
        vec2(self.y, -self.x)
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        self.x.hypot(self.y)
    }

    #[inline(always)]
    pub fn length_sq(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Measures the angle of the vector.
    ///
    /// ```
    /// # use emath::Vec2;
    /// use std::f32::consts::TAU;
    ///
    /// assert_eq!(Vec2::ZERO.angle(), 0.0);
    /// assert_eq!(Vec2::angled(0.0).angle(), 0.0);
    /// assert_eq!(Vec2::angled(1.0).angle(), 1.0);
    /// assert_eq!(Vec2::X.angle(), 0.0);
    /// assert_eq!(Vec2::Y.angle(), 0.25 * TAU);
    ///
    /// assert_eq!(Vec2::RIGHT.angle(), 0.0);
    /// assert_eq!(Vec2::DOWN.angle(), 0.25 * TAU);
    /// assert_eq!(Vec2::UP.angle(), -0.25 * TAU);
    /// ```
    #[inline(always)]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Create a unit vector with the given CW angle (in radians).
    /// * An angle of zero gives the unit X axis.
    /// * An angle of 𝞃/4 = 90° gives the unit Y axis.
    ///
    /// ```
    /// # use emath::Vec2;
    /// use std::f32::consts::TAU;
    ///
    /// assert_eq!(Vec2::angled(0.0), Vec2::X);
    /// assert!((Vec2::angled(0.25 * TAU) - Vec2::Y).length() < 1e-5);
    /// ```
    #[inline(always)]
    pub fn angled(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        vec2(cos, sin)
    }

    #[must_use]
    #[inline(always)]
    pub fn floor(self) -> Self {
        vec2(self.x.floor(), self.y.floor())
    }

    #[must_use]
    #[inline(always)]
    pub fn round(self) -> Self {
        vec2(self.x.round(), self.y.round())
    }

    #[must_use]
    #[inline(always)]
    pub fn ceil(self) -> Self {
        vec2(self.x.ceil(), self.y.ceil())
    }

    #[must_use]
    #[inline]
    pub fn abs(self) -> Self {
        vec2(self.x.abs(), self.y.abs())
    }

    /// True if all members are also finite.
    #[inline(always)]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// True if any member is NaN.
    #[inline(always)]
    pub fn any_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    #[must_use]
    #[inline]
    pub fn min(self, other: Self) -> Self {
        vec2(self.x.min(other.x), self.y.min(other.y))
    }

    #[must_use]
    #[inline]
    pub fn max(self, other: Self) -> Self {
        vec2(self.x.max(other.x), self.y.max(other.y))
    }

    /// The dot-product of two vectors.
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the minimum of `self.x` and `self.y`.
    #[must_use]
    #[inline(always)]
    pub fn min_elem(self) -> f32 {
        self.x.min(self.y)
    }

    /// Returns the maximum of `self.x` and `self.y`.
    #[inline(always)]
    #[must_use]
    pub fn max_elem(self) -> f32 {
        self.x.max(self.y)
    }

    /// Swizzle the axes.
    #[inline]
    #[must_use]
    pub fn yx(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    #[must_use]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }
}

impl std::ops::Index<usize> for Vec2 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of bounds: {index}"),
        }
    }
}

impl std::ops::IndexMut<usize> for Vec2 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Vec2 index out of bounds: {index}"),
        }
    }
}

impl Eq for Vec2 {}

impl Neg for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        vec2(-self.x, -self.y)
    }
}

impl AddAssign for Vec2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

impl SubAssign for Vec2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

impl Add for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Element-wise multiplication
impl Mul<Self> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, vec: Self) -> Self {
        Self {
            x: self.x * vec.x,
            y: self.y * vec.y,
        }
    }
}

/// Element-wise division
impl Div<Self> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    #[inline(always)]
    fn mul(self, vec: Vec2) -> Vec2 {
        Vec2 {
            x: self * vec.x,
            y: self * vec.y,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, factor: f32) -> Self {
        Self {
            x: self.x / factor,
            y: self.y / factor,
        }
    }
}

impl fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "[{1:.0$} {2:.0$}]", precision, self.x, self.y)
        } else {
            write!(f, "[{:.1} {:.1}]", self.x, self.y)
        }
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        self.x.fmt(f)?;
        f.write_str(" ")?;
        self.y.fmt(f)?;
        f.write_str("]")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! almost_eq {
        ($left: expr, $right: expr) => {
            let left = $left;
            let right = $right;
            assert!((left - right).abs() < 1e-6, "{} != {}", left, right);
        };
    }

    #[test]
    fn test_vec2() {
        use std::f32::consts::TAU;

        assert_eq!(Vec2::ZERO.angle(), 0.0);
        assert_eq!(Vec2::angled(0.0).angle(), 0.0);
        assert_eq!(Vec2::angled(1.0).angle(), 1.0);
        assert_eq!(Vec2::X.angle(), 0.0);
        assert_eq!(Vec2::Y.angle(), 0.25 * TAU);

        assert_eq!(Vec2::RIGHT.angle(), 0.0);
        assert_eq!(Vec2::DOWN.angle(), 0.25 * TAU);
        almost_eq!(Vec2::LEFT.angle(), 0.50 * TAU);
        assert_eq!(Vec2::UP.angle(), -0.25 * TAU);

        let mut assignment = vec2(1.0, 2.0);
        assignment += vec2(3.0, 4.0);
        assert_eq!(assignment, vec2(4.0, 6.0));

        let mut assignment = vec2(4.0, 6.0);
        assignment -= vec2(1.0, 2.0);
        assert_eq!(assignment, vec2(3.0, 4.0));

        let mut assignment = vec2(1.0, 2.0);
        assignment *= 2.0;
        assert_eq!(assignment, vec2(2.0, 4.0));

        let mut assignment = vec2(2.0, 4.0);
        assignment /= 2.0;
        assert_eq!(assignment, vec2(1.0, 2.0));
    }

    #[test]
    fn test_vec2_normalized() {
        fn generate_spiral(n: usize, start: Vec2, end: Vec2) -> impl Iterator<Item = Vec2> {
            let angle_step = 2.0 * std::f32::consts::PI / n as f32;
            let radius_step = (end.length() - start.length()) / n as f32;

            (0..n).map(move |i| {
                let angle = i as f32 * angle_step;
                let radius = start.length() + i as f32 * radius_step;
                let x = radius * angle.cos();
                let y = radius * angle.sin();
                vec2(x, y)
            })
        }

        for v in generate_spiral(40, Vec2::splat(0.1), Vec2::splat(2.0)) {
            let vn = v.normalized();
            almost_eq!(vn.length(), 1.0);
            assert!(vn.is_normalized());
        }
    }
}
