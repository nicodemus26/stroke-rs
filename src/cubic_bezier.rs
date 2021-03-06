use core::default::Default;

use super::*;
use super::point::Point;
use super::line::LineSegment; 
use super::quadratic_bezier::QuadraticBezier;

/// A 2d  cubic Bezier curve defined by four points: the starting point, two successive
/// control points and the ending point.
/// The curve is defined by equation:
/// ```∀ t ∈ [0..1],  P(t) = (1 - t)³ * start + 3 * (1 - t)² * t * ctrl1 + 3 * t² * (1 - t) * ctrl2 + t³ * end```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CubicBezier<P>
{
    pub (crate) start:  P,
    pub (crate) ctrl1:  P,
    pub (crate) ctrl2:  P,
    pub (crate) end:    P,
}

//#[allow(dead_code)]
impl<P> CubicBezier<P> 
where 
P: Point<Scalar = NativeFloat>
    + Copy
    + Add<P, Output = P>
    + Sub<P, Output = P>
    + Mul<NativeFloat, Output = P>,
{

    pub fn new(start: P, ctrl1: P, ctrl2: P,  end: P) -> Self 
    {
        CubicBezier { 
            start, 
            ctrl1, 
            ctrl2, 
            end 
        }
    }

    /// Evaluate a CubicBezier curve at t by direct evaluation of the polynomial (not numerically stable)
    pub fn eval<F>(&self, t: F) -> P 
    where 
    F: Float,
    P: Add<P, Output = P>
        + Sub<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
                + Mul<F, Output = F>
                + Into<F>
    {
        return self.start * ((1.0-t) * (1.0-t) * (1.0-t))
                + self.ctrl1 * (3.0 * t * (1.0-t) * (1.0-t))
                + self.ctrl2 * (3.0 * t * t * (1.0-t))
                + self.end * (t * t * t);
    }

    /// Evaluate a CubicBezier curve at t using the numerically stable De Casteljau algorithm
    pub fn eval_casteljau<F>(&self, t: F) -> P 
    where 
    F: Float,
    P: Add<P, Output = P>
        + Sub<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Mul<F, Output = F>
    {
        // unrolled de casteljau algorithm
        // _1ab is the first iteration from first (a) to second (b) control point and so on
        let ctrl_1ab = self.start + (self.ctrl1 - self.start) * t;
        let ctrl_1bc = self.ctrl1 + (self.ctrl2 - self.ctrl1) * t;
        let ctrl_1cd = self.ctrl2 + (self.end - self.ctrl2)   * t;
        // second iteration
        let ctrl_2ab  = ctrl_1ab + (ctrl_1bc - ctrl_1ab) * t;
        let ctrl_2bc  = ctrl_1bc + (ctrl_1cd - ctrl_1bc) * t;
        // third iteration, final point on the curve
        let ctrl_3ab = ctrl_2ab + (ctrl_2bc - ctrl_2ab) * t;

        return ctrl_3ab
    }

    /// Returns the x coordinate of the curve evaluated at t
    /// Convenience shortcut for bezier.eval(t).x()
    pub fn axis<F>(&self, t: F, axis: usize) -> F
    where
    F : Float,
    P: Mul<NativeFloat, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Mul<F, Output = F> 
        + Into<F>
    {
        let t2 = t * t;
        let t3  = t2 * t;
        let one_t  = (1.0 as NativeFloat) - t;
        let one_t2 = one_t * one_t;
        let one_t3 = one_t2 * one_t;

        self.start.axis(axis).into() * one_t3
            + self.ctrl1.axis(axis).into() * 3.0.into() * one_t2 * t
            + self.ctrl2.axis(axis).into() * 3.0.into() * one_t * t2
            + self.end.axis(axis) * t3
    }

    /// Approximates the arc length of the curve by flattening it with straight line segments.
    /// Remember arclen also works by linear approximation, not the integral, so we have to accept error!
    /// This approximation is unfeasable if desired accuracy is greater than 2 decimal places
    pub fn arclen<F>(&self, nsteps: usize) -> F
    where
    F: Float,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let stepsize: NativeFloat = 1.0/(nsteps as NativeFloat);
        let mut arclen: NativeFloat = 0.0;
        for t in 1..nsteps {
            let t = t as NativeFloat * 1.0/(nsteps as NativeFloat);
            let p1 = self.eval_casteljau(t.into());
            let p2 = self.eval_casteljau(t.into()+stepsize.into());

            arclen = arclen + (p1-p2).squared_length().sqrt();
        
        }
        return arclen.into()
    }


    pub fn split<F>(&self, t: F) -> (Self, Self)
    where
    F: Float,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Mul<F, Output = F>,
    {
       // unrolled de casteljau algorithm
        // _1ab is the first iteration from first (a) to second (b) control point and so on
        let ctrl_1ab = self.start + (self.ctrl1 - self.start) * t;
        let ctrl_1bc   = self.ctrl1 + (self.ctrl2 - self.ctrl1) * t;
        let ctrl_1cd   = self.ctrl2 + (self.end - self.ctrl2) * t;
        // second iteration
        let ctrl_2ab  = ctrl_1ab + (ctrl_1bc - ctrl_1ab) * t;
        let ctrl_2bc  = ctrl_1bc + (ctrl_1cd - ctrl_1bc) * t;
        // third iteration, final point on the curve
        let ctrl_3ab = ctrl_2ab + (ctrl_2bc - ctrl_2ab) * t;

        return (
            CubicBezier {
                start: self.start,
                ctrl1: ctrl_1ab,
                ctrl2: ctrl_2ab,
                end: ctrl_3ab,
            },
            CubicBezier {
                start: ctrl_3ab,
                ctrl1: ctrl_2bc,
                ctrl2: ctrl_1cd,
                end: self.end,
            },
        );
    }

    /// Return the derivative function.
    /// The derivative is also a bezier curve but of degree n-1 (cubic->quadratic)
    /// Since it returns the derivative function, eval() needs to be called separately
    pub fn derivative<F>(&self) -> QuadraticBezier<P>
    where
    F: Float,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Into<F>
    {
        return QuadraticBezier{
            start: (self.ctrl1 - self.start) * 3.0.into(),
            ctrl:  (self.ctrl2 - self.ctrl1) * 3.0.into(),
            end:   (self.end - self.ctrl2)   * 3.0.into()
        }
    }



    /// Direct Derivative - Sample the axis coordinate at 'axis' of the curve's derivative at t.
    /// Parameters: 
    /// t: the sampling parameter on the curve interval [0..1]
    /// axis: the index of the coordinate axis [0..N]
    /// Returns:
    /// Scalar value of the points own type type F
    /// This is a convenience function for .derivative().eval(t).axis(n)
    pub fn dd<F>(&self, t: F, axis: usize) -> F 
    where
    F: Float,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let t2 = t*t;
        let c0 = -3.0.into() * t + 6.0.into() * t - 3.0.into();
        let c1 = 9.0.into() * t2 - 12.0.into() * t + 3.0.into();
        let c2 = -9.0.into() * t2 + 6.0.into() * t;
        let c3 = 3.0.into() * t2;
        return self.start.axis(axis).into() * c0 
                + self.ctrl1.axis(axis).into() * c1 
                + self.ctrl2.axis(axis).into() * c2 
                + self.end.axis(axis).into() * c3
    }




    // pub fn curvature<F>(&self, t: F) -> F
    // where
    // F: Float,
    // P:  Sub<P, Output = P>
    //     + Add<P, Output = P>
    //     + Mul<F, Output = P>,
    // NativeFloat: Sub<F, Output = F> 
    //     + Add<F, Output = F>
    //     + Mul<F, Output = F>
    //     + Float
    //     + Into<F>
    // {
    //     let d = self.derivative();
    //     let dd = d.derivative();
    //     let dx = d.x(t);
    //     let dy = d.y(t);
    //     let ddx = dd.x(t);
    //     let ddy = dd.y(t);
    //     let numerator = dx * ddy.into() - ddx * dy;
    //     let denominator = (dx*dx + dy*dy).powf(1.5.into());
    //     return numerator / denominator
    // }

    // pub fn radius<F>(&self, t: F) -> F
    // where
    // F: Float,
    // P:  Sub<P, Output = P>
    //     + Add<P, Output = P>
    //     + Mul<F, Output = P>,
    // NativeFloat: Sub<F, Output = F> 
    //     + Add<F, Output = F>
    //     + Mul<F, Output = F>
    //     + Float
    //     + Into<F>
    // {
    //     return 1.0.into() / self.curvature(t)
    // }


    pub fn baseline(&self) -> LineSegment<P> {
        LineSegment {
            start: self.start,
            end: self.end,
        }
    }

    pub fn is_linear<F>(&self, tolerance: F) -> bool 
    where
    F: Float 
        + Default
        + Into<NativeFloat>,
    P: Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F> 
    {
        // if start and end are (nearly) the same
        if (self.start-self.end).squared_length() < EPSILON {
            return false;
        } 
        // else check if ctrl points lie on baseline
        self.are_points_colinear(tolerance)
    }


    fn are_points_colinear<F>(&self, tolerance: F) -> bool
    where
    F: Float
        + Default
        + Into<NativeFloat>,
    P: Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let lineeq = self.baseline();//.to_line().equation();
        lineeq.distance_to_point(self.ctrl1) <= tolerance
            && lineeq.distance_to_point(self.ctrl2) <= tolerance
    }

    // Returs if the whole set of control points can be considered one singular point 
    // given some tolerance. 
    // TODO use machine epsilon vs squared_length OK?
    pub(crate) fn is_a_point<F>(&self, tolerance: F) -> bool 
    where
    F: Float,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let tolerance_squared = tolerance * tolerance;
        // Use <= so that tolerance can be zero.
        (self.start-self.end).squared_length().into() <= tolerance_squared
            && (self.start-self.ctrl1).squared_length().into() <= tolerance_squared
            && (self.end-self.ctrl2).squared_length().into() <= tolerance_squared
    }

    /// Compute the real roots of the cubic bezier function with
    /// parameters of the form a*t^3 + b*t^2 + c*t + d for each dimension
    /// using cardano's algorithm (code adapted from github.com/nical/lyon)
    /// returns an ArrayVec of the present roots (max 3)
    fn real_roots<F>(&self, a: F, b: F, c: F, d: F) -> ArrayVec<[F; 3]>
    where
    F:  Float
        + Default,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let mut result = ArrayVec::new();

        let pi = 3.141592.into();

        // check if can be handled below cubic order
        if a.abs() < EPSILON.into() {
            if b.abs() < EPSILON.into() {
                if c.abs() < EPSILON.into() {
                    // no solutions
                    return result;
                }
                // is linear equation
                result.push(-d / c);
                return result;
            }
            // is quadratic equation
            let delta = c * c - 4.0.into() * b * d;
            if delta > 0.0.into() {
                let sqrt_delta = delta.sqrt();
                result.push((-c - sqrt_delta) / (2.0.into() * b));
                result.push((-c + sqrt_delta) / (2.0.into() * b));
            } else if delta.abs() < EPSILON.into() {
                result.push(-c / (2.0.into() * b));
            }
            return result;
        }

        // is cubic equation -> use cardano's algorithm
        let frac_1_3 = 1.0.into() / 3.0.into();

        let bn = b / a;
        let cn = c / a;
        let dn = d / a;
    
        let delta0 = (3.0.into() * cn - bn * bn) / 9.0.into();
        let delta1 = (9.0.into() * bn * cn - 27.0.into() * dn - 2.0.into() * bn * bn * bn) / 54.0.into();
        let delta_01 = delta0 * delta0 * delta0 + delta1 * delta1;
    
        if delta_01 >= 0.0.into() {
            let delta_p_sqrt = delta1 + delta_01.sqrt();
            let delta_m_sqrt = delta1 - delta_01.sqrt();
    
            let s = delta_p_sqrt.signum() * delta_p_sqrt.abs().powf(frac_1_3);
            let t = delta_m_sqrt.signum() * delta_m_sqrt.abs().powf(frac_1_3);
    
            result.push(-bn * frac_1_3 + (s + t));
    
            // Don't add the repeated root when s + t == 0.
            if (s - t).abs() < EPSILON.into() && (s + t).abs() >= EPSILON.into() {
                result.push(-bn * frac_1_3 - (s + t) / 2.0.into());
            }
        } else {
            let theta = (delta1 / (-delta0 * delta0 * delta0).sqrt()).acos();
            let two_sqrt_delta0 = 2.0.into() * (-delta0).sqrt();
            result.push(two_sqrt_delta0 * Float::cos(theta * frac_1_3) - bn * frac_1_3);
            result.push(
                two_sqrt_delta0 * Float::cos((theta + 2.0.into() * pi) * frac_1_3) - bn * frac_1_3,
            );
            result.push(
                two_sqrt_delta0 * Float::cos((theta + 4.0.into() * pi) * frac_1_3) - bn * frac_1_3,
            );
        }
    
        result
    }

    /// Solves the cubic bezier function given the control points' x OR y values
    /// by solving the roots for x or y axis functions
    /// Returns those roots of the function that are in the interval [0.0, 1.0].
    fn solve_t_for_axis<F>(&self, value: F, axis: usize) -> ArrayVec<[F; 3]> 
    where
    F:  Float
        + Default
        + Into<NativeFloat>,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        let mut result = ArrayVec::new();
        // check if all points are the same or if the curve is really a line
        if self.is_a_point(0.0.into())
            || (self.are_points_colinear(0.0.into()) )// && self.start.y() == self.end.y())
        {
            return result
        }
        // these are just the x or y components of the points
        let a = -self.start.axis(axis) + 
                    3.0 * self.ctrl1.axis(axis).into()
                    - 3.0 * self.ctrl2.axis(axis).into()
                    + self.end.axis(axis).into();
        let b = 3.0.into() * self.start.axis(axis).into()
                    - 6.0.into() * self.ctrl1.axis(axis).into()
                    + 3.0.into() * self.ctrl2.axis(axis).into();
        let c = -3.0.into() * self.start.axis(axis).into() 
                    + 3.0.into() * self.ctrl1.axis(axis).into();
        let d = self.start.axis(axis) - value;

        let roots = self.real_roots(a, b, c, d);
        for root in roots {
            if root > 0.0.into() && root < 1.0.into() {
                result.push(root);
            }
        }

        result
    }

    /// Return the bounding box of the curve as an array of (min, max) tuples for each dimension (its index)
    pub fn bounding_box<F>(&self) -> [(F, F); P::DIM] 
    where
    F: Float
        + Default,
    P:  Sub<P, Output = P>
        + Add<P, Output = P>
        + Mul<F, Output = P>,
    NativeFloat: Sub<F, Output = F> 
        + Add<F, Output = F>
        + Mul<F, Output = F>
        + Float
        + Into<F>
    {
        // calculate coefficients for the derivative: at^2 + bt + c
        // from the expansion of the cubic bezier curve: sum_i=0_to_3( binomial(3, i) * t^i * (1-t)^(n-i) )
        // yields coeffcients
        // po: [1, -2,  1]
        // p1: [0,  2, -2]
        // p2: [0,  0,  1]
        //      c   b   a
        let mut bounds = [(0.0.into(), 0.0.into()); P::DIM];
        let derivative = self.derivative();
        // calculate coefficients for derivative
        let a: P = derivative.start + derivative.ctrl * -2.0.into() + derivative.end;
        let b: P = derivative.start * -2.0.into() + derivative.ctrl * 2.0.into();
        let c: P = derivative.start;

        // calculate roots for t over x axis and plug them into the bezier function
        //  to get x,y values (make vec 2 bigger for t=0,t=1 values)
        // loop over any of the points dimensions (they're all the same)
        for (dim, _) in a.into_iter().enumerate() {
            let mut extrema: ArrayVec<[F; 4]> = ArrayVec::new();
            extrema.extend(derivative.real_roots(
                                                a.axis(dim).into(), 
                                                b.axis(dim).into(), 
                                                c.axis(dim).into(), 
                                                ).into_iter());
            // only retain roots for which t is in [0..1] 
            extrema.retain(|root| -> bool {root > &mut 0.0.into() && root < &mut 1.0.into()});
            // evaluates roots in original function
            for t in extrema.iter_mut() {
                *t = self.eval_casteljau(*t).axis(dim).into();
            }
            // add y-values for start and end point as candidates
            extrema.push(self.start.axis(dim).into()); 
            extrema.push(self.end.axis(dim).into());
            // sort to get min and max values for bounding box
            extrema.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());


            // determine xmin, xmax, ymin, ymax, from the set {B(xroots), B(yroots), B(0), B(1)} 
            // (Intermediate control points can't form a boundary)
            // .unwrap() is ok as it can never be empty as it always at least contains the endpoints
            bounds[dim] = (extrema[0], *extrema.last().unwrap());
        }
        return bounds
    }

}


