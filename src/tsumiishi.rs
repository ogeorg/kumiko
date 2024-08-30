use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{polygon, InfiniteLine};
use geo::{AffineOps, AffineTransform};
use geo_svg::ToSvg;
use geo_types::{LineString, Point, Polygon};
use itertools::Itertools;

pub struct Tsumiishi {
    pub polygon: Polygon,
    points: Vec<Point>,
}

pub struct TsumiishiConfig {
    parent: KumikoConfig,
}
pub trait TsumiishiConfigTrait: KumikoConfigTrait {}

impl TsumiishiConfigTrait for TsumiishiConfig {}
impl KumikoConfigTrait for TsumiishiConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}
//use geo_svg::ToSvg;

impl KumikoFigure<TsumiishiConfig> for Tsumiishi {
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
impl Tsumiishi {
    //     * -3- 2             2
    //    /    /   1         v /
    //   *    O --- *         O -- *
    //    \        /             u
    //     * --- *
    fn points(base: &Base) -> Vec<Point> {
        let origin = base.origin;
        let pts = vec![
            origin,                          // O
            origin + (base.u + base.v) / 2., // 1
            origin + base.v,                 // 2
            origin + base.v - base.u / 2.,   // 3
        ];
        println!("{:?}", pts);
        pts
    }

    //     * --- 2             2
    //    /    / 3 \         v /
    //   *    O --- 1         O -- 1
    //    \        /             u
    //     * --- *
    fn contour(base: &Base) -> Vec<Point> {
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

    pub fn new_at_base(base: &Base, config: &dyn TsumiishiConfigTrait) -> Self {
        let pts: Vec<Point> = Tsumiishi::points(base);

        let inner_lines: Vec<InfiniteLine> = pts
            .clone()
            .into_iter()
            .circular_tuple_windows()
            .map(|(p1, p2)| InfiniteLine::from_to(&p1, &p2).shift_by(-config.width_fine() / 2.))
            .collect();

        let figure_unit: Polygon = polygon(&inner_lines); // 0 - 1 - 3

        let mut all_polygons: Vec<Polygon> = vec![figure_unit.clone()];
        vec![60., 120., 180., 240., 300.].iter().for_each(|phi| {
            let t: AffineTransform = AffineTransform::rotate(*phi, pts[0]);
            let poly = figure_unit.affine_transform(&t);
            all_polygons.push(poly);
        });

        let holes: Vec<LineString> = all_polygons.iter().map(|p| p.exterior().clone()).collect();
        let contour = Tsumiishi::contour(base);
        Tsumiishi {
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
    use crate::operations::intersect;
    use crate::svg::save_polygon_as_svg;
    use geo::Translate;
    use geo_types::{point, Coord, LineString};

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

        let config = TsumiishiConfig {
            parent: KumikoConfig {
                width_fine: 0.2,
                width_outer: 0.2,
            },
        };

        let side: f64 = 3.0;
        let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
        let side_1o2 = side / 2.0;

        let base = Base::new(
            point! { x: 0., y: 0. },
            point! {x: side_r3o2, y: -side_1o2},
            point! {x: side_r3o2, y: side_1o2},
        );

        // creates an asanoha
        let figure = Tsumiishi::new_at_base(&base, &config);

        // Extract a polygon from the asanoha
        let phexa = figure.polygon;
        save_polygon_as_svg(&phexa, "test_figures/tsumiishi1.svg");

        // Makes a list of LineString's
        let mut interiors: Vec<LineString> = Vec::new();

        let dx: f64 = side_r3o2;
        let dy = 3. * side_1o2;
        let grid: HoneycombGrid = HoneycombGrid::new(dx, dy, 7, 8);
        let origins: &Vec<Coord<f64>> = &grid.nodes;

        //
        for origin in origins {
            let copy = phexa.translate(origin.x, origin.y);
            copy.interiors().iter().for_each(|p| {
                interiors.push(p.clone());
            });
        }

        //    let figure = clip(interiors, contour_line, clipping_line);
        let contour_line = grid.contour_large();
        let clipping_line = grid.contour_small();
        let figure = intersect(interiors, contour_line, clipping_line);
        save_polygon_as_svg(&figure, "test_figures/tsumiishi_plane.svg");
    }
}
