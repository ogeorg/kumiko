use geo_types::{point, polygon};
use geo_types::{Coord, Geometry, GeometryCollection, LineString, Point, Polygon};

#[derive(Debug, Clone)]
pub struct InfiniteLine {
    point: Point,
    uvec: Vec<f64>,
}

impl InfiniteLine {
    pub fn from_point_angle(pt: &Point, angle: f64) -> Self {
        Self {
            point: pt.clone(),
            uvec: vec![f64::cos(angle), f64::sin(angle)],
        }
    }
    /// + --->     
    ///         ->
    ///             + --->
    pub fn shift_by(&self, d: f64) -> InfiniteLine {
        let vx = self.uvec[1];
        let vy = -self.uvec[0];
        InfiniteLine {
            point: Point::new(self.point.x() + vx * d, self.point.y() + vy * d),
            uvec: self.uvec.clone(),
        }
    }

    pub fn intersection(&self, other: &Self) -> Point {
        let a1 = self.uvec[0];
        let b1 = -other.uvec[0];
        let c1 = other.point.x() - self.point.x();
        let a2 = self.uvec[1];
        let b2 = -other.uvec[1];
        let c2 = other.point.y() - self.point.y();
        let d = a1 * b2 - b1 * a2;
        let dk = c1 * b2 - c2 * b1;
        let k = dk / d;
        Point::new(
            self.point.x() + k * self.uvec[0],
            self.point.y() + k * self.uvec[1],
        )
    }

    pub fn from_to(pa: &Point, pb: &Point) -> Self {
        let dx = pb.x() - pa.x();
        let dy = pb.y() - pa.y();
        let norm = f64::sqrt(dx * dx + dy * dy);
        Self {
            point: pa.clone(),
            uvec: vec![dx / norm, dy / norm],
        }
    }

    pub fn at_time(&self, time: f64) -> Point {
        point! {x: self.point.x() + time * self.uvec[0], y: self.point.y() + time * self.uvec[1]}
    }

    pub fn at_times(&self, times: Vec<f64>) -> Vec<Point> {
        times.iter().map(|t| self.at_time(*t)).collect()
    }
}

pub struct LinesLR {
    pub l: InfiniteLine,
    pub r: InfiniteLine,
}

impl LinesLR {
    pub fn new(c: InfiniteLine, width: f64) -> Self {
        LinesLR {
            l: c.shift_by(-width / 2.),
            r: c.shift_by(width / 2.),
        }
    }
}

pub struct KumikoConfig {
    pub width_fine: f64,
    pub width_outer: f64,
}

impl Default for KumikoConfig {
    fn default() -> KumikoConfig {
        KumikoConfig {
            width_fine: 0.15,
            width_outer: 0.05,
        }
    }
}

pub fn mean_point(pa: &Point, pb: &Point) -> Point {
    point! { x: (pb.x()-pa.x())/2., y:(pb.y()-pa.y()) / 2.}
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

pub fn triangle(la: &InfiniteLine, lb: &InfiniteLine, lc: &InfiniteLine) -> Polygon {
    polygon![
        la.intersection(lb).into(),
        lb.intersection(lc).into(),
        lc.intersection(la).into()
    ]
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

pub fn points2geometry(pts: Vec<Point>) -> GeometryCollection {
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
        assert_eq!(beam.uvec[0], 1.0);
        assert_eq!(beam.uvec[1], 0.0);

        let beam2 = beam.shift_by(1.);
        println!("{:?}", beam2);
        assert_eq!(beam2.point, Point::new(0.0, -1.0));
    }
    #[test]
    fn intersection() {
        let pta = Point::new(0., 0.);
        let la = InfiniteLine::from_point_angle(&pta, PI / 4.);
        let ptb = Point::new(2., 0.);
        let lb = InfiniteLine::from_point_angle(&ptb, 3. * PI / 4.);
        let intersection = la.intersection(&lb);
        println!("{:?}", intersection);
    }
}
