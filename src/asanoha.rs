use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{triangle, InfiniteLine};
use geo::{AffineOps, AffineTransform};
use geo_svg::ToSvg;
use geo_types::{LineString, Point, Polygon};

pub struct Asanoha {
    pub polygon: Polygon,
    points: Vec<Point>,
}

pub struct AsanohaConfig {
    parent: KumikoConfig,
}
pub trait AsanohaConfigTrait: KumikoConfigTrait {}

impl AsanohaConfigTrait for AsanohaConfig {}
impl KumikoConfigTrait for AsanohaConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}
//use geo_svg::ToSvg;

impl KumikoFigure<AsanohaConfig> for Asanoha {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        return &self.points;
    }

    fn draw_figure(&self) -> geo_svg::Svg {
        let poly = self.polygon();
        let svg: geo_svg::Svg = poly
            .to_svg()
            .with_stroke_width(0.01)
            .with_stroke_color(geo_svg::Color::Rgb(200, 0, 0))
            .with_fill_opacity(0.2);
        svg
    }
}

/// Implements methods to help creating the Asanoha
///
/// The base is given by the Origin and vectors u and v:
///
///        2
///     v /
///      O -- 1
///        u
///
impl Asanoha {
    //     * --- 2             2
    //    /    / 3 \         v /
    //   *    O --- 1         O -- 1
    //    \        /             u
    //     * --- *
    fn points(base: &Base) -> Vec<Point> {
        let origin = base.origin;
        let pts = vec![
            origin,                          // O
            origin + base.u,                 // 1
            origin + base.v,                 // 2
            (origin + base.u + base.v) / 3., // 3
        ];
        pts
    }
    fn contour(base: &Base) -> Vec<Point> {
        //     * --- 2             2
        //    /    / 3 \         v /
        //   *    O --- 1         O -- 1
        //    \        /             u
        //     * --- *
        let origin = base.origin;
        let pts = vec![
            origin + base.u,          // 1
            origin + base.v,          // 2
            origin - base.u + base.v, // 3
            origin - base.u,          // 4
            origin - base.v,          // 5
            origin + base.u - base.v, // 6
        ];
        pts
    }

    pub fn new_at_base(base: &Base, config: &dyn AsanohaConfigTrait) -> Self {
        let pts: Vec<Point> = Asanoha::points(base);

        let inner_lines: Vec<InfiniteLine> = vec![
            InfiniteLine::from_to(&pts[0], &pts[1]).shift_by(-config.width_fine() / 2.), // 0 -> 1
            InfiniteLine::from_to(&pts[1], &pts[3]).shift_by(-config.width_fine() / 2.), // 1 -> 3
            InfiniteLine::from_to(&pts[3], &pts[0]).shift_by(-config.width_fine() / 2.), // 3 -> 1
        ];

        let tri_base: Polygon = triangle(&inner_lines[0], &inner_lines[1], &inner_lines[2]); // 0 - 1 - 3

        let mut three_tri: Vec<Polygon> = vec![tri_base.clone()];
        vec![120., 240.0].iter().for_each(|phi| {
            let t: AffineTransform = AffineTransform::rotate(*phi, pts[3]);
            let tri = tri_base.affine_transform(&t);
            three_tri.push(tri);
        });

        let mut all_polygons: Vec<Polygon> = three_tri.clone();
        for p in three_tri.iter() {
            vec![60., 120., 180., 240., 300.].iter().for_each(|phi| {
                let t: AffineTransform = AffineTransform::rotate(*phi, pts[0]);
                let tri = p.affine_transform(&t);
                all_polygons.push(tri);
            })
        }

        let holes: Vec<LineString> = all_polygons.iter().map(|p| p.exterior().clone()).collect();
        let contour = Asanoha::contour(base);
        Asanoha {
            polygon: Polygon::new(LineString(contour.iter().map(|p| p.0).collect()), holes),
            points: contour,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::HoneycombGrid;
    use crate::kumiko::KumikoConfig;
    use crate::svg::save_polygon_as_svg;
    use geo_types::point;

    fn make_base() -> Base {
        let side: f64 = 4.0;
        let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
        let side_1o2 = side / 2.0;

        Base::new(
            point! { x: 0., y: 0. },
            point! {x: side_r3o2, y: -side_1o2},
            point! {x: side_r3o2, y: side_1o2},
        )
    }

    fn make_config() -> AsanohaConfig {
        AsanohaConfig {
            parent: KumikoConfig {
                width_fine: 0.15,
                width_outer: 0.2,
            },
        }
    }

    /// Creates a unique asanoha made of triskells and eventail
    ///
    /// ´´´
    ///  .-^-.
    ///  |   |
    ///   `v´
    /// ´´´
    #[test]
    pub fn asanoha() {
        println!("#########################################################");

        // creates an asanoha

        let base = make_base();
        let figure = Asanoha::new_at_base(&base, &make_config());

        // Extract a polygon from the asanoha
        let phexa = figure.polygon;
        save_polygon_as_svg(&phexa, "test_figures/asanoha1.svg");

        // Repeat polygon on a honeycomb grid
        let dx: f64 = base.u.x();
        let dy: f64 = 3. * base.u.y();
        let mut grid = HoneycombGrid::new(dx, dy, 2, 5);
        let figure = grid.fill_with_unit(&phexa);

        save_polygon_as_svg(&figure, "test_figures/asanoha_plane.svg");
    }
}
