use crate::oglines::{mean_point, points2geometry, triangle, InfiniteLine, KumikoConfig, LinesLR};
use core::f64::consts::PI;

use crate::svg::write_svg;
use geo::{AffineOps, AffineTransform};
use geo_svg::{Color, ToSvg};
use geo_types::{LineString, Point, Polygon};

pub struct Flower {
    pub polygon: Polygon,
}

impl Flower {
    /// The points must form a square
    pub fn new_inside_box(pts: &[Point], config: KumikoConfig) -> Self {
        let pa = pts[0].clone();
        let pb = mean_point(&pts[0], &pts[1]);
        let pc = mean_point(&pts[0], &pts[2]);
        let pd: Point = mean_point(&pts[0], &pts[3]);

        let base_polygons = Flower::make_base_polygons(&[pa, pb, pc, pd], config);

        let mut all_polygons: Vec<Polygon> = base_polygons.clone();
        for p in base_polygons.iter() {
            vec![90.0, 180.0, 270.0].iter().for_each(|phi| {
                let t: AffineTransform = AffineTransform::rotate(*phi, pc.0);
                let p2 = p.affine_transform(&t);
                all_polygons.push(p2);
            })
        }
        let holes: Vec<LineString> = all_polygons.iter().map(|p| p.exterior().clone()).collect();
        Flower {
            polygon: Polygon::new(LineString(pts.iter().map(|p| p.0).collect()), holes),
        }
    }
    fn make_base_polygons(pts: &[Point], config: KumikoConfig) -> Vec<Polygon> {
        /*
         * Base lines
         */
        let inner_lines: Vec<InfiniteLine> = vec![
            InfiniteLine::from_to(&pts[0], &pts[1]).shift_by(-config.width_outer),
            InfiniteLine::from_to(&pts[1], &pts[2]),
            InfiniteLine::from_to(&pts[2], &pts[3]),
            InfiniteLine::from_to(&pts[3], &pts[0]).shift_by(-config.width_outer),
        ];
        let pts = vec![
            inner_lines[3].intersection(&inner_lines[0]),
            inner_lines[0].intersection(&inner_lines[1]),
            inner_lines[1].intersection(&inner_lines[2]),
            inner_lines[2].intersection(&inner_lines[3]),
        ];

        let _la0 = InfiniteLine::from_point_angle(&pts[0], 0.).shift_by(config.width_outer);

        // Lines from point A
        let la: Vec<LinesLR> = (0..=4)
            .map(|i| {
                let cline = InfiniteLine::from_point_angle(&pts[0], (i as f64) * PI / 8.);
                LinesLR::new(cline, config.width_fine)
            })
            .collect();

        // Lined from point B
        let cline = InfiniteLine::from_point_angle(&pts[1], 3. * PI / 4.);
        let lb: LinesLR = LinesLR::new(cline, config.width_fine);

        //Lines from point C
        let lc: Vec<LinesLR> = (8..=12)
            .map(|i| {
                let cline = InfiniteLine::from_point_angle(&pts[2], (i as f64) * PI / 8.);
                LinesLR::new(cline, config.width_fine)
            })
            .collect();

        // Line from point D
        let cline = InfiniteLine::from_point_angle(&pts[3], -PI / 4.);
        let ld: LinesLR = LinesLR::new(cline, config.width_fine);

        // Polygon in first quater
        // D-----C
        // | E   |
        // |   F |
        // A-----B
        let mut polygons: Vec<Polygon> = Vec::new();
        polygons.push(triangle(&la[0].l, &la[1].r, &lb.l)); // ABF
        polygons.push(triangle(&la[1].l, &la[2].r, &lc[3].r)); // AFC
        polygons.push(triangle(&la[2].l, &lc[1].l, &la[3].r)); // ACE
        polygons.push(triangle(&la[3].l, &ld.r, &la[4].r)); // AED
        polygons.push(triangle(&lc[0].l, &ld.l, &lc[1].r)); // CDE
        polygons.push(triangle(&lc[3].l, &lb.r, &lc[4].r)); // CAF
        polygons
    }
}

pub fn draw_test_flower(filename: &str) {
    // Make the basic poligons inside the base square
    let side = 4.;
    let p_a = Point::new(0., 0.);
    let p_b = Point::new(side, 0.);
    let p_c = Point::new(side, side);
    let p_d = Point::new(0., side);
    let points = points2geometry(vec![p_a, p_b, p_c, p_d]);

    let config = KumikoConfig::default();
    let flower: Flower = Flower::new_inside_box(&[p_a, p_b, p_c, p_d], config);

    /*
       let clip: Polygon = Polygon::new(
           LineString(vec![
               Coord { x: -0.5, y: -0.5 },
               Coord { x: 1.5, y: -0.5 },
               Coord { x: 1.5, y: 1.5 },
               Coord { x: -0.5, y: 1.5 },
           ]),
           vec![],
       );

       let clipline: LineString = LineString(vec![
           Coord { x: -0.5, y: -0.5 },
           Coord { x: 1.5, y: -0.5 },
           Coord { x: 1.5, y: 1.5 },
           Coord { x: -0.5, y: 1.5 },
       ]);
       let mclipline = MultiLineString::new(vec![clipline]);

       let mut clipped: Vec<MultiPolygon> = Vec::new();
       clipped = all_polygons
           .iter()
           .map(|p: &Polygon| p.intersection(&clip, 1.0))
           .collect();
    */

    // let all = GeometryCollection::new_from(vec![Geometry::GeometryCollection(points), mpolys.into()]);
    // let mut g_svg = all.to_svg();
    let svg_points = points
        .to_svg()
        .with_radius(0.02)
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(100, 0, 200))
        .with_fill_opacity(0.2);
    let svg_flower = flower
        .polygon
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(200, 0, 0))
        .with_fill_opacity(0.2);

    let svg = svg_points.and(svg_flower).to_string();

    write_svg(&svg, filename);
    //    rewrite_svg(&svg);
}
