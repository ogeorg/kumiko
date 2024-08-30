use crate::oglines::{points2geometry, triangle, InfiniteLine, KumikoConfig, LinesLR};
use crate::svg::write_svg;
use core::f64::consts::PI;
use geo::MultiPoint;
use geo_svg::{Color, ToSvg};
use geo_types::{point, LineString, MultiLineString, Point, Polygon};

pub struct Eventail {
    pub polygon: Polygon,
    pub rays: MultiLineString,
}

impl Eventail {
    /// The points must form a rhomboid
    ///

    pub fn new_inside_box(pts: &MultiPoint, config: &KumikoConfig) -> Self {
        let pts = &pts.0;
        let base_polygons = Eventail::make_base_polygons(pts, &config);
        let holes: Vec<LineString> = base_polygons
            .iter()
            .map(|poly| poly.exterior().clone())
            .collect();
        let rays = Eventail::make_rays(pts);

        let outer_lines: Vec<InfiniteLine> = vec![
            InfiniteLine::from_to(&pts[0], &pts[1]).shift_by(config.width_outer),
            InfiniteLine::from_to(&pts[1], &pts[2]).shift_by(config.width_outer),
            InfiniteLine::from_to(&pts[2], &pts[3]).shift_by(config.width_outer),
            InfiniteLine::from_to(&pts[3], &pts[0]).shift_by(config.width_outer),
        ];
        let pts = &[
            outer_lines[3].intersection(&outer_lines[0]),
            outer_lines[0].intersection(&outer_lines[1]),
            outer_lines[1].intersection(&outer_lines[2]),
            outer_lines[2].intersection(&outer_lines[3]),
        ];

        Eventail {
            polygon: Polygon::new(LineString(pts.iter().map(|p| p.0).collect()), holes),
            rays,
        }
    }

    fn make_rays(pts: &[Point]) -> MultiLineString {
        let lra: Vec<LineString> = (2..=10)
            .map(|n| {
                let l = InfiniteLine::from_point_angle(&pts[0], n as f64 * PI / 12.);
                let positions = l.at_times(vec![0., 1.]);
                LineString::new(positions.iter().map(|p| p.0).collect())
            })
            .collect();
        MultiLineString(lra)
    }

    fn make_base_polygons(pts: &[Point], config: &KumikoConfig) -> Vec<Polygon> {
        // Lines LR from the bottom point Pa
        let lra: Vec<LinesLR> = (2..=10)
            .map(|n| {
                let l = InfiniteLine::from_point_angle(&pts[0], n as f64 * PI / 12.);
                LinesLR::new(l, config.width_fine)
            })
            .collect();

        // Lines LR from the right point Pb
        let lb = InfiniteLine::from_point_angle(&pts[1], 5. * PI / 6.);
        let lrb = LinesLR::new(lb, config.width_fine);

        // Lines LR from the left point Pd
        let ld = InfiniteLine::from_point_angle(&pts[3], PI / 6.);
        let lrd = LinesLR::new(ld, config.width_fine);

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
pub fn draw_test_eventail(filename: &str) {
    let side: f64 = 4.0;
    let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
    let side_1o2 = side / 2.0;
    let config = KumikoConfig::default();

    let p_a = point! { x: 0., y: 0. }; // base point is the bottom of the eventail
    let p_b = point! {x: side_r3o2, y: side_1o2};
    let p_c = point! {x:0., y: side};
    let p_d = point! {x: -side_r3o2, y: side_1o2};

    let pts_eventail: Vec<Point> = vec![p_a, p_b, p_c, p_d];
    let eventail: Eventail = Eventail::new_inside_box(&pts_eventail.into(), &config);

    let svg_rays = eventail
        .rays
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(100, 0, 200));

    let points = points2geometry(vec![p_a, p_b, p_c, p_d]);
    let svg_points = points
        .to_svg()
        .with_radius(0.02)
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(100, 0, 200))
        .with_fill_opacity(0.2);
    let svg_eventail = eventail
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
