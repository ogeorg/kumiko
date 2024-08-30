fn rewrite_svg(svg: &String) {
    let mut g = Element::builder("g", SVG_NS).build();
    let root: Element = svg.parse().unwrap();
    for child in root.children() {
        if child.is("path", SVG_NS) {
            g.append_child(child.clone());
        }
    }

    let mut root2 = Element::builder("svg", SVG_NS)
        .attr("preserveAspectRatio", "xMidYMid meet")
        .attr("viewBox", "-1 -1 4 4")
        .build();
    root2.append_child(g.clone());

    g.set_attr("transform", "rotate(90) translate(0, 0)");
    root2.append_child(g.clone());

    g.set_attr("transform", "rotate(180) translate(0, 0)");
    root2.append_child(g.clone());

    g.set_attr("transform", "rotate(270) translate(0, 0)");
    root2.append_child(g.clone());

    let xml = String::from(&root2);
    println!("{}", xml);

    fs::write("subj2.svg", xml).expect("Unable to write file");
}

fn test_clip(filename: &str) {
    let left = 0.;
    let right = 10.;
    let grid: MultiLineString = MultiLineString::new(
        (0..=10)
            .map(|i| {
                vec![
                    LineString(vec![
                        Coord {
                            x: left,
                            y: i as f64,
                        },
                        Coord {
                            x: right,
                            y: i as f64,
                        },
                    ]),
                    LineString(vec![
                        Coord {
                            x: i as f64,
                            y: left,
                        },
                        Coord {
                            x: i as f64,
                            y: right,
                        },
                    ]),
                ]
            })
            .flatten()
            .collect(),
    );

    let triangle: Polygon = Polygon::new(
        LineString(vec![
            Coord { x: 1., y: 1. },
            Coord { x: 7., y: 1. },
            Coord { x: 4., y: 6. },
        ]),
        vec![LineString(vec![
            Coord { x: 2., y: 1.5 },
            Coord { x: 6., y: 1.5 },
            Coord { x: 4., y: 5. },
        ])],
    );

    let outer = LineString(vec![
        Coord { x: 3., y: 2. },
        Coord { x: 8., y: 2. },
        Coord { x: 8., y: 7. },
        Coord { x: 3., y: 7. },
        Coord { x: 3., y: 2. },
    ]);
    let clip: Polygon = Polygon::new(
        //
        outer.clone(),
        vec![LineString(vec![])],
    );

    let clipped1 = triangle.clip(&MultiLineString(vec![outer]), false);
    let clipped2 = clip.clip(&MultiLineString(vec![triangle.exterior().clone()]), false);
    let clipped3 = clip.clip(&MultiLineString(triangle.interiors().into()), false);

    let svg_grid = grid
        .to_svg()
        .with_stroke_width(0.005)
        .with_stroke_color(Color::Rgb(255, 0, 0));
    let svg_triangle = triangle
        .to_svg()
        .with_stroke_width(0.01)
        .with_stroke_color(Color::Rgb(0, 100, 200))
        .with_fill_opacity(0.2);
    let svg_clipped = clipped1
        .to_svg()
        .and(clipped2.to_svg())
        .and(clipped3.to_svg())
        .with_stroke_width(0.1)
        .with_stroke_color(Color::Rgb(0, 0, 200))
        .with_fill_opacity(0.2);

    let svg = svg_grid.and(svg_triangle).and(svg_clipped).to_string();

    write_svg(&svg, filename);
}
fn test_intersect() {
    let subject = Polygon::new(
        LineString(vec![
            Coord { x: 180.0, y: 200.0 },
            Coord { x: 260.0, y: 200.0 },
            Coord { x: 260.0, y: 150.0 },
            Coord { x: 180.0, y: 150.0 },
        ]),
        vec![LineString(vec![
            Coord { x: 215.0, y: 160.0 },
            Coord { x: 230.0, y: 190.0 },
            Coord { x: 200.0, y: 190.0 },
        ])],
    );

    let clip = Polygon::new(
        LineString(vec![
            Coord { x: 190.0, y: 210.0 },
            Coord { x: 240.0, y: 210.0 },
            Coord { x: 240.0, y: 130.0 },
            Coord { x: 190.0, y: 130.0 },
        ]),
        vec![],
    );

    let result = subject.intersection(&clip);
    println!("{:?}", result);

    let s2 = subject.clone().translate(-100., 0.);
    let svg_s2 = s2
        .to_svg()
        .with_stroke_width(1.)
        .with_stroke_color(Color::Rgb(200, 0, 100));

    let c2 = clip.clone().translate(0., -100.);
    let svg_c2 = c2
        .to_svg()
        .with_stroke_width(1.)
        .with_stroke_color(Color::Rgb(100, 0, 200));

    let r2 = result.clone();
    let svg_r2 = r2
        .to_svg()
        .with_stroke_width(1.)
        .with_stroke_color(Color::Rgb(100, 100, 0));
    fs::write("subj.svg", svg_r2.and(svg_s2).and(svg_c2).to_string())
        .expect("Unable to write file");
}

fn test_svg() {
    let point: Point = Point::new(10.0, 28.1);
    let line = Line::new(
        Coord {
            x: 114.19,
            y: 22.26,
        },
        Coord {
            x: 15.93,
            y: -15.76,
        },
    );

    let svg = point
        .to_svg()
        .with_radius(2.0)
        .and(line.to_svg().with_stroke_width(2.5))
        .with_fill_color(Color::Named("red"))
        .with_stroke_color(Color::Rgb(200, 0, 100))
        .with_fill_opacity(0.7);

    println!("{}", svg);
    fs::write("poly.svg", svg.to_string()).expect("Unable to write file");
}
