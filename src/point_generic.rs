use super::*;
use num_traits::Float;
use super::point::Point;


/// Point with dimensions of constant generic size N and of generic type T
/// (Implemented as Newtype Pattern on an array 
/// see book or https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
/// This type only interacts with the library through 
/// the point trait, so you are free to use your own 
/// Point/Coord/Vec structures instead by implementing the (small) trait
#[derive(Debug, Copy, Clone)]
pub struct PointN<T, const N: usize>([T; N]);

impl<T, const N: usize> PointN<T, N> {
    pub fn new(array: [T;N]) -> Self {
        return PointN(array)
    }
}

/// Initialize with the Default value for the underlying type
impl<T: Default + Copy, const N: usize> Default for PointN<T, N> {
    fn default() -> Self {
        PointN([T::default(); N])
    }
}

impl<T, const N: usize> PartialEq for PointN<T, N> 
where T: PartialOrd {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..N {
            if self.0[i] != other.0[i] {
                return false
            }
        }
        return true
    }
}

impl<T, const N: usize> Add for PointN<T, N>
where
    T: Add<Output=T> + Clone + Copy,
{
    type Output = Self;

    fn add(self, other: PointN<T, N>) -> PointN<T, N> {
        let mut res = self.clone();
        for i in 0..N {
            res.0[i] = self.0[i] + other.0[i];
        }
        res
    }
}

impl<T, const N: usize> Sub for PointN<T, N>
where
    T: Sub<Output=T> + Clone + Copy,
{
    type Output = Self;

    fn sub(self, other: PointN<T, N>) -> PointN<T, N> {
        let mut res = self.clone();
        for i in 0..N {
            res.0[i] = self.0[i] - other.0[i];
        }
        res
    }
}



impl<T, const N:usize, U> Mul<U> for PointN<T, N>
where
    // How you have the mulitplication done is mulitpling T * U => T, this
    // trait bounds for T will specify this requirement as the mul operator is
    // translated to using the first operand as self and the second as rhs. 
    T: Mul<U,Output=T> + Clone + Copy,
    U: Clone + Copy,
{
    type Output = PointN<T, N>;

    fn mul(self, _rhs: U) -> PointN<T, N> {
        let mut res = self.clone();
        for i in 0..res.0.len() {
            res.0[i] = res.0[i] * _rhs;
        }
        res
    }
}


impl<T, const N: usize> IntoIterator for PointN<T, N> {
    type Item = T;
    type IntoIter = core::array::IntoIter<Self::Item, N>;

    fn into_iter(self) -> Self::IntoIter {
        core::array::IntoIter::new(self.0)
    }
}

// impl<'a, T, const N: usize> IntoIterator for &'a mut PointN<T, N> {
//     type Item = &'a mut T;
//     type IntoIter = slice::IterMut<'a, T>;

//     fn into_iter(self) -> slice::IterMut<'a, T> { 
//         core::array::IntoIter::new(self.0).next().map(|node| {
//             self.next = node.next.as_deref_mut();
//             &mut node.elem
//         }) 
//     }
// }


impl<T, const N: usize> Point for PointN<T, N>
where 
T: Float + Add + Copy + Default + Into<NativeFloat>,
NativeFloat: Add + Into<T>,
{
    type Scalar = NativeFloat;
    const DIM: usize = {N};

    fn axis(&self, index: usize) -> Self::Scalar {
        return self.0[index].into()
    }

    fn squared_length(&self) -> Self::Scalar {
        let mut sqr_dist: Self::Scalar = 0.0;
        for i in 0..N {
            sqr_dist = sqr_dist + (self.0[i]  * self.0[i]).into(); 
        }
        return sqr_dist
    }

}