use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{angle_between, triangle, InfiniteLine, LinesLR, Rotation};
use geo_types::{LineString, MultiLineString, Point, Polygon};

/// An eventail (fan)
pub struct Eventail<'b> {
    polygon: Polygon,
    pub rays: MultiLineString,
    points: Vec<Point>,
    base: &'b Base,
}

pub struct EventailConfig {
    parent: KumikoConfig,
}
pub trait EventailConfigTrait: KumikoConfigTrait {}

impl EventailConfigTrait for EventailConfig {}

impl KumikoConfigTrait for EventailConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}

/// A rhombo-like eventail (fan)
impl<'b> KumikoFigure<EventailConfig> for Eventail<'b> {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        return &self.points;
    }
}

impl<'b> Eventail<'b> {
    /// The points must form a rhomboid
    ///
    pub fn new_at_base(base: &'b Base, config: &dyn EventailConfigTrait) -> Self {
        let pu = base.origin + base.u;
        let pv = base.origin + base.v;
        let puv = pu + base.v;

        let corners = vec![base.origin, pu, puv, pv];
        let lines = Eventail::make_lines(base, &corners);
        let base_polygons = Eventail::make_base_polygons(&lines, config);
        let holes: Vec<LineString> = base_polygons
            .iter()
            .map(|poly| poly.exterior().clone())
            .collect();
        let rays = Eventail::make_rays(&lines);

        let outer_lines: Vec<InfiniteLine> = vec![
            InfiniteLine::from_to(&corners[0], &corners[1]).shift_by(config.width_outer()),
            InfiniteLine::from_to(&corners[1], &corners[2]).shift_by(config.width_outer()),
            InfiniteLine::from_to(&corners[2], &corners[3]).shift_by(config.width_outer()),
            InfiniteLine::from_to(&corners[3], &corners[0]).shift_by(config.width_outer()),
        ];
        let pts = &[
            outer_lines[3].intersection(&outer_lines[0]),
            outer_lines[0].intersection(&outer_lines[1]),
            outer_lines[1].intersection(&outer_lines[2]),
            outer_lines[2].intersection(&outer_lines[3]),
        ];

        Eventail {
            polygon: Polygon::new(LineString(pts.iter().map(|p| p.0).collect()), holes),
            points: vec![base.origin, pu, puv, pv],
            rays,
            base,
        }
    }
    /// Creates all the lines
    /// Lines 0 to 8 are the "rays" lines, lines 9 and 10 are the "top" lines
    fn make_lines(base: &Base, corners: &Vec<Point>) -> Vec<InfiniteLine> {
        let angle = angle_between(&base.u, &base.v) / 8.0;
        let rotation = Rotation::by(angle);

        let mut la: Vec<InfiniteLine> = Vec::new();
        let mut v = base.u;
        for _ in 0..=8 {
            let l = InfiniteLine::from_point_vec(&base.origin, &v);
            la.push(l);
            v = rotation.rotate(&v);
        }
        la.push(InfiniteLine::from_to(&corners[1], &corners[2]));
        la.push(InfiniteLine::from_to(&corners[3], &corners[2]));
        la
    }

    /// Creates rays of length 1, MultiLineString made of vector of LineString made of segments
    fn make_rays(lines: &Vec<InfiniteLine>) -> MultiLineString {
        let ls: Vec<LineString> = lines
            .iter()
            .take(9) // first 9 lines
            .map(|l| {
                let positions = l.at_times(vec![0., 1.]);
                LineString::new(positions.iter().map(|p| p.0).collect())
            })
            .collect();
        MultiLineString(ls)
    }

    fn make_base_polygons(
        lines: &Vec<InfiniteLine>,
        config: &dyn EventailConfigTrait,
    ) -> Vec<Polygon> {
        // Lines LR from the origin
        let lra: Vec<LinesLR> = lines
            .iter()
            .take(9) // first 8 lines
            .map(|l| LinesLR::new(l, config.width_fine()))
            .collect();

        // Lines LR from the right point Pb
        let lrb = LinesLR::new(&lines[9], config.width_fine());

        // Lines LR from the left point Pd
        let lrd = LinesLR::new(&lines[10], config.width_fine());

        let mut polygons: Vec<Polygon> = Vec::new();
        polygons.push(triangle(&lra[0].l, &lrb.l, &lra[1].r));
        polygons.push(triangle(&lra[1].l, &lrb.l, &lra[2].r));
        polygons.push(triangle(&lra[2].l, &lrb.l, &lra[3].r));
        polygons.push(triangle(&lra[3].l, &lrb.l, &lra[4].r));
        polygons.push(triangle(&lra[4].l, &lrd.r, &lra[5].r));
        polygons.push(triangle(&lra[5].l, &lrd.r, &lra[6].r));
        polygons.push(triangle(&lra[6].l, &lrd.r, &lra[7].r));
        polygons.push(triangle(&lra[7].l, &lrd.r, &lra[8].r));

        polygons
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oglines::points2geometry;
    use crate::svg::write_svg;
    use geo_svg::{Color, ToSvg};
    use geo_types::point;

    fn make_points() -> Vec<Point> {
        let side: f64 = 4.0;
        let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
        let side_1o2 = side / 2.0;
        vec![
            point! { x: 0., y: 0. }, // base point is the bottom of the eventail
            point! {x: side_r3o2, y: side_1o2},
            point! {x:0., y: side},
            point! {x: -side_r3o2, y: side_1o2},
        ]
    }

    fn make_base(points: &[Point]) -> Base {
        Base::new(points[0], points[1], points[3])
    }

    #[test]
    pub fn draw_eventail() {
        let config = EventailConfig {
            parent: KumikoConfig::default(),
        };

        let points = make_points();
        let base = make_base(&points);

        let eventail: Eventail = Eventail::new_at_base(&base, &config);

        let svg_rays = eventail
            .rays
            .to_svg()
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200));

        let points = points2geometry(&points);
        let svg_points = points
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200))
            .with_fill_opacity(0.2);
        let svg_eventail = eventail.draw_figure();
        let svg_base = eventail.base.draw();

        let svg = svg_eventail //
            .and(svg_base) //
            //.and(svg_rays) //
            .and(svg_points) //
            .to_string();

        write_svg(&svg, "test_figures/eventail.svg");
    }
}
