use geo::BooleanOps;
use geo_clipper::Clipper;
use geo_types::{LineString, MultiLineString, Polygon};

use crate::svg::save_polygon_as_svg;
fn clip(
    interiors: Vec<LineString>,
    contour_line: LineString,
    clipping_line: LineString,
) -> Polygon {
    let clipping_poly: Polygon = Polygon::new(clipping_line, vec![]);

    for i in interiors.clone() {
        let interior = MultiLineString::new(vec![i]);
        let res = clipping_poly.clip(&interior, false);
        println!("{:?}", res);
    }

    let interiors = MultiLineString::new(interiors);
    let res = clipping_poly.clip(&interiors, false);

    let figure = Polygon::new(contour_line, res.0);
    figure
}
pub fn intersect(
    interiors: Vec<LineString>,
    contour_line: LineString,
    clipping_line: LineString,
) -> Polygon {
    let clipping_poly: Polygon = Polygon::new(clipping_line, vec![]);
    save_polygon_as_svg(&clipping_poly, "clipper.svg");

    // intersection is for polygon/polygon
    let cloned = contour_line.clone();
    let mut lines: Vec<LineString> = Vec::new();
    for inter in interiors.clone() {
        let poly: Polygon = Polygon::new(inter, vec![]);
        let res = Clipper::intersection(&poly, &clipping_poly, 1000.0);
        for p in res {
            let ext = p.exterior();
            lines.push(ext.clone());
        }
    }
    if false {
        let poly: Polygon = Polygon::new(cloned, interiors);
        let res = Clipper::intersection(&poly, &clipping_poly, 1000.0);
        for p in res {
            lines.extend_from_slice(p.interiors().into());
        }
    }
    let figure = Polygon::new(contour_line, lines);
    figure
}
