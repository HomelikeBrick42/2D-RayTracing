use std::ops::{Add, Div, Mul, Sub};

use encase::impl_vector;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T, U> Add<Vector3<U>> for Vector3<T>
where
    T: Add<U>,
{
    type Output = Vector3<<T as Add<U>>::Output>;

    fn add(self, rhs: Vector3<U>) -> Self::Output {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T, U> Sub<Vector3<U>> for Vector3<T>
where
    T: Sub<U>,
{
    type Output = Vector3<<T as Sub<U>>::Output>;

    fn sub(self, rhs: Vector3<U>) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T, U> Mul<Vector3<U>> for Vector3<T>
where
    T: Mul<U>,
{
    type Output = Vector3<<T as Mul<U>>::Output>;

    fn mul(self, rhs: Vector3<U>) -> Self::Output {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T, U> Div<Vector3<U>> for Vector3<T>
where
    T: Div<U>,
{
    type Output = Vector3<<T as Div<U>>::Output>;

    fn div(self, rhs: Vector3<U>) -> Self::Output {
        Vector3 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl<T> AsRef<[T; 3]> for Vector3<T> {
    fn as_ref(&self) -> &[T; 3] {
        assert!(std::mem::size_of::<Vector3<T>>() == std::mem::size_of::<f32>() * 3);
        unsafe { &*(self as *const Vector3<T> as *const [T; 3]) }
    }
}

impl<T> AsMut<[T; 3]> for Vector3<T> {
    fn as_mut(&mut self) -> &mut [T; 3] {
        assert!(std::mem::size_of::<Vector3<T>>() == std::mem::size_of::<f32>() * 3);
        unsafe { &mut *(self as *mut Vector3<T> as *mut [T; 3]) }
    }
}

impl<T> From<Vector3<T>> for [T; 3] {
    fn from(value: Vector3<T>) -> Self {
        [value.x, value.y, value.z]
    }
}

impl<T> From<[T; 3]> for Vector3<T> {
    fn from([x, y, z]: [T; 3]) -> Self {
        Self { x, y, z }
    }
}

impl_vector!(3, Vector3<T>; using AsRef AsMut From);
