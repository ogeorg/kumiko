use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{triangle, InfiniteLine, LinesLR};
use core::f64::consts::PI;

use geo::{AffineOps, AffineTransform};
use geo_types::{LineString, Point, Polygon};

pub struct Flower {
    polygon: Polygon,
    points: Vec<Point>,
}

struct FlowerConfig {
    parent: KumikoConfig,
}
pub trait FlowerConfigTrait: KumikoConfigTrait {}

impl FlowerConfigTrait for FlowerConfig {}

impl KumikoConfigTrait for FlowerConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}

impl KumikoFigure<FlowerConfig> for Flower {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        &self.points
    }
}

impl Flower {
    pub fn new_at_base(base: &Base, config: &dyn FlowerConfigTrait) -> Self {
        // The base is the whole square.
        // We take the vertices from the lower left quarter of the square.
        //
        //   D .... C
        //   ^      :
        // v |      :
        //   A ---> B
        //     u
        let pa = base.origin.clone();
        let pb = pa + base.u / 2.0;
        let pc = pa + (base.u + base.v) / 2.0;
        let pd = pa + base.v / 2.0;

        // We construct the polygons within that lower left quarter
        let base_polygons: Vec<Polygon> = Flower::make_base_polygons(&[pa, pb, pc, pd], config);

        let mut all_polygons: Vec<Polygon> = base_polygons.clone();
        for p in base_polygons.iter() {
            vec![90.0, 180.0, 270.0].iter().for_each(|phi| {
                let t: AffineTransform = AffineTransform::rotate(*phi, pc.0);
                let p2 = p.affine_transform(&t);
                all_polygons.push(p2);
            })
        }

        let holes: Vec<LineString> = all_polygons.iter().map(|p| p.exterior().clone()).collect();

        let pts = vec![pa, pa + base.u, pa + base.u + base.v, pa + base.v];
        Flower {
            polygon: Polygon::new(LineString(pts.iter().map(|p| p.0).collect()), holes),
            points: vec![pa, pb, pc, pd],
        }
    }

    /// The points must form a square
    ///
    /// Polygon in first quater
    /// D-----C
    /// | E   |
    /// |   F |
    /// A-----B
    pub fn make_base_polygons(pts: &[Point], config: &dyn FlowerConfigTrait) -> Vec<Polygon> {
        /*
         * Base lines
         */
        println!("Puntos (1): {:.2?}", pts);
        let inner_lines: Vec<InfiniteLine> = vec![
            InfiniteLine::from_to(&pts[0], &pts[1]).shift_by(-config.width_outer()),
            InfiniteLine::from_to(&pts[1], &pts[2]),
            InfiniteLine::from_to(&pts[2], &pts[3]),
            InfiniteLine::from_to(&pts[3], &pts[0]).shift_by(-config.width_outer()),
        ];
        println!("Lines (1): {:.2?}", inner_lines);
        let pts = vec![
            inner_lines[3].intersection(&inner_lines[0]),
            inner_lines[0].intersection(&inner_lines[1]),
            inner_lines[1].intersection(&inner_lines[2]),
            inner_lines[2].intersection(&inner_lines[3]),
        ];
        println!("Puntos (2): {:.3?}", pts);

        // Lines from point A
        // la[0] = AB, la[1] = AF, la[2] = AC, la[3] = AE, la[4] = AD
        // lb    = BF
        // lc[0] = CD, lc[1] = CE, lc[3] = CF, lc[4] = CF
        // ld    = DE
        //
        // D-----C
        // | E   |
        // |   F |
        // A-----B
        let la: Vec<LinesLR> = (0..=4)
            .map(|i| {
                let cline = InfiniteLine::from_point_angle(&pts[0], (i as f64) * PI / 8.);
                LinesLR::new(&cline, config.width_fine())
            })
            .collect();

        // Lined from point B
        let cline = InfiniteLine::from_point_angle(&pts[1], 3. * PI / 4.);
        let lb: LinesLR = LinesLR::new(&cline, config.width_fine());

        //Lines from point C
        let lc: Vec<LinesLR> = (8..=12)
            .map(|i| {
                let cline = InfiniteLine::from_point_angle(&pts[2], (i as f64) * PI / 8.);
                LinesLR::new(&cline, config.width_fine())
            })
            .collect();

        // Line from point D
        // ld = DE
        let cline = InfiniteLine::from_point_angle(&pts[3], -PI / 4.);
        let ld: LinesLR = LinesLR::new(&cline, config.width_fine());

        // la[0] = AB
        // la[1] = AF
        // la[2] = AC
        // la[3] = AE
        // lb = BF
        // lc[0] = CD
        // lc[1] = CE
        // lc[3] = CF
        // lc[4] = CF
        // ld = DE
        // D-----C
        // | E   |
        // |   F |
        // A-----B
        let polygons: Vec<Polygon> = vec![
            triangle(&la[0].l, &la[1].r, &lb.l),    // ABF - AB-AF-BF
            triangle(&la[1].l, &la[2].r, &lc[3].r), // AFC - AF-AC-CF
            triangle(&la[2].l, &lc[1].l, &la[3].r), // ACE - AC-CE-AE
            triangle(&la[3].l, &ld.r, &la[4].r),    // AED - AE-DE-AD
            triangle(&lc[0].l, &ld.l, &lc[1].r),    // CDE - CD-DE-CE
            triangle(&lc[3].l, &lb.r, &lc[4].r),    // CAF - CF-BF-CB
        ];
        polygons
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kumiko::{Base, KumikoConfig, KumikoFigure};

    use crate::svg::write_svg;
    use geo_svg::{Color, Svg, ToSvg};
    use geo_types::Point;

    #[test]
    pub fn draw_test_flower() {
        let config = FlowerConfig {
            parent: KumikoConfig::default(),
        };

        // Make the basic poligons inside the base square
        let side = 4.;
        let pa = Point::new(0., 0.);
        let pb = Point::new(side, 0.);
        let _pc = Point::new(side, side);
        let pd = Point::new(0., side);

        let base = Base::new(pa, pb - pa, pd - pa);
        let flower: Flower = Flower::new_at_base(&base, &config);

        let points = flower.points();
        let svg_points = points
            .iter()
            .map(|pt| {
                pt.to_svg()
                    .with_radius(0.02)
                    .with_stroke_width(0.01)
                    .with_stroke_color(Color::Rgb(100, 0, 200))
                    .with_fill_opacity(0.2)
            })
            .reduce(|cur: Svg, nxt: Svg| cur.and(nxt))
            .unwrap();

        let svg = svg_points;
        let svg_flower = flower.draw_figure();
        let svg = svg.and(svg_flower).to_string();

        let svg = svg.to_string();
        write_svg(&svg, "test_figures/flower.svg");
    }
}
