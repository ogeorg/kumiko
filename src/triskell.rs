use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{linestring_from_lines, InfiniteLine, LinesLR};
use geo::{AffineOps, AffineTransform};
use geo_types::{LineString, MultiLineString, Point, Polygon};

pub struct Triskell {
    polygon: Polygon,
    pub rays: MultiLineString,
    points: Vec<Point>,
}

pub struct TriskellConfig {
    space: f64,
    parent: KumikoConfig,
}

pub trait TriskellConfigTrait: KumikoConfigTrait {
    fn space(&self) -> f64;
}
impl TriskellConfigTrait for TriskellConfig {
    fn space(&self) -> f64 {
        self.space
    }
}
impl KumikoConfigTrait for TriskellConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}

impl KumikoFigure<TriskellConfig> for Triskell {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        return &self.points;
    }
}

impl Triskell {
    pub fn new_at_base(base: &Base, config: &dyn TriskellConfigTrait) -> Self {
        //    fn new_inside_box(pts: &MultiPoint, space: f64, config: &KumikoConfig) -> Self {
        let origin = base.origin;
        let pa = base.origin + base.u;
        let pb = base.origin + base.v;

        // the 3 sides
        let lab = InfiniteLine::from_to(&origin, &pa);
        let lbc = InfiniteLine::from_to(&pa, &pb);
        let lca = InfiniteLine::from_to(&pb, &origin);

        // left/right around the sides
        let lablr = LinesLR::new(&lab, config.width_fine());
        let _lbclr = LinesLR::new(&lbc, config.width_fine());
        let lcalr = LinesLR::new(&lca, config.width_fine());

        // The linesLR offset by space
        let space = config.space();
        let lab2lr = LinesLR::new(&lab.shift_by(-space), config.width_fine());
        let lbc2lr = LinesLR::new(&lbc.shift_by(-space), config.width_fine());
        let lca2lr = LinesLR::new(&lca.shift_by(-space), config.width_fine());

        let outer_lines: LineString = linestring_from_lines(vec![
            &lab.shift_by(config.width_outer()),
            &lbc.shift_by(config.width_outer()),
            &lca.shift_by(config.width_outer()),
        ]);

        let linestring_a = linestring_from_lines(vec![&lablr.l, &lbc2lr.l, &lab2lr.r, &lcalr.l]);

        let center = (origin + pa + pb) / 3.;
        let t: AffineTransform = AffineTransform::rotate(120., center.0);
        let linestring_b = linestring_a.affine_transform(&t);
        let linestring_c = linestring_b.affine_transform(&t);

        let linestring_o = linestring_from_lines(vec![&lab2lr.l, &lbc2lr.l, &lca2lr.l]);

        let poly = Polygon::new(
            outer_lines,
            vec![linestring_a, linestring_b, linestring_c, linestring_o],
        );
        Triskell {
            polygon: poly,
            points: vec![origin, pa, pb],
            rays: Triskell::make_rays(vec![&lab2lr.r, &lbc2lr.r, &lca2lr.r]),
        }
    }

    fn make_rays(lines: Vec<&InfiniteLine>) -> MultiLineString {
        let lra: Vec<LineString> = lines
            .iter()
            .map(|line| {
                let positions = line.at_times(vec![0., 1.]);
                LineString::new(positions.iter().map(|p| p.0).collect())
            })
            .collect();
        MultiLineString(lra)
    }
}

#[cfg(test)]
mod tests {
    use crate::kumiko::{Base, KumikoConfig, KumikoFigure};
    use crate::oglines::points2geometry;
    use crate::svg::write_svg;
    use geo_svg::{Color, ToSvg};
    use geo_types::point;

    use super::*;
    #[test]
    pub fn draw_test_triskell() {
        let side: f64 = 4.0;
        let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
        let side_1o2 = side / 2.0;

        //    __
        // C |  --__
        //   |    __* B
        // A |__--
        let pa = point! {x: 0., y: 0. };
        let pb = point! {x: side_r3o2, y: side_1o2};
        let pc = point! {x:0., y: side};
        let points = points2geometry(&[pa, pb, pc]);

        let config = TriskellConfig {
            space: 0.75,
            parent: KumikoConfig::default(),
        };

        let base = Base::new(pa, pb - pa, pc - pa);
        let triskell: Triskell = Triskell::new_at_base(&base, &config);

        let svg_rays = triskell
            .rays
            .to_svg()
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200));
        let svg_points = points
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200))
            .with_fill_opacity(0.2);
        let svg_eventail = triskell.draw_figure();

        let svg = svg_eventail //
            .and(svg_rays) //
            .and(svg_points) //
            .to_string();

        write_svg(&svg, "test_figures/triskel.svg");
    }
}
