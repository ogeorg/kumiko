use crate::eventail::{Eventail, EventailConfigTrait};
use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::triskell::{Triskell, TriskellConfigTrait};
use geo::{polygon, AffineOps, AffineTransform};
use geo_svg::ToSvg;
use geo_types::{Coord, LineString, Point, Polygon};

pub struct Hexagon {
    pub polygon: Polygon,
    points: Vec<Point>,
}

pub struct HexagonConfig {
    space: f64,
    parent: KumikoConfig,
}
pub trait HexagonConfigTrait:
    KumikoConfigTrait + EventailConfigTrait + TriskellConfigTrait
{
    fn as_eventail_config(&self) -> &dyn EventailConfigTrait;
    fn as_triskell_config(&self) -> &dyn TriskellConfigTrait;
}

impl HexagonConfigTrait for HexagonConfig {
    fn as_eventail_config(&self) -> &dyn EventailConfigTrait {
        self
    }
    fn as_triskell_config(&self) -> &dyn TriskellConfigTrait {
        self
    }
}
impl TriskellConfigTrait for HexagonConfig {
    fn space(&self) -> f64 {
        self.space
    }
}
impl EventailConfigTrait for HexagonConfig {}
impl KumikoConfigTrait for HexagonConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}
//use geo_svg::ToSvg;

impl KumikoFigure<HexagonConfig> for Hexagon {
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
///     3    4
///    v \  / w
///       O -- 1
///          u
///
impl Hexagon {
    fn points(base: &Base) -> Vec<Point> {
        let origin = base.origin;
        let w = base.u + base.v;
        //     3 --- 2         3    4
        //    /  \ /  \       v \  / w
        //   4    O -- 1         O -- 1
        //    \       /             u
        //     5 --- 6
        let pts = vec![
            origin + base.u, // 1
            origin + w,      // 2
            origin + base.v, // 3
            origin - base.u, // 4
            origin - w,      // 5
            origin - base.v, // 6
        ];
        pts
    }
    pub fn new_at_base(base: &Base, config: &dyn HexagonConfigTrait) -> Self {
        let pts = Hexagon::points(base);
        let mut hexa = Hexagon {
            polygon: polygon!(),
            points: pts.clone(),
        };

        let contour_line: LineString = LineString::from(pts);

        let mut interiors: Vec<LineString> = Vec::new();

        hexa.add_eventail(&base, config, &mut interiors);

        let base_triskell = Base::new(base.origin, base.v, -base.u);
        hexa.add_triskells(&base_triskell, config, &mut interiors);
        hexa.polygon = Polygon::new(contour_line, interiors);

        hexa
    }

    /// Creates an eventail and adds it to the list of interior line-strings
    fn add_eventail(
        &self,
        base: &Base,
        config: &dyn HexagonConfigTrait,
        interiors: &mut Vec<LineString>,
    ) {
        let eventail: Eventail = Eventail::new_at_base(&base, config.as_eventail_config());
        eventail.polygon().interiors().iter().for_each(|p| {
            interiors.push(p.clone());
        });
    }

    /// Creates some triskells and adds then to the list of interior line-strings
    fn add_triskells(
        &self,
        base: &Base,
        config: &dyn HexagonConfigTrait,
        interiors: &mut Vec<LineString>,
    ) {
        let triskell: Triskell = Triskell::new_at_base(&base, config.as_triskell_config());

        let mut triskell_poly = triskell.polygon().clone();
        triskell_poly.interiors().iter().for_each(|p| {
            interiors.push(p.clone());
        });

        let t: AffineTransform = AffineTransform::rotate(60., Coord { x: 0., y: 0. });
        for _i in 0..3 {
            triskell_poly = triskell_poly.affine_transform(&t);
            triskell_poly.interiors().iter().for_each(|p| {
                interiors.push(p.clone());
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, SimpleFrame};
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
            point! {x: side_r3o2, y: side_1o2},
            point! {x: -side_r3o2, y: side_1o2},
        )
    }

    fn make_config() -> HexagonConfig {
        HexagonConfig {
            space: 0.75,
            parent: KumikoConfig::default(),
        }
    }

    /// Creates a unique hexagon made of triskells and eventail
    ///
    /// ´´´
    ///  .-^-.
    ///  |   |
    ///   `v´
    /// ´´´
    #[test]
    pub fn hexagon() {
        println!("#########################################################");

        // creates an hexagon
        let base = make_base();
        let hexa = Hexagon::new_at_base(&base, &make_config());

        // Extract a polygon from the hexagon
        let phexa = hexa.polygon;
        save_polygon_as_svg(&phexa, "test_figures/hexa1.svg");

        // Repeat polygon on a honeycomb grid
        let dx: f64 = base.u.x();
        let dy = 3. * base.u.y();
        let nx = 4;
        let ny = 5;
        let mut grid = HoneycombGrid::new(dx, dy, nx, ny);
        let inner_figure = grid.fill_with_unit(&phexa);

        let width: f64 = (2 * nx - 2) as f64 * dx;
        let height: f64 = (ny - 1) as f64 * dy;
        let frame = SimpleFrame::new(point! {x:0.0, y:0.0}, width, height, 1.0);
        let figure = frame.frame(&inner_figure);
        save_polygon_as_svg(&figure, "test_figures/plane.svg");
    }
}
