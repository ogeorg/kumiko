use geo::{coord, line_string, Coord, LineString, Point, Polygon};

use crate::operations::intersect;

pub trait Frame {
    fn frame(&self, interiors: &[LineString]) -> Polygon {
        let contour_line = self.outer_contour();
        let clipping_line = self.inner_contour();

        intersect(interiors, contour_line, clipping_line)
    }
    /// Returns the four inner vertices of the frame
    fn inner_contour(&self) -> LineString;
    fn outer_contour(&self) -> LineString;
}

trait RectangularFrame {
    fn rectangular_contour_vertices(&self, margin: f64) -> Vec<Coord<f64>> {
        let xmin = self.origin().x() - margin;
        let xmax = self.origin().x() + self.width() + margin;
        let ymin = self.origin().y() - margin;
        let ymax = self.origin().y() + self.height() + margin;

        vec![
            coord! {x:xmin, y:ymin},
            coord! {x:xmax, y:ymin},
            coord! {x:xmax, y:ymax},
            coord! {x:xmin, y:ymax},
        ]
    }
    fn origin(&self) -> Point;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}
pub struct SimpleFrame {
    /// Origin of the inner frame (lower left corner)
    origin: Point,
    /// Width of the inner frame
    width: f64,
    /// Height of the inner frame
    height: f64,
    /// Margin around the inner frame
    margin: f64,
}
impl RectangularFrame for SimpleFrame {
    fn origin(&self) -> Point {
        self.origin
    }
    fn width(&self) -> f64 {
        self.width
    }
    fn height(&self) -> f64 {
        self.height
    }
}
impl SimpleFrame {
    pub fn new(origin: Point, width: f64, height: f64, margin: f64) -> SimpleFrame {
        SimpleFrame {
            origin,
            width,
            height,
            margin,
        }
    }
}

impl Frame for SimpleFrame {
    fn inner_contour(&self) -> LineString {
        LineString(self.rectangular_contour_vertices(0.0))
    }

    fn outer_contour(&self) -> LineString {
        LineString(self.rectangular_contour_vertices(self.margin))
    }
}

#[derive(Debug)]
pub struct SideParams {
    phi: f64,
    delta: f64,
    n: Option<u16>,
    inverse: bool,
}

pub struct FrameParams<'a> {
    bottom: &'a SideParams,
    right: &'a SideParams,
    top: &'a SideParams,
    left: &'a SideParams,
    depth: f64,
}

pub struct ZigZagFrame<'a> {
    /// Origin of the inner frame (lower left corner)
    origin: Point,
    /// Width of the inner frame
    width: f64,
    /// Height of the inner frame
    height: f64,
    /// Margin around the inner frame
    margin: f64,
    /// Parameters for zig-zagging the frame
    params: FrameParams<'a>,
}

#[derive(Debug)]
struct Iter {
    n: Option<u16>,
}

impl Iter {
    fn new(n: &Option<u16>) -> Iter {
        Iter { n: n.clone() }
    }
    fn next(&mut self) -> () {
        if let Some(nn) = &mut self.n {
            *nn -= 1;
        }
    }
    fn is_done(&self) -> bool {
        if let Some(n) = &self.n {
            *n == 0
        } else {
            false
        }
    }
}

impl<'a> ZigZagFrame<'a> {
    pub fn new(
        origin: Point,
        width: f64,
        height: f64,
        margin: f64,
        params: FrameParams<'a>,
    ) -> ZigZagFrame {
        ZigZagFrame {
            origin,
            width,
            height,
            margin,
            params,
        }
    }

    pub fn lower_string(&self) -> LineString {
        let sp = self.params.bottom;
        let m = self.margin;
        let mut c: Coord = self.origin.0 + coord! {x: -m, y: -m};
        let xmax: f64 = self.origin.x() + self.width + m;
        if sp.inverse {
            c.y += self.params.depth;
        }
        let sign = if sp.inverse { -1.0 } else { 1.0 };
        let is_past_end = |x: f64| -> bool { x >= xmax };
        let iter = Iter::new(&sp.n);
        self.hor_string(c, sp.phi, sp.delta, sign, xmax, iter, &is_past_end)
    }

    pub fn upper_string(&self) -> LineString {
        let sp = self.params.top;
        let m = self.margin;
        let mut c: Coord =
            self.origin.0 + coord! {x:self.width, y:self.height} + coord! {x: m, y: m};
        let xmin: f64 = self.origin.x() - m;
        if sp.inverse {
            c.y += self.params.depth;
        }
        let sign = if sp.inverse { -1.0 } else { 1.0 };
        let is_past_end = |x: f64| -> bool { x <= xmin };
        let iter = Iter::new(&sp.n);
        self.hor_string(c, -sp.phi, -sp.delta, -sign, xmin, iter, &is_past_end)
    }

