use geo::{Point, Polygon, Translate};
use geo_types::{coord, Coord, LineString};

use crate::operations::{clip, intersect};

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
pub struct HoneycombGrid {
    /// horizontal distance from on figure to another
    dx: f64,
    /// vertical distance from on figure to another
    dy: f64,
    /// number of copies horizontally
    nx: usize,
    /// number of copies vertically
    ny: usize,
    /// Keeps track of where to place each unit in the grid
    pub nodes: Vec<Coord<f64>>,
}

impl HoneycombGrid {
    pub fn new(dx: f64, dy: f64, nx: usize, ny: usize) -> HoneycombGrid {
        let mut grid: HoneycombGrid = HoneycombGrid {
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

    pub fn fill_with_unit(&mut self, unit: &Polygon) -> Vec<LineString> {
        let mut interiors: Vec<LineString> = Vec::new();

        let origins: &Vec<Coord<f64>> = &self.nodes;

        //
        for origin in origins {
            let copy = unit.translate(origin.x, origin.y);
            copy.interiors().iter().for_each(|p| {
                interiors.push(p.clone());
            });
        }
        interiors
    }
}
