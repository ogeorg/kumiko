use geo::BooleanOps;
use geo_clipper::Clipper;
use geo_types::{LineString, MultiLineString, Polygon};

use crate::svg::save_polygon_as_svg;
pub fn clip(
    interiors: &[LineString],
    contour_line: LineString,
    clipping_line: LineString,
) -> Polygon {
    // Transforms the clipping line into a polygon
    let clipping_poly: Polygon = Polygon::new(clipping_line, vec![]);

    for interior in interiors {
        let interior = MultiLineString::new(vec![interior.clone()]);
        let res = clipping_poly.clip(&interior, false);
        dbg!(res);
    }

    // Transforms the interiors fron an array of LineString's into a MultiLineString
    // and clips the interiors to the clipping poly
    let interiors = MultiLineString::new(interiors.to_vec());
    let res = clipping_poly.clip(&interiors, false);

    // Returns a Polygon with only the clipped interior and the outer contour
    Polygon::new(contour_line, res.0)
}
pub fn intersect(
    interiors: &[LineString],
    contour_line: LineString,
    clipping_line: LineString,
) -> Polygon {
    let clipping_poly: Polygon = Polygon::new(clipping_line, vec![]);
    save_polygon_as_svg(&clipping_poly, "clipper.svg");

    // intersection is for polygon/polygon
    let cloned = contour_line.clone();
    let mut lines: Vec<LineString> = Vec::new();
    for inter in interiors {
        let poly: Polygon = Polygon::new(inter.clone(), vec![]);
        let res = Clipper::intersection(&poly, &clipping_poly, 1000.0);
        for p in res {
            let ext = p.exterior();
            lines.push(ext.clone());
        }
    }
    if false {
        let poly: Polygon = Polygon::new(cloned, interiors.to_vec());
        let res = Clipper::intersection(&poly, &clipping_poly, 1000.0);
        for p in res {
            lines.extend_from_slice(p.interiors().into());
        }
    }
    let figure = Polygon::new(contour_line, lines);
    figure
}

mod tests {
    use geo::{Coord, LineString};

    use crate::operations::intersect;
    use DoubleEndedIterator;

    fn make_linestring(xys: Vec<(f64, f64)>) -> LineString {
        LineString::new(xys.iter().map(|(x, y)| Coord { x: *x, y: *y }).collect())
    }
    #[test]
    fn clip_triangles() {
        // Given
        //
        //  O                 O
        //     X           X
        //     |        C
        //     |      / |
        //     X    /   |  X
        // -O--+--A--+--B--+--O--+--
        //

        let interiors = make_linestring(vec![(1.0, 0.0), (3.0, 0.0), (3.0, 3.0)]);
        let clipping_line = make_linestring(vec![(0.0, 1.0), (4.0, 1.0), (4.0, 4.0), (0.0, 4.0)]);
        let contour_line = make_linestring(vec![(-1.0, 0.0), (5.0, 0.0), (5.0, 5.0), (-1.0, 5.0)]);

        // When
        let clipped = intersect(&vec![interiors], contour_line, clipping_line);

        // Then
        let clipped_interiors = clipped.interiors();
        let clipped_interior = clipped_interiors.get(0).unwrap();
        let clipped_coords = clipped_interior.clone().into_inner();
        let c0: Coord<f64> = Coord { x: 3.0, y: 3.0 };
        assert_eq!(c0, *clipped_coords.get(0).unwrap());
    }
}
