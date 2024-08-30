use itertools::Itertools;
use std::fmt;

use geo_types::{point, polygon};
use geo_types::{Coord, Geometry, GeometryCollection, LineString, Point, Polygon};

/// An infinite, oriented line, defined by a point and a unit vector
/// It actually defines the line by a parametric equation
#[derive(Clone)]
pub struct InfiniteLine {
    /// A point by which passes the line
    point: Point,
    /// A unit, director vector
    uvec: Point,
}

impl fmt::Debug for InfiniteLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[({:.3}, {:.3?}) + k ({:.3}, {:.3?})]",
            self.point.x(),
            self.point.y(),
            self.uvec.x(),
            self.uvec.y(),
        )
    }
}

impl InfiniteLine {
    pub fn from_point_angle(pt: &Point, angle: f64) -> Self {
        Self {
            point: pt.clone(),
            uvec: point! { x:f64::cos(angle), y: f64::sin(angle)},
        }
    }

    pub fn from_point_vec(pt: &Point, vec: &Point) -> Self {
        Self {
            point: pt.clone(),
            uvec: *vec / vec.norm(),
        }
    }

    /// Shifts the line by a given distance.  
    /// First calculate the vector v normal tu u, and "moves" the point along v.
    /// Positive is to the right
    /// ```
    /// ^               ^
    /// | uvec  =>  (-) | (+)
    /// |               +---> v
    /// ```
    pub fn shift_by(&self, d: f64) -> InfiniteLine {
        let vx = self.uvec.y();
        let vy = -self.uvec.x();
        InfiniteLine {
            point: Point::new(self.point.x() + vx * d, self.point.y() + vy * d),
            uvec: self.uvec.clone(),
        }
    }

    /// Calculates the intersection point of two infinite lines
    /// ```
    /// let pta = (0., 0.).into();
    /// let la = InfiniteLine::from_point_angle(&pta, PI / 4.);
    /// let ptb = (2., 0.).into();
    /// let lb = InfiniteLine::from_point_angle(&ptb, 3. * PI / 4.);
    /// let intersection = la.intersection(&lb);
    /// ```
    pub fn intersection(&self, other: &Self) -> Point {
        let a1 = self.uvec.x();
        let b1 = -other.uvec.x();
        let c1 = other.point.x() - self.point.x();
        let a2 = self.uvec.y();
        let b2 = -other.uvec.y();
        let c2 = other.point.y() - self.point.y();
        let d = a1 * b2 - b1 * a2;
        let dk = c1 * b2 - c2 * b1;
        let k = dk / d;

        self.point + scalar_times(k, &self.uvec)
    }

    /// Yields the lines that passed through both points pa and pb, having the sense from pa to pb
    pub fn from_to(p_from: &Point, p_to: &Point) -> Self {
        let d: Point = *p_to - *p_from;
        let norm = d.norm();
        Self {
            point: p_from.clone(),
            uvec: d / norm,
        }
    }

    /// Yields the point on the line for a given value of the parameter
    pub fn at_time(&self, time: f64) -> Point {
        self.point + scalar_times(time, &self.uvec)
    }

    /// Yields a vector of points on the line for some given values of the parameter
    pub fn at_times(&self, times: Vec<f64>) -> Vec<Point> {
        times.iter().map(|t| self.at_time(*t)).collect()
    }
}

pub fn angle_between(a: &Point, b: &Point) -> f64 {
    let na = a.norm();
    let nb = b.norm();
    let ab = a.dot(*b);
    (ab / (na * nb)).acos()
}

pub struct Rotation {
    cos: f64,
    sin: f64,
}

impl Rotation {
    pub fn by(angle: f64) -> Self {
        Self {
            cos: angle.cos(),
            sin: angle.sin(),
        }
    }
    pub fn rotate(&self, p: &Point) -> Point {
        point! {x: p.x() * self.cos - p.y() * self.sin, y: p.x() * self.sin + p.y() *self.cos }
    }
}
fn scalar_times(k: f64, pt: &Point) -> Point {
    Point::new(pt.x() * k, pt.y() * k)
}
trait Normalizable {
    fn norm(&self) -> f64;
}

impl Normalizable for Point {
    fn norm(&self) -> f64 {
        (self.x() * self.x() + self.y() * self.y()).sqrt()
    }
}

/// Defines a pair of lines supposed to be on each side of a given line
pub struct LinesLR {
    /// Line on the left
    pub l: InfiniteLine,
    /// Line on the right
    pub r: InfiniteLine,
}

impl LinesLR {
    /// Creates a new pair of lines separated by a given width
    pub fn new(c: &InfiniteLine, width: f64) -> Self {
        LinesLR {
            l: c.shift_by(-width / 2.),
            r: c.shift_by(width / 2.),
        }
    }
}

/*
#[macro_export]
macro_rules! mean_point {
    ($pa:ty, $pb:ty) => {
        mean_point_pair($pa, $pb);
    };
    ($ps:tt) => {
        mean_point_vec($ps)
    };
}
*/

pub fn mean_point_vec(pts: &[Point]) -> Point {
    let mut x = 0.;
    let mut y = 0.;
    for pt in pts {
        x += pt.x();
        y += pt.y();
    }
    let count: f64 = pts.len() as f64;
    Point::new(x / count, y / count)
}

/// Yields a Polygon created by 3 lines
pub fn triangle(la: &InfiniteLine, lb: &InfiniteLine, lc: &InfiniteLine) -> Polygon {
    polygon![
        la.intersection(lb).into(),
        lb.intersection(lc).into(),
        lc.intersection(la).into()
    ]
}

pub fn polygon(lines: &[InfiniteLine]) -> Polygon {
    let mut coords: Vec<(f64, f64)> = vec![];
    for (l1, l2) in lines.into_iter().circular_tuple_windows() {
        let p = l1.intersection(l2).into();
        coords.push(p);
    }
    Polygon::new(LineString::from(coords), vec![])
}

pub fn linestring_from_lines(lines: Vec<&InfiniteLine>) -> LineString {
    let count = lines.len();
    let pts: Vec<Coord> = (0..count)
        .map(|i| {
            let p1 = lines[i];
            let p2 = lines[(i + 1) % count];
            p1.intersection(p2).into()
        })
        .collect();
    LineString::new(pts)
}

pub fn points2geometry(pts: &[Point]) -> GeometryCollection {
    let gpts: Vec<Geometry> = pts.iter().map(|pt| Geometry::Point(pt.clone())).collect();
    GeometryCollection::new_from(gpts)
}

// test with
// cargo test -- --nocapture oglines

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;

    #[test]
    fn internal() {
        println!("Testing");
        let center = Point::new(0., 0.);
        let beam = InfiniteLine::from_point_angle(&center, 0.);
        println!("{:?}", beam);
        assert_eq!(beam.uvec.x(), 1.0);
        assert_eq!(beam.uvec.y(), 0.0);

        let beam2 = beam.shift_by(1.);
        println!("{:?}", beam2);
        assert_eq!(beam2.point, Point::new(0.0, -1.0));
    }
    #[test]
    fn intersection() {
        let pta = (0., 0.).into();
        let la = InfiniteLine::from_point_angle(&pta, PI / 4.);
        let ptb = (2., 0.).into();
        let lb = InfiniteLine::from_point_angle(&ptb, 3. * PI / 4.);
        let intersection = la.intersection(&lb);
        println!("{:?}", intersection);
    }
}
