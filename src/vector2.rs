use std::ops::{Add, Div, Mul, Sub};

use encase::impl_vector;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Add for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl AsRef<[f32; 2]> for Vector2 {
    fn as_ref(&self) -> &[f32; 2] {
        const _: () = assert!(std::mem::size_of::<Vector2>() == std::mem::size_of::<f32>() * 2);
        unsafe { &*(self as *const Vector2 as *const [f32; 2]) }
    }
}

impl AsMut<[f32; 2]> for Vector2 {
    fn as_mut(&mut self) -> &mut [f32; 2] {
        const _: () = assert!(std::mem::size_of::<Vector2>() == std::mem::size_of::<f32>() * 2);
        unsafe { &mut *(self as *mut Vector2 as *mut [f32; 2]) }
    }
}

impl From<Vector2> for [f32; 2] {
    fn from(value: Vector2) -> Self {
        [value.x, value.y]
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from(value: [f32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl_vector!(2, Vector2, f32; using AsRef AsMut From);