#[cfg(test)]
mod tests 
{
    use super::*;
    use super::point_generic::PointN;
    #[test]
    fn circle_approximation_error() 
    {
        // define closure for unit circle 
        let circle = |p: PointN<f64, 2>| -> f64 { 
            p.into_iter().map(|x| x*x).sum::<f64>().sqrt() - 1f64
        };

        // define control points for 4 bezier segments 
        // control points are chosen for minimum radial distance error
        // according to: http://spencermortensen.com/articles/bezier-circle/ 
        // TODO don't hardcode values
        let c               = 0.551915024494;
        let max_drift_perc  = 0.019608; // radial drift percent
        let max_error       = max_drift_perc * 0.01; // absolute max radial error

        let bezier_quadrant_1= CubicBezier{ 
                                    start:  PointN::new([0f64, 1f64]),
                                    ctrl1:  PointN::new([c, 1f64]),
                                    ctrl2:  PointN::new([1f64, c]),
                                    end:    PointN::new([1f64, 0f64])
        };
        let bezier_quadrant_2 = CubicBezier{ 
                                    start:  PointN::new([1f64, 0f64]),
                                    ctrl1:  PointN::new([1f64, -c]),
                                    ctrl2:  PointN::new([c,  -1f64]),
                                    end:    PointN::new([0f64, -1f64])
        };
        let bezier_quadrant_3 = CubicBezier{ 
                                    start:  PointN::new([0f64, -1f64]),
                                    ctrl1:  PointN::new([-c, -1f64]),
                                    ctrl2:  PointN::new([-1f64, -c]),
                                    end:    PointN::new([-1f64, 0f64])
        };
        let bezier_quadrant_4 = CubicBezier{ 
                                    start:  PointN::new([-1f64, 0f64]),
                                    ctrl1:  PointN::new([-1f64, c]),
                                    ctrl2:  PointN::new([-c, 1f64]),
                                    end:    PointN::new([0f64, 1f64])
        };
        let nsteps =  1000;                                      
        for t in 0..=nsteps {
            let t = t as f64 * 1f64/(nsteps as f64);

            let point = bezier_quadrant_1.eval(t);
            let contour = circle(point);
            assert!( contour.abs() <= max_error );

            let point = bezier_quadrant_2.eval(t);
            let contour = circle(point);
            assert!( contour.abs() <= max_error );

            let point = bezier_quadrant_3.eval(t);
            let contour = circle(point);
            assert!( contour.abs() <= max_error );

            let point = bezier_quadrant_4.eval(t);
            let contour = circle(point);
            assert!( contour.abs() <= max_error );
        }
    }


