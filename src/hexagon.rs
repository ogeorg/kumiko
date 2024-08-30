use crate::eventail::Eventail;
use crate::oglines::KumikoConfig;
use crate::operations::intersect;
use crate::svg::save_polygon_as_svg;
use crate::triskell::Triskell;
use geo::Translate;
use geo::{AffineOps, AffineTransform};
use geo_types::{coord, Coord, LineString, Point, Polygon};

/// Creates a unique hexagon made of triskells and eventail
///
/// ´´´
///  .-^-.
///  |   |
///   `v´
/// ´´´
pub fn hexagon(filename: &str) {
    let side: f64 = 4.0; // Side of one figure (triangle)
    let space: f64 = 0.5; // Space for the triskell
    let config = KumikoConfig::default();

    // creates an hexagon
    let mut hexa = Hexagon::new(side, space, &config);
    hexa.build();

    let dx = hexa.side_r3o2;
    let dy = 3. * hexa.side_1o2;

    // Extract a polygon from the hexagon
    let phexa = hexa.polygon.expect("Figure not constructed");
    save_polygon_as_svg(&phexa, "hexa1.svg");

    // Makes a list of LineString's
    let mut interiors: Vec<LineString> = Vec::new();

    let grid = Grid::new(dx, dy, 4, 5);
    let origins: &Vec<Coord<f64>> = &grid.nodes;

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
    save_polygon_as_svg(&figure, filename);
}

///
/// o | * | * | *   -      even
///  ´ ` ´ ` ´ `     dy
/// | * | * | * |   -      odd
///  ` ´ ` ´ ` ´
/// * | * | * | *          even
///    <->
///     dx
/// 0   2   4   6
///   1   3   5
/// nx = 4 / ny = 3
struct Grid {
    dx: f64,
    dy: f64,
    nx: usize,
    ny: usize,
    nodes: Vec<Coord<f64>>,
}

impl Grid {
    fn new(dx: f64, dy: f64, nx: usize, ny: usize) -> Grid {
        let mut grid: Grid = Grid {
            dx,
            dy,
            nx,
            ny,
            nodes: vec![],
        };
        let mut evenrow = true;
        let row_even: Vec<usize> = (0..=(2 * nx - 2)).step_by(2).collect();
        let row_odd: Vec<usize> = (1..(2 * nx - 1)).step_by(2).collect();
        for j in 0..ny {
            let y = dy * j as f64;
            let row = if evenrow { &row_even } else { &row_odd };
            for i in row {
                let x = dx * *i as f64;
                grid.nodes.push(coord! {x:x, y:y});
            }
            evenrow = !evenrow;
        }
        grid
    }

    fn contour_vertices(&self, margin: f64) -> Vec<Coord<f64>> {
        let xmin = margin;
        let xmax = (2 * self.nx - 2) as f64 * self.dx - margin;
        let ymin = margin;
        let ymax = (self.ny - 1) as f64 * self.dy - margin;

        vec![
            coord! {x:xmin, y:ymin},
            coord! {x:xmax, y:ymin},
            coord! {x:xmax, y:ymax},
            coord! {x:xmin, y:ymax},
        ]
    }

    fn contour_large(&self) -> LineString {
        LineString(self.contour_vertices(-0.4))
    }

    fn contour_small(&self) -> LineString {
        let margin: f64 = 0.10;
        LineString(self.contour_vertices(margin))
    }
}

struct Hexagon<'a> {
    space: f64,
    config: &'a KumikoConfig,
    side_r3o2: f64,
    side_1o2: f64,
    pts: Vec<Point>,
    pub polygon: Option<Polygon>,
}

impl Hexagon<'_> {
    fn new(side: f64, space: f64, config: &KumikoConfig) -> Hexagon {
        let side_r3o2 = side * f64::sqrt(3.0) / 2.0;
        let side_1o2 = side / 2.0;

        Hexagon {
            space,
            config,
            side_r3o2,
            side_1o2,
            polygon: Option::None,
            pts: vec![
                /* A */ (0., 0.), // base point is the bottom of the eventail
                /* B */ (side_r3o2, side_1o2),
                /* C */ (0., side),
                /* D */ (-side_r3o2, side_1o2),
                /* E */ (-side_r3o2, -side_1o2),
                /* F */ (0., -side),
                /* G */ (side_r3o2, -side_1o2),
            ]
            .iter()
            .map(|xy| Point::new(xy.0, xy.1))
            .collect(),
        }
    }
    fn build(&mut self) {
        let pts_contour: Vec<Point> = self.pts[1..].into();
        let contour_line: LineString = LineString::from(pts_contour);

        let mut interiors: Vec<LineString> = Vec::new();
        self.add_eventail(&mut interiors);

        self.add_triskell(&mut interiors);

        self.polygon = Option::Some(Polygon::new(contour_line, interiors));
    }

    fn add_eventail(&self, interiors: &mut Vec<LineString>) {
        let pts_eventail: Vec<Point> = self.pts[0..=3].into();
        let eventail: Eventail = Eventail::new_inside_box(&pts_eventail.into(), &self.config);
        eventail.polygon.interiors().iter().for_each(|p| {
            interiors.push(p.clone());
        });
    }
    fn add_triskell(&self, interiors: &mut Vec<LineString>) {
        let pts_triskell: Vec<Point> = vec![self.pts[4], self.pts[0], self.pts[3]];
        let triskell: Triskell =
            Triskell::new_inside_box(&pts_triskell.into(), self.space, &self.config);

        let mut triskell_poly = triskell.polygon;
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