    fn hor_string<T>(
        &self,
        c: Coord,   // c is the starting point
        phi: f64,   // phi is the first x shift (+ for lower, - for upper)
        delta: f64, // delta is the recurring x shift (+ for lower, - for upper)
        sign: f64,  // sign is for reversing y shift
        end: f64,   // end
        mut iter: Iter,
        is_past_end: T, // Tests whether we are past the end
    ) -> LineString
    where
        T: Fn(f64) -> bool,
    {
        let mut c = c;
        let mut sign = sign;
        let mut cs = vec![c.clone()];
        c.x += phi;
        cs.push(c.clone());
        while !is_past_end(c.x) && !iter.is_done() {
            iter.next();

            // Move y
            c.y += sign * self.params.depth;
            sign = -sign;
            cs.push(c.clone());
            // Move x
            c.x += delta;
            cs.push(c.clone());
        }
        let lastc = cs.last_mut().unwrap();
        lastc.x = end;
        LineString::new(cs)
    }

    fn rectangular_contour_vertices(&self, margin: f64) -> Vec<Coord<f64>> {
        vec![]
    }
}

impl<'a> RectangularFrame for ZigZagFrame<'a> {
    fn origin(&self) -> Point {
        self.origin
    }
    fn width(&self) -> f64 {
        self.width
    }
    fn height(&self) -> f64 {
        self.height
    }
}

impl<'a> Frame for ZigZagFrame<'a> {
    fn inner_contour(&self) -> LineString {
        LineString(self.rectangular_contour_vertices(0.0))
    }

    fn outer_contour(&self) -> LineString {
        LineString(self.rectangular_contour_vertices(self.margin))
    }
}

#[cfg(test)]
mod tests {
    use geo::{coord, point, Coord};

    use super::{FrameParams, SideParams, ZigZagFrame};

    #[test]
    fn test_zigzag_bottom() {
        // given the config
        let bot_config: SideParams = SideParams {
            phi: 2.,
            delta: 3.,
            n: None,
            inverse: true,
        };
        let top_config: SideParams = SideParams {
            phi: 4.,
            delta: 3.,
            n: Some(4),
            inverse: false,
        };
        let ver_config: SideParams = SideParams {
            phi: 0.,
            delta: 4.,
            n: Some(4),
            inverse: false,
        };
        let params: FrameParams = FrameParams {
            bottom: &bot_config,
            right: &ver_config,
            top: &top_config,
            left: &ver_config,
            depth: 1.,
        };
        let width = 12.;
        let height = 30.;
        let frame: ZigZagFrame = ZigZagFrame::new(point! {x:0., y:0.}, width, height, 3., params);

        // When

        //  3  2  1  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
        //  |        |                                   |        |
        //  |        +-----------------------------------+        |
        //  |                                                     |
        //  |                                                     |
        //  +-----------------------------------------------------+
        //
        //  3  2  1  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
        //
        //            <---<---<---<---<---<---<---<---<---
        //
        //  +--=--=--=--=--+        +--=--=--+        +--=--=--=--+ 33
        //  |              +--=--=--+        +--=--=--+           | 32
        //  |                                                     |
        //  |        +-----------------------------------+        | 30 = origin.y+height
        //  |        |                                   |        |
        //  |        +-----------------------------------+        |  0 = origin.y
        //  |                                                     |
        //  +-----+        +--=--=--+        +--=--=--+        +--+ -2
        //        +--=--=--+        +--=--=--+        +--=--=--+    -3
        //
        //             --->--->--->--->--->--->--->--->--->
        let low_str = frame.lower_string();
        let low_cs: Vec<Coord> = low_str.coords().map(|c| *c).collect::<Vec<Coord>>();
        let top_str = frame.upper_string();
        let top_cs: Vec<Coord> = top_str.coords().map(|c| *c).collect::<Vec<Coord>>();
        // Then
        let expected_bot_coords = vec![
            coord! {x: -3., y: -2. },
            coord! {x: -1., y: -2. },
            coord! {x: -1., y: -3. },
            coord! {x: 2., y: -3. },
            coord! {x: 2., y: -2. },
            coord! {x: 5., y: -2. },
            coord! {x: 5., y: -3. },
            coord! {x: 8., y: -3. },
            coord! {x: 8., y: -2. },
            coord! {x: 11., y: -2. },
            coord! {x: 11., y: -3. },
            coord! {x: 14., y: -3. },
            coord! {x: 14., y: -2. },
            coord! {x: 15., y: -2. },
        ];
        assert_eq!(expected_bot_coords, low_cs);

        let expected_top_coords = vec![
            coord! {x: 15., y: height+3. },
            coord! {x: 11., y: height+3. },
            coord! {x: 11., y: height+2. },
            coord! {x: 8., y: height+2. },
            coord! {x: 8., y: height+3. },
            coord! {x: 5., y: height+3. },
            coord! {x: 5., y: height+2. },
            coord! {x: 2., y: height+2. },
            coord! {x: 2., y: height+3. },
            coord! {x: -3., y: height+3. },
        ];
        assert_eq!(expected_top_coords, top_cs);
    }
}