    #[test]
    fn circle_circumference_approximation() 
    {
        // define control points for 4 cubic bezier segments to best approximate a unit circle
        // control points are chosen for minimum radial distance error, see circle_approximation_error() in this file
        // given this, the circumference will also be close to 2*pi 
        // (remember arclen also works by linear approximation, not the true integral, so we have to accept error)!
        // This approximation is unfeasable if desired accuracy is greater than 2 decimal places (at 1000 steps)
        // TODO don't hardcode values, solve for them
        let c         = 0.551915024494;
        let max_error = 1e-2;
        let nsteps  = 1e3 as usize;
        let pi        = 3.14159265359;
        let tau       = 2.*pi;

        let bezier_quadrant_1= CubicBezier{ 
                                start:  PointN::new([0f64, 1f64]),
                                ctrl1:  PointN::new([c, 1f64]),
                                ctrl2:  PointN::new([1f64, c]),
                                end:    PointN::new([1f64, 0f64])
        };
        let bezier_quadrant_2 = CubicBezier{ 
                                start:  PointN::new([1f64, 0f64]),
                                ctrl1:  PointN::new([1f64, -c]),
                                ctrl2:  PointN::new([c, -1f64]),
                                end:    PointN::new([0f64, -1f64])
        };
        let bezier_quadrant_3 = CubicBezier{ 
                                start:  PointN::new([0f64, -1f64]),
                                ctrl1:  PointN::new([-c,   -1f64]),
                                ctrl2:  PointN::new([-1f64, -c]),
                                end:    PointN::new([-1f64, 0f64])
        };
        let bezier_quadrant_4 = CubicBezier{ 
                                start:  PointN::new([-1f64, 0f64]),
                                ctrl1:  PointN::new([-1f64, c]),
                                ctrl2:  PointN::new([-c, 1f64]),
                                end:    PointN::new([0f64, 1f64])
        };
        let circumference = bezier_quadrant_1.arclen::<NativeFloat>(nsteps) +
                                bezier_quadrant_2.arclen::<NativeFloat>(nsteps) +
                                bezier_quadrant_3.arclen::<NativeFloat>(nsteps) +
                                bezier_quadrant_4.arclen::<NativeFloat>(nsteps);
        //dbg!(circumference);
        //dbg!(tau);
        assert!( ((tau + max_error) > circumference) && ((tau - max_error) < circumference) );
    }

