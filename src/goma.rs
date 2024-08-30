use crate::kumiko::{Base, KumikoConfig, KumikoConfigTrait, KumikoFigure};
use crate::oglines::{linestring_from_lines, InfiniteLine, LinesLR};
use geo::{AffineOps, AffineTransform, Coord};
use geo_types::{LineString, MultiLineString, Point, Polygon};

pub struct GomaConfig {
    /// Space between a border line and an interior line
    space: f64,
    parent: KumikoConfig,
}

pub trait GomaConfigTrait: KumikoConfigTrait {
    fn space(&self) -> f64;
}
impl GomaConfigTrait for GomaConfig {
    fn space(&self) -> f64 {
        self.space
    }
}

impl KumikoConfigTrait for GomaConfig {
    fn width_outer(&self) -> f64 {
        self.parent.width_outer()
    }

    fn width_fine(&self) -> f64 {
        self.parent.width_fine()
    }
}

impl<'b> KumikoFigure<GomaConfig> for Goma<'b> {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        return &self.points;
    }
}

pub struct Goma<'b> {
    polygon: Polygon,
    pub rays: MultiLineString,
    points: Vec<Point>,
    base: &'b Base,
}

impl<'b> Goma<'b> {
    pub fn new_at_base(base: &'b Base, config: &dyn GomaConfigTrait) -> Self {
        //    fn new_inside_box(pts: &MultiPoint, space: f64, config: &KumikoConfig) -> Self {
        let pa = base.origin;
        let pb = base.origin + base.u;
        let pc = base.origin + base.v;

        // the 3 sides
        let lab = InfiniteLine::from_to(&pa, &pb);
        let lbc = InfiniteLine::from_to(&pb, &pc);
        let lca = InfiniteLine::from_to(&pc, &pa);

        // left/right around the sides
        let lablr = LinesLR::new(&lab, config.width_fine());
        let lbclr = LinesLR::new(&lbc, config.width_fine());

        // The linesLR offset by space
        let space = config.space();
        let lab2lr = LinesLR::new(&lab.shift_by(-space), config.width_fine());
        let lbc2lr = LinesLR::new(&lbc.shift_by(-space), config.width_fine());
        let lca2lr = LinesLR::new(&lca.shift_by(-space), config.width_fine());

        // central triangle
        let centre_triangle = linestring_from_lines(vec![&lab2lr.l, &lbc2lr.l, &lca2lr.l]);
        let vertex_quad = linestring_from_lines(vec![&lablr.l, &lbclr.l, &lab2lr.r, &lbc2lr.r]);
        let side_quad = linestring_from_lines(vec![&lablr.l, &lbc2lr.l, &lab2lr.r, &lca2lr.l]);

        let center = (pa + pb + pc) / 3.;
        let t: AffineTransform = AffineTransform::rotate(120., center);
        let t2: AffineTransform = AffineTransform::rotate(240., center);

        let outer_lines: LineString = linestring_from_lines(vec![
            &lab.shift_by(config.width_outer()),
            &lbc.shift_by(config.width_outer()),
            &lca.shift_by(config.width_outer()),
        ]);

        let poly: Polygon = Polygon::new(
            outer_lines,
            vec![
                centre_triangle,
                vertex_quad.affine_transform(&t),
                vertex_quad.affine_transform(&t2),
                vertex_quad,
                side_quad.affine_transform(&t),
                side_quad.affine_transform(&t2),
                side_quad,
            ],
        );
        Goma {
            polygon: poly,
            points: vec![pa, pb, pc],
            rays: Goma::make_rays(vec![&lab2lr.r, &lbc2lr.r, &lca2lr.r]),
            base,
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

pub struct GomaHexagon {
    polygon: Polygon,
    points: Vec<Point>,
}

impl KumikoFigure<GomaConfig> for GomaHexagon {
    fn polygon(&self) -> &Polygon {
        &self.polygon
    }
    fn points(&self) -> &[Point] {
        return &self.points;
    }
}

impl GomaHexagon {
    pub fn new_at_base(base: &Base, config: &dyn GomaConfigTrait) -> Self {
        let unit = Goma::new_at_base(base, config);
        let pa = base.origin + base.u;

        let mut points: Vec<Point> = Vec::new();
        let mut coords: Vec<Coord> = Vec::new();
        let mut polygons: Vec<LineString> = Vec::new();
        (0..6)
            .into_iter()
            .map(|i: i32| {
                let phi = (i as f64) * 60.;
                AffineTransform::rotate(phi, base.origin)
            })
            .for_each(|t| {
                let p = pa.affine_transform(&t);
                points.push(p);
                coords.push(p.0);
                let interiors: &[LineString] = unit.polygon.interiors();
                interiors
                    .into_iter()
                    .for_each(|ls| polygons.push(ls.affine_transform(&t)));
            });

        let poly: Polygon = Polygon::new(LineString::new(coords), polygons);
        GomaHexagon {
            polygon: poly,
            points: points,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::HoneycombGrid;
    use crate::kumiko::{Base, KumikoConfig, KumikoFigure};
    use crate::oglines::points2geometry;
    use crate::operations::intersect;
    use crate::svg::{save_polygon_as_svg, write_svg};
    use geo::Translate;
    use geo_svg::{Color, ToSvg};
    use geo_types::point;

    fn make_config() -> GomaConfig {
        let config = GomaConfig {
            space: 0.75,
            parent: KumikoConfig::default(),
        };
        config
    }
    fn make_points() -> [Point; 3] {
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
        [pa, pb, pc]
    }
    use super::*;
    #[test]
    pub fn draw_test_goma() {
        let [pa, pb, pc] = make_points();

        let base = Base::new(pa, pb - pa, pc - pa);
        let goma: Goma = Goma::new_at_base(&base, &make_config());

        let svg_rays = goma
            .rays
            .to_svg()
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200));

        let points: geo::GeometryCollection = points2geometry(&[pa, pb, pc]);
        let svg_points = points
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200))
            .with_fill_opacity(0.2);
        let svg_base = goma.base.draw();

        let svg_figure = goma.draw_figure();
        let svg = svg_figure //
            .and(svg_rays) //
            .and(svg_points) //
            .and(svg_base)
            .to_string();

        write_svg(&svg, "test_figures/goma.svg");
    }

    #[test]
    pub fn draw_test_goma2() {
        let [pa, pb, pc] = make_points();

        let base = Base::new(pa, pb - pa, pc - pa);
        let goma2: GomaHexagon = GomaHexagon::new_at_base(&base, &make_config());

        let points: geo::GeometryCollection = points2geometry(&[pa, pb, pc]);
        let svg_points = points
            .to_svg()
            .with_radius(0.02)
            .with_stroke_width(0.01)
            .with_stroke_color(Color::Rgb(100, 0, 200))
            .with_fill_opacity(0.2);
        let svg_figure = goma2.draw_figure();
        let svg_base = base.draw();

        let svg = svg_figure //
            .and(svg_points) //
            .and(svg_base)
            .to_string();

        write_svg(&svg, "test_figures/gomahax.svg");
    }

    #[test]
    pub fn draw_test_gomaplane() {
        let [pa, pb, pc] = make_points();

        let base = Base::new(pa, pb - pa, pc - pa);
        let goma2: GomaHexagon = GomaHexagon::new_at_base(&base, &make_config());

        // Extract a polygon from the hexagon
        let phexa = goma2.polygon;

        // Makes a list of LineString's
        let figure = make_honeycomb_grid(&base, &phexa);
        save_polygon_as_svg(&figure, "test_figures/goma_plane.svg");
    }

    fn make_honeycomb_grid(base: &Base, unit: &Polygon) -> Polygon {
        let dx: f64 = base.u.x();
        let dy = 3. * base.u.y();

        let mut interiors: Vec<LineString> = Vec::new();

        let grid = HoneycombGrid::new(dx, dy, 4, 5);
        let origins: &Vec<Coord<f64>> = &grid.nodes;

        //
        for origin in origins {
            let copy = unit.translate(origin.x, origin.y);
            copy.interiors().iter().for_each(|p| {
                interiors.push(p.clone());
            });
        }

        //    let figure = clip(interiors, contour_line, clipping_line);
        let contour_line = grid.contour_large();
        let clipping_line = grid.contour_small();
        let figure = intersect(interiors, contour_line, clipping_line);
        figure
    }
}
