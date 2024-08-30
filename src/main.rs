mod eventail;
mod flower;
mod hexagon;
mod oglines;
mod operations;
mod svg;
mod triskell;

use hexagon::hexagon;

const DEBUG: bool = true;

fn main() {
    //    draw_test_flower("flower.svg");
    //    draw_test_eventail("eventail.svg");
    //    draw_test_triskell("trsiskel.svg");
    hexagon("plane.svg");
}