    #[test]
    fn eval_equivalence() {
        // all eval methods should be approximately equivalent for well defined test cases
        // and not equivalent where numerical stability becomes an issue for normal eval
        let bezier = CubicBezier::new( 
            PointN::new([0f64,  1.77f64]),
            PointN::new([1.1f64, -1f64]),
            PointN::new([4.3f64,3f64]),
            PointN::new([3.2f64, -4f64]),
        );

        let max_err = 1e-14;
        let nsteps: usize =  1000;                                      
        for t in 0..=nsteps {
            let t = t as f64 * 1f64/(nsteps as f64);
            let p1 = bezier.eval(t);
            let p2 = bezier.eval_casteljau(t);
            let err = p2-p1;
            //dbg!(p1);
            //dbg!(p2);
            for axis in err {
                assert!(axis.abs() < max_err);
            }
            
        }
    }

    #[test]
    fn split_equivalence() {
        // chose some arbitrary control points and construct a cubic bezier
        let bezier = CubicBezier{ 
                                start:  PointN::new([0f64, 1.77f64]),
                                ctrl1:  PointN::new([2.9f64, 0f64]),
                                ctrl2:  PointN::new([4.3f64, 3f64]),
                                end:    PointN::new([3.2f64, -4f64])
                            };
        // split it at an arbitrary point
        let at = 0.5;
        let (left, right) = bezier.split(at);
        // compare left and right subcurves with parent curve
        // this is tricky as we have to map t->t/2 (for left) which will 
        // inevitably contain rounding errors from floating point ops.
        // instead, take the difference of the two points which must not exceed the absolute error
        // TODO update test to use Point::distance() instead for error
        let max_err = 1e-14;
        let nsteps: usize =  1000;                                      
        for t in 0..=nsteps {
            let t = t as f64 * 1f64/(nsteps as f64);
            //dbg!(bezier.eval(t/2.0));
            //dbg!(left.eval(t));
            // left
            let mut err = bezier.eval(t/2.0) - left.eval(t);
            //dbg!(err);
            for axis in err {
                assert!(axis.abs() < max_err);
            }
            // right
            err = bezier.eval((t*0.5)+0.5) - right.eval(t);
            //dbg!(err);
            for axis in err {
                assert!(axis.abs() < max_err);
            }  
        }
    }


    #[test]
    fn bounding_box_contains() {
        // check if bounding box for a curve contains all points (with some approximation error)
        let bezier = CubicBezier{ 
                        start:  PointN::new([0f64, 1.77f64]),
                        ctrl1: PointN::new([2.9f64, 0f64]),
                        ctrl2: PointN::new([4.3f64, -3f64]),
                        end:   PointN::new([3.2f64, 4f64])
        };

        let bounds = bezier.bounding_box::<f64>();

        let max_err = 1e-2;

        let nsteps: usize =  100;                                      
        for t in 0..=nsteps {
            let t = t as f64 * 1f64/(nsteps as f64);
            let p = bezier.eval_casteljau(t);
            //dbg!(t);
            //dbg!(p);
            //dbg!(xmin-max_err, ymin-max_err, xmax+max_err, ymax+max_err);
            for (idx, axis) in p.into_iter().enumerate() {
                assert!( (axis >= (bounds[idx].0 - max_err)) && (axis <= (bounds[idx].1 + max_err)) )
            }
        }
    }
}