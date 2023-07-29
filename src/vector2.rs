use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use encase::impl_vector;

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T, U> Add<Vector2<U>> for Vector2<T>
where
    T: Add<U>,
{
    type Output = Vector2<<T as Add<U>>::Output>;

    fn add(self, rhs: Vector2<U>) -> Self::Output {
        Vector2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T, U> AddAssign<Vector2<U>> for Vector2<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: Vector2<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T, U> Sub<Vector2<U>> for Vector2<T>
where
    T: Sub<U>,
{
    type Output = Vector2<<T as Sub<U>>::Output>;

    fn sub(self, rhs: Vector2<U>) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T, U> SubAssign<Vector2<U>> for Vector2<T>
where
    T: SubAssign<U>,
{
    fn sub_assign(&mut self, rhs: Vector2<U>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T, U> Mul<Vector2<U>> for Vector2<T>
where
    T: Mul<U>,
{
    type Output = Vector2<<T as Mul<U>>::Output>;

    fn mul(self, rhs: Vector2<U>) -> Self::Output {
        Vector2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T, U> MulAssign<Vector2<U>> for Vector2<T>
where
    T: MulAssign<U>,
{
    fn mul_assign(&mut self, rhs: Vector2<U>) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T, U> Div<Vector2<U>> for Vector2<T>
where
    T: Div<U>,
{
    type Output = Vector2<<T as Div<U>>::Output>;

    fn div(self, rhs: Vector2<U>) -> Self::Output {
        Vector2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T, U> DivAssign<Vector2<U>> for Vector2<T>
where
    T: DivAssign<U>,
{
    fn div_assign(&mut self, rhs: Vector2<U>) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<T> AsRef<[T; 2]> for Vector2<T> {
    fn as_ref(&self) -> &[T; 2] {
        assert!(std::mem::size_of::<Vector2<T>>() == std::mem::size_of::<T>() * 2);
        unsafe { &*(self as *const Vector2<T> as *const [T; 2]) }
    }
}

impl<T> AsMut<[T; 2]> for Vector2<T> {
    fn as_mut(&mut self) -> &mut [T; 2] {
        assert!(std::mem::size_of::<Vector2<T>>() == std::mem::size_of::<T>() * 2);
        unsafe { &mut *(self as *mut Vector2<T> as *mut [T; 2]) }
    }
}

impl<T> From<Vector2<T>> for [T; 2] {
    fn from(value: Vector2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> From<[T; 2]> for Vector2<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self { x, y }
    }
}

impl_vector!(2, Vector2<T>; using AsRef AsMut From);
