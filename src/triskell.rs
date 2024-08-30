use crate::oglines::{
    linestring_from_lines, mean_point_vec, points2geometry, InfiniteLine, KumikoConfig, LinesLR,
};
use crate::svg::write_svg;
use geo::{AffineOps, AffineTransform};
use geo_svg::{Color, ToSvg};
use geo_types::{point, LineString, MultiLineString, MultiPoint, Point, Polygon};

pub struct Triskell {
    pub polygon: Polygon,
    pub rays: MultiLineString,
}

impl Triskell {
    pub fn new_inside_box(pts: &MultiPoint, space: f64, config: &KumikoConfig) -> Self {
        let pts = &pts.0;
        let lab = InfiniteLine::from_to(&pts[0], &pts[1]);
        let lbc = InfiniteLine::from_to(&pts[1], &pts[2]);
        let lca = InfiniteLine::from_to(&pts[2], &pts[0]);

        let lablr = LinesLR::new(lab.clone(), config.width_fine);
        let _lbclr = LinesLR::new(lbc.clone(), config.width_fine);
        let lcalr = LinesLR::new(lca.clone(), config.width_fine);

        let lab2lr = LinesLR::new(lab.shift_by(-space), config.width_fine);
        let lbc2lr = LinesLR::new(lbc.shift_by(-space), config.width_fine);
        let lca2lr = LinesLR::new(lca.shift_by(-space), config.width_fine);

        let outer_lines: LineString = linestring_from_lines(vec![
            &lab.shift_by(config.width_outer),
            &lbc.shift_by(config.width_outer),
            &lca.shift_by(config.width_outer),
        ]);

        let center = mean_point_vec(pts);

        let linestring_a = linestring_from_lines(vec![&lablr.l, &lbc2lr.l, &lab2lr.r, &lcalr.l]);

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

pub fn draw_test_triskell(filename: &str) {
    let side: f64 = 4.0;
    let space: f64 = 0.8;
    let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
    let side_1o2 = side / 2.0;

    //    __
    // C |  --__
    //   |    __* B
    // A |__--
    let p_a = point! {x: 0., y: 0. };
    let p_b = point! {x: side_r3o2, y: side_1o2};
    let p_c = point! {x:0., y: side};
    let points = points2geometry(vec![p_a, p_b, p_c]);

    let config = KumikoConfig::default();

    let pts_triskell: Vec<Point> = vec![p_a, p_b, p_c];
    let triskell: Triskell = Triskell::new_inside_box(&pts_triskell.into(), space, &config);

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
    let svg_eventail = triskell
        .polygon
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(200, 0, 0))
        .with_fill_opacity(0.2);

    let svg = svg_eventail //
        .and(svg_rays) //
        .and(svg_points) //
        .to_string();

    write_svg(&svg, filename);
}
