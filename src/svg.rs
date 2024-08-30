use geo_svg::Color;
use geo_svg::{Svg, ToSvg};
use geo_types::Polygon;
use std::fs;
pub fn write_svg(svg: &String, filename: &str) {
    fs::write(filename, svg).expect("Unable to write file");
}

/*
struct SvgWriter<'a> {
    filename: String,
    svg: Option<&'a Svg<'a>>,
}

impl<'a> SvgWriter<'a> {
    pub fn new(filename: String) -> SvgWriter<'a> {
        SvgWriter {
            filename,
            svg: None,
        }
    }

    pub fn add_polygon<'b>(&'a mut self, polygon: &'a Polygon) {
        match &self.svg {
            None => self.svg = Some(&polygon.to_svg()),
            Some(svg) => self.svg = Some(&svg.and(polygon.to_svg())),
        }
    }
}
*/

pub fn save_polygon_as_svg(figure: &Polygon, filename: &str) {
    let svg_figure = figure
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(200, 0, 0))
        .with_fill_opacity(0.2);

    let svg = svg_figure //
        .to_string();
    write_svg(&svg, filename);
}

pub fn polygon_to_svg<'a>(figure: &'a Polygon) -> Svg<'a> {
    let svg_figure = figure
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(200, 0, 0))
        .with_fill_opacity(0.2);
    svg_figure
}
