use core::slice::*;

use super::*;
use super::point::Point;

/// General Implementation of a BSpline with choosable degree, control points and knots,
/// subject to restrictions by definition
#[derive(Clone, Debug)]
pub struct BSpline<'a, P, F> 
where 
P: Point + Copy,
F: Float + Into<NativeFloat>
{
    /// Degree of the polynomial pieces
    degree: usize,
    /// Control points (reference to any slice of points)
    control_points: &'a [P],
    /// The knot vector (reference to any slice of floats)
    knots: &'a [F],
}

impl<'a, P, F> BSpline<'a, P, F> 
where
P: Point + Copy,
F: Float + Into<NativeFloat> 
{
    /// Create a new B-spline curve that interpolates
    /// the `control_points` using a piecewise polynomial of `degree` within intervals specified by the `knots`. 
    /// The knots _must_ be sorted in non-decreasing order, the constructor enforces this which may yield undesired results. 
    /// The degree is defined as `curve_order - 1`.
    /// Desired curve must have a valid number of control points and knots in relation to its degree or the constructor will return None. 
    /// A B-Spline curve requires at least one more control point than the degree (`control_points.len() >
    /// degree`) and the number of knots should be equal to `control_points.len() + degree + 1`.
    pub fn new(degree: usize, control_points: &'a [P], knots: &'a [F]) -> Option< BSpline<'a, P, F> > {
        if control_points.len() <= degree {
            //panic!("Too few control points for curve");
            None
        }
        else if knots.len() != control_points.len() + degree + 1 {
            // panic!(format!("Invalid number of knots, got {}, expected {}", knots.len(),
            //     control_points.len() + degree + 1));
            None
        } else {
            // TODO force sorting of the knots required for binary search (knot span) -> mutable reference required
            // FIX maybe dont sort and just use linear search for knot span, as knot vectors wont be really large anyway
            //.sort_by(|a, b| a.partial_cmp(b).unwrap());
            Some(BSpline { degree, control_points, knots })
        }
        
    }

    /// Compute a point on the curve at `t`, the parameter **must** be in the inclusive range
    /// of values returned by `knot_domain`. If `t` is out of bounds this function will assert
    /// on debug builds and on release builds you'll likely get an out of bounds crash.
    // pub fn eval(&self, t: F) -> P {
    //     debug_assert!(t >= self.knot_domain().0 && t <= self.knot_domain().1);
    //     // Find the knot span that contains t i.e. the first index with a knot value greater than the t we're searching for. 
    //     // We need to find the knot span such that: knot[span] <= t < knot[span + 1]
    //     // Note: A custom function is used to exploit binary search (knots are sorted)
    //     let span = match self.upper_bounds(&self.knots[..], t) {
    //         Some(x) if x == 0 => self.degree,
    //         Some(x) if x >= self.knots.len() - self.degree - 1 =>
    //             self.knots.len() - self.degree - 1,
    //         Some(x) => x,
    //         None => self.knots.len() - self.degree - 1,
    //     };
    //     self.de_boor_iterative(t, span)
    // }


    /// Returns an iterator over the control points.
    pub fn control_points(&self) -> Iter<'_, P>  {
        self.control_points.iter()
    }

    /// Returns an iterator over the knots.
    pub fn knots(&self) -> Iter<'_, F> {
        self.knots.iter()
    }

    /// Get the min and max knot domain values for finding the `t` range to compute
    /// the curve over. The curve is only defined over the inclusive range `[min, max]`,
    /// passing a `t` value outside of this range will result in an assert on debug builds
    /// and likely a crash on release builds.
    pub fn knot_domain(&self) -> (F, F) {
        (self.knots[self.degree], self.knots[self.knots.len() - 1 - self.degree])
    }


    /// Iteratively compute de Boor's B-spline algorithm, this computes the recursive
    /// de Boor algorithm tree from the bottom up. At each level we use the results
    /// from the previous one to compute this level and store the results in the
    /// array indices we no longer need to compute the current level (the left one
    /// used computing node j).
    // fn de_boor_iterative(&self, t: F, i_start: usize) -> P {
    //     let mut tmp: ArrayVec<[P; self.degree + 1]> = ArrayVec::new();
    //     for j in 0..=self.degree {
    //         let p = j + i_start - self.degree - 1;
    //         tmp.push(self.control_points[p]);
    //     }
    //     for lvl in 0..self.degree {
    //         let k = lvl + 1;
    //         for j in 0..self.degree - lvl {
    //             let i = j + k + i_start - self.degree;
    //             let alpha = (t - self.knots[i - 1]) / (self.knots[i + self.degree - k] - self.knots[i - 1]);
    //             debug_assert!(!alpha.is_nan());
    //             tmp[j] = tmp[j].interpolate(&tmp[j + 1], alpha);
    //         }
    //     }
    //     tmp[0]
    // }

    /// Return the index of the first element greater than the value passed.
    /// Becaus the knot vector is sorted, this function uses binary search. 
    /// If no element greater than the value passed is found, the function returns None.
    fn upper_bounds(&self, data: &[F], value: F) -> Option<usize> {
        let mut first = 0usize;
        let mut step;
        let mut count = data.len() as isize;
        while count > 0 {
            step = count / 2;
            let it = first + step as usize;
            if !value.lt(&data[it]) {
                first = it + 1;
                count -= step + 1;
            } else {
                count = step;
            }
        }
        // If we didn't find an element greater than value
        if first == data.len() {
            None
        } else {
            Some(first)
        }
    }

}