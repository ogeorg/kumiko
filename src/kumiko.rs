use std::f64::consts::PI;

use geo::{coord, line_string, MultiLineString};
use geo_svg::{Color, Svg, ToSvg};
use geo_types::{point, Point, Polygon};

impl Default for KumikoConfig {
    fn default() -> KumikoConfig {
        KumikoConfig {
            width_fine: 0.15,
            width_outer: 0.2,
        }
    }
}

pub struct Base {
    pub origin: Point,
    pub u: Point,
    pub v: Point,
    arrows: MultiLineString,
}

impl Base {
    pub fn new(origin: Point, pu: Point, pv: Point) -> Base {
        let c15: f64 = f64::cos(PI / 10.0);
        let s15: f64 = f64::sin(PI / 10.0);

        let u = pu.0;
        let v = pv.0;

        let c0 = origin.0;

        let cu = c0 + u;
        let cul = cu - coord! {x: c15*u.x-s15*u.y, y: s15*u.x+c15*u.y} * 0.2;
        let cur = cu - coord! {x: c15*u.x+s15*u.y, y: -s15*u.x+c15*u.y} * 0.2;

        let cv = c0 + v;
        let cvl = cv - coord! {x: c15*v.x-s15*v.y, y: s15*v.x+c15*v.y} * 0.2;
        let cvr = cv - coord! {x: c15*v.x+s15*v.y, y: -s15*v.x+c15*v.y} * 0.2;

        let mls = MultiLineString::new(vec![
            line_string![c0, cu],
            line_string![cu, cul],
            line_string![cu, cur],
            line_string![c0, cv],
            line_string![cv, cvl],
            line_string![cv, cvr],
        ]);
        Base {
            origin,
            u: pu,
            v: pv,
            arrows: mls,
        }
    }
    pub fn draw(&self) -> Svg {
        self.arrows
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 100, 0))
            .with_fill_opacity(0.2)
    }
}
pub trait KumikoConfigTrait {
    fn width_outer(&self) -> f64;
    fn width_fine(&self) -> f64;
}

pub struct KumikoConfig {
    pub width_fine: f64,
    pub width_outer: f64,
    //    pub params: HashMap<&'static str, f64>,
}

impl KumikoConfigTrait for KumikoConfig {
    fn width_outer(&self) -> f64 {
        self.width_outer
    }

    fn width_fine(&self) -> f64 {
        self.width_fine
    }
}

pub trait KumikoFigure<KC: KumikoConfigTrait> {
    //    fn new_at_base(base: &Base, config: &KC) -> Self;
    fn polygon(&self) -> &Polygon;
    fn points(&self) -> &[Point];

    /*
    fn draw_points(&self) -> Svg {
        let points = self.points();
        let svg = points.to_svg();
        //            .with_radius(0.02)
        //            .with_stroke_width(0.01)
        //            .with_stroke_color(Color::Rgb(100, 0, 200))
        //            .with_fill_opacity(0.2);
        svg
    }
    */

    fn draw_point(&self) -> Svg {
        let points: &Point = self.points().get(0).unwrap();
        let svg = points
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200))
            .with_fill_opacity(0.2);
        svg
    }
    fn draw_figure(&self) -> Svg {
        let poly = self.polygon();
        let svg: Svg = poly
            .to_svg()
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(200, 0, 0))
            .with_fill_opacity(0.2);
        svg
    }
}

pub fn mean_point(pa: &Point, pb: &Point) -> Point {
    point! { x: (pb.x()-pa.x())/2., y:(pb.y()-pa.y()) / 2.}
}
