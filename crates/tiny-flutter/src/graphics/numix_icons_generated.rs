// Generated Numix Circle derivatives. GPL-3.0-or-later; see third_party/numix-p4desk.
use super::{Color, VectorCommand, VectorIcon, VectorLayer, VectorPaint, VectorShape};
use crate::tiny_gfx::{FillRule, LineCap, LineJoin};
pub static NUMIX_DARK_CLOCK: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x244c43),
                bottom: Color::from_hex(0x305f52),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 66.434782,
                y: 66.434782,
                radius: 38.956522,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 38.956522,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(220, 233, 225, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_TIMER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x51333d),
                bottom: Color::from_hex(0x69414c),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 37.217391),
                VectorCommand::Cubic(
                    48.977391, 37.217391, 34.782609, 51.424348, 34.782609, 68.869565,
                ),
                VectorCommand::Cubic(
                    34.782609, 86.324522, 48.982261, 100.519304, 66.434783, 100.519304,
                ),
                VectorCommand::Cubic(
                    83.709565, 100.519304, 97.738783, 86.604522, 98.030957, 69.400348,
                ),
                VectorCommand::Cubic(
                    98.068377, 69.225121, 98.086341, 69.046299, 98.084522, 68.86713,
                ),
                VectorCommand::Cubic(
                    98.086883, 68.685523, 98.068916, 68.504221, 98.030957, 68.326609,
                ),
                VectorCommand::Cubic(
                    97.733913, 51.137043, 83.704696, 37.214957, 66.434783, 37.214957,
                ),
                VectorCommand::Move(66.434783, 42.084522),
                VectorCommand::Cubic(
                    81.257739, 42.084522, 93.217391, 54.053913, 93.217391, 68.86713,
                ),
                VectorCommand::Cubic(
                    93.217391, 83.692522, 81.262609, 95.649739, 66.434783, 95.649739,
                ),
                VectorCommand::Cubic(
                    51.609391, 95.649739, 39.652174, 83.694957, 39.652174, 68.86713,
                ),
                VectorCommand::Cubic(
                    39.652174, 54.051478, 51.611826, 42.084522, 66.434783, 42.084522,
                ),
                VectorCommand::Line(66.434783, 42.084522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.398261, 50.993391),
                VectorCommand::Cubic(65.054467, 51.013417, 63.981036, 52.118452, 64.0, 53.462261),
                VectorCommand::Line(64.0, 69.144696),
                VectorCommand::Line(77.834435, 83.198261),
                VectorCommand::Cubic(
                    78.441238, 83.834047, 79.343589, 84.093462, 80.195401, 83.877011,
                ),
                VectorCommand::Cubic(
                    81.047213, 83.66056, 81.71626, 83.001843, 81.945937, 82.153501,
                ),
                VectorCommand::Cubic(82.175614, 81.30516, 81.930269, 80.398882, 81.304, 79.782261),
                VectorCommand::Line(68.869565, 67.150609),
                VectorCommand::Line(68.869565, 53.462261),
                VectorCommand::Cubic(
                    68.878853, 52.804249, 68.621388, 52.170499, 68.155828, 51.705397,
                ),
                VectorCommand::Cubic(
                    67.690267, 51.240295, 67.056264, 50.983455, 66.398261, 50.993391,
                ),
                VectorCommand::Line(66.398261, 50.993391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.570435, 50.104696),
                VectorCommand::Line(47.833043, 46.832348),
                VectorCommand::Line(42.125913, 40.954783),
                VectorCommand::Line(38.853565, 44.387826),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(54.26087, 34.782609),
                VectorCommand::Cubic(
                    60.004522, 31.938783, 70.296348, 31.834087, 76.173913, 34.782609,
                ),
                VectorCommand::Cubic(
                    67.272348, 25.043478, 63.077217, 25.043478, 54.26087, 34.782609,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(63.998812, 34.782609),
                VectorCommand::Cubic(
                    46.541421, 34.782609, 32.346638, 48.989565, 32.346638, 66.434783,
                ),
                VectorCommand::Cubic(
                    32.346638, 83.889739, 46.54629, 98.084522, 63.998812, 98.084522,
                ),
                VectorCommand::Cubic(
                    81.273595, 98.084522, 95.302812, 84.169739, 95.594986, 66.965565,
                ),
                VectorCommand::Cubic(
                    95.632407, 66.790338, 95.65037, 66.611516, 95.648551, 66.432348,
                ),
                VectorCommand::Cubic(
                    95.650912, 66.25074, 95.632945, 66.069438, 95.594986, 65.891826,
                ),
                VectorCommand::Cubic(
                    95.297942, 48.702261, 81.268725, 34.780174, 63.998812, 34.780174,
                ),
                VectorCommand::Move(63.998812, 39.649739),
                VectorCommand::Cubic(
                    78.821769, 39.649739, 90.781421, 51.61913, 90.781421, 66.432348,
                ),
                VectorCommand::Cubic(
                    90.781421, 81.257739, 78.826638, 93.214957, 63.998812, 93.214957,
                ),
                VectorCommand::Cubic(
                    49.173421, 93.214957, 37.216203, 81.260174, 37.216203, 66.432348,
                ),
                VectorCommand::Cubic(
                    37.216203, 51.616696, 49.175855, 39.649739, 63.998812, 39.649739,
                ),
                VectorCommand::Line(63.998812, 39.649739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(245, 187, 198, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(63.96229, 48.558609),
                VectorCommand::Cubic(
                    62.618497, 48.578634, 61.545065, 49.683669, 61.564029, 51.027478,
                ),
                VectorCommand::Line(61.564029, 66.709913),
                VectorCommand::Line(75.398464, 80.763478),
                VectorCommand::Cubic(
                    76.005268, 81.399264, 76.907618, 81.65868, 77.75943, 81.442229,
                ),
                VectorCommand::Cubic(
                    78.611242, 81.225778, 79.280289, 80.56706, 79.509966, 79.718719,
                ),
                VectorCommand::Cubic(
                    79.739644, 78.870377, 79.494298, 77.9641, 78.868029, 77.347478,
                ),
                VectorCommand::Line(66.433595, 64.715826),
                VectorCommand::Line(66.433595, 51.027478),
                VectorCommand::Cubic(
                    66.442883, 50.369466, 66.185418, 49.735716, 65.719857, 49.270614,
                ),
                VectorCommand::Cubic(
                    65.254297, 48.805512, 64.620293, 48.548672, 63.96229, 48.558609,
                ),
                VectorCommand::Line(63.96229, 48.558609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(245, 187, 198, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.134464, 47.669913),
                VectorCommand::Line(45.397073, 44.397565),
                VectorCommand::Line(39.689942, 38.52),
                VectorCommand::Line(36.417595, 41.953043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(245, 187, 198, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.824899, 32.347826),
                VectorCommand::Cubic(
                    57.568551, 29.504, 67.860377, 29.399304, 73.737942, 32.347826,
                ),
                VectorCommand::Cubic(
                    64.836377, 22.608696, 60.641247, 22.608696, 51.824899, 32.347826,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(245, 187, 198, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_NOTES: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x4c432a),
                bottom: Color::from_hex(0x625537),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 37.217391),
                VectorCommand::Line(95.652174, 37.217391),
                VectorCommand::Line(95.652174, 95.652174),
                VectorCommand::Line(37.217391, 95.652174),
                VectorCommand::Line(37.217391, 37.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 34.782609),
                VectorCommand::Line(93.217391, 34.782609),
                VectorCommand::Line(93.217391, 93.217391),
                VectorCommand::Line(34.782609, 93.217391),
                VectorCommand::Line(34.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(237, 203, 128, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Rect {
                x: 34.782609,
                y: 34.782609,
                width: 58.434783,
                height: 9.73913,
                radius: 0.0,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.086957, 51.826087),
                VectorCommand::Line(42.086957, 56.695652),
                VectorCommand::Line(46.956522, 56.695652),
                VectorCommand::Line(46.956522, 51.826087),
                VectorCommand::Line(42.086957, 51.826087),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 51.826087),
                VectorCommand::Line(54.26087, 56.695652),
                VectorCommand::Line(85.913043, 56.695652),
                VectorCommand::Line(85.913043, 51.826087),
                VectorCommand::Line(54.26087, 51.826087),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 66.434783),
                VectorCommand::Line(42.086957, 71.304348),
                VectorCommand::Line(46.956522, 71.304348),
                VectorCommand::Line(46.956522, 66.434783),
                VectorCommand::Line(42.086957, 66.434783),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 66.434783),
                VectorCommand::Line(54.26087, 71.304348),
                VectorCommand::Line(85.913043, 71.304348),
                VectorCommand::Line(85.913043, 66.434783),
                VectorCommand::Line(54.26087, 66.434783),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 81.043478),
                VectorCommand::Line(42.086957, 85.913043),
                VectorCommand::Line(46.956522, 85.913043),
                VectorCommand::Line(46.956522, 81.043478),
                VectorCommand::Line(42.086957, 81.043478),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 81.043478),
                VectorCommand::Line(54.26087, 85.913043),
                VectorCommand::Line(85.913043, 85.913043),
                VectorCommand::Line(85.913043, 81.043478),
                VectorCommand::Line(54.26087, 81.043478),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(88, 67, 38, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_CALCULATOR: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(33.070957, 8.0, 8.0, 33.070957, 8.0, 64.0),
                VectorCommand::Line(81.043478, 81.043478),
                VectorCommand::Line(64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(40, 88, 106, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Line(90.782609, 71.304348),
                VectorCommand::Line(120.0, 64.0),
                VectorCommand::Cubic(120.0, 33.070957, 94.929043, 8.0, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(53, 107, 123, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 64.0),
                VectorCommand::Line(59.130435, 90.782609),
                VectorCommand::Line(64.0, 120.0),
                VectorCommand::Cubic(94.929043, 120.0, 120.0, 94.929043, 120.0, 64.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(119, 96, 57, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(8.0, 64.0),
                VectorCommand::Cubic(8.0, 94.929043, 33.070957, 120.0, 64.0, 120.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Line(8.0, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(32, 75, 91, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.086957, 34.782609),
                VectorCommand::Line(42.086957, 42.086957),
                VectorCommand::Line(34.782609, 42.086957),
                VectorCommand::Line(34.782609, 46.956522),
                VectorCommand::Line(42.086957, 46.956522),
                VectorCommand::Line(42.086957, 54.26087),
                VectorCommand::Line(46.956522, 54.26087),
                VectorCommand::Line(46.956522, 46.956522),
                VectorCommand::Line(54.26087, 46.956522),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Line(46.956522, 42.086957),
                VectorCommand::Line(46.956522, 34.782609),
                VectorCommand::Line(42.086957, 34.782609),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 42.086957),
                VectorCommand::Line(78.608696, 46.956522),
                VectorCommand::Line(98.086957, 46.956522),
                VectorCommand::Line(98.086957, 42.086957),
                VectorCommand::Line(78.608696, 42.086957),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 78.608696),
                VectorCommand::Line(42.086957, 83.478261),
                VectorCommand::Line(46.956522, 83.478261),
                VectorCommand::Line(46.956522, 78.608696),
                VectorCommand::Line(42.086957, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 81.043478),
                VectorCommand::Line(78.608696, 85.913043),
                VectorCommand::Line(98.086957, 85.913043),
                VectorCommand::Line(98.086957, 81.043478),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Close,
                VectorCommand::Move(34.782609, 85.913043),
                VectorCommand::Line(34.782609, 90.782609),
                VectorCommand::Line(54.26087, 90.782609),
                VectorCommand::Line(54.26087, 85.913043),
                VectorCommand::Line(34.782609, 85.913043),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 90.782609),
                VectorCommand::Line(78.608696, 95.652174),
                VectorCommand::Line(98.086957, 95.652174),
                VectorCommand::Line(98.086957, 90.782609),
                VectorCommand::Line(78.608696, 90.782609),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 93.217391),
                VectorCommand::Line(42.086957, 98.086957),
                VectorCommand::Line(46.956522, 98.086957),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Line(42.086957, 93.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Line(39.652174, 39.652174),
                VectorCommand::Line(32.347826, 39.652174),
                VectorCommand::Line(32.347826, 44.521739),
                VectorCommand::Line(39.652174, 44.521739),
                VectorCommand::Line(39.652174, 51.826087),
                VectorCommand::Line(44.521739, 51.826087),
                VectorCommand::Line(44.521739, 44.521739),
                VectorCommand::Line(51.826087, 44.521739),
                VectorCommand::Line(51.826087, 39.652174),
                VectorCommand::Line(44.521739, 39.652174),
                VectorCommand::Line(44.521739, 32.347826),
                VectorCommand::Line(39.652174, 32.347826),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 39.652174),
                VectorCommand::Line(76.173913, 44.521739),
                VectorCommand::Line(95.652174, 44.521739),
                VectorCommand::Line(95.652174, 39.652174),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 76.173913),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Line(44.521739, 81.043478),
                VectorCommand::Line(44.521739, 76.173913),
                VectorCommand::Line(39.652174, 76.173913),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 78.608696),
                VectorCommand::Line(76.173913, 83.478261),
                VectorCommand::Line(95.652174, 83.478261),
                VectorCommand::Line(95.652174, 78.608696),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 83.478261),
                VectorCommand::Line(32.347826, 88.347826),
                VectorCommand::Line(51.826087, 88.347826),
                VectorCommand::Line(51.826087, 83.478261),
                VectorCommand::Line(32.347826, 83.478261),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 88.347826),
                VectorCommand::Line(76.173913, 93.217391),
                VectorCommand::Line(95.652174, 93.217391),
                VectorCommand::Line(95.652174, 88.347826),
                VectorCommand::Line(76.173913, 88.347826),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 90.782609),
                VectorCommand::Line(39.652174, 95.652174),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(44.521739, 90.782609),
                VectorCommand::Line(39.652174, 90.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(244, 239, 225, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_MAC: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x303e54),
                bottom: Color::from_hex(0x41536c),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 42.086957),
                VectorCommand::Line(32.347826, 49.391304),
                VectorCommand::Cubic(
                    32.347826, 50.742609, 33.441043, 51.826087, 34.782609, 51.826087,
                ),
                VectorCommand::Line(39.652174, 51.826087),
                VectorCommand::Cubic(
                    40.993739, 51.826087, 42.086957, 50.742609, 42.086957, 49.391304,
                ),
                VectorCommand::Line(42.086957, 44.521739),
                VectorCommand::Cubic(
                    42.086957, 43.180174, 40.993739, 42.086957, 39.652174, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 42.086957),
                VectorCommand::Line(46.956522, 49.391304),
                VectorCommand::Cubic(
                    46.956522, 50.742609, 48.049739, 51.826087, 49.391304, 51.826087,
                ),
                VectorCommand::Line(54.26087, 51.826087),
                VectorCommand::Cubic(
                    55.602435, 51.826087, 56.695652, 50.742609, 56.695652, 49.391304,
                ),
                VectorCommand::Line(56.695652, 44.521739),
                VectorCommand::Cubic(
                    56.695652, 43.180174, 55.602435, 42.086957, 54.26087, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 42.086957),
                VectorCommand::Line(61.565217, 49.391304),
                VectorCommand::Cubic(61.565217, 50.742609, 62.658435, 51.826087, 64.0, 51.826087),
                VectorCommand::Line(68.869565, 51.826087),
                VectorCommand::Cubic(
                    70.21113, 51.826087, 71.304348, 50.742609, 71.304348, 49.391304,
                ),
                VectorCommand::Line(71.304348, 44.521739),
                VectorCommand::Cubic(
                    71.304348, 43.180174, 70.21113, 42.086957, 68.869565, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 42.086957),
                VectorCommand::Line(76.173913, 49.391304),
                VectorCommand::Cubic(
                    76.173913, 50.742609, 77.26713, 51.826087, 78.608696, 51.826087,
                ),
                VectorCommand::Line(83.478261, 51.826087),
                VectorCommand::Cubic(
                    84.819826, 51.826087, 85.913043, 50.742609, 85.913043, 49.391304,
                ),
                VectorCommand::Line(85.913043, 44.521739),
                VectorCommand::Cubic(
                    85.913043, 43.180174, 84.819826, 42.086957, 83.478261, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 42.086957),
                VectorCommand::Line(90.782609, 49.391304),
                VectorCommand::Cubic(
                    90.782609, 50.742609, 91.875826, 51.826087, 93.217391, 51.826087,
                ),
                VectorCommand::Line(98.086957, 51.826087),
                VectorCommand::Cubic(
                    99.428522, 51.826087, 100.521739, 50.742609, 100.521739, 49.391304,
                ),
                VectorCommand::Line(100.521739, 44.521739),
                VectorCommand::Cubic(
                    100.521739, 43.180174, 99.428522, 42.086957, 98.086957, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 56.695652),
                VectorCommand::Line(32.347826, 64.0),
                VectorCommand::Cubic(
                    32.347826, 65.351304, 33.441043, 66.434783, 34.782609, 66.434783,
                ),
                VectorCommand::Line(39.652174, 66.434783),
                VectorCommand::Cubic(40.993739, 66.434783, 42.086957, 65.351304, 42.086957, 64.0),
                VectorCommand::Line(42.086957, 59.130435),
                VectorCommand::Cubic(
                    42.086957, 57.78887, 40.993739, 56.695652, 39.652174, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 56.695652),
                VectorCommand::Line(46.956522, 64.0),
                VectorCommand::Cubic(
                    46.956522, 65.351304, 48.049739, 66.434783, 49.391304, 66.434783,
                ),
                VectorCommand::Line(54.26087, 66.434783),
                VectorCommand::Cubic(55.602435, 66.434783, 56.695652, 65.351304, 56.695652, 64.0),
                VectorCommand::Line(56.695652, 59.130435),
                VectorCommand::Cubic(
                    56.695652, 57.78887, 55.602435, 56.695652, 54.26087, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 56.695652),
                VectorCommand::Line(61.565217, 64.0),
                VectorCommand::Cubic(61.565217, 65.351304, 62.658435, 66.434783, 64.0, 66.434783),
                VectorCommand::Line(68.869565, 66.434783),
                VectorCommand::Cubic(70.21113, 66.434783, 71.304348, 65.351304, 71.304348, 64.0),
                VectorCommand::Line(71.304348, 59.130435),
                VectorCommand::Cubic(
                    71.304348, 57.78887, 70.21113, 56.695652, 68.869565, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 56.695652),
                VectorCommand::Line(76.173913, 64.0),
                VectorCommand::Cubic(
                    76.173913, 65.351304, 77.26713, 66.434783, 78.608696, 66.434783,
                ),
                VectorCommand::Line(83.478261, 66.434783),
                VectorCommand::Cubic(84.819826, 66.434783, 85.913043, 65.351304, 85.913043, 64.0),
                VectorCommand::Line(85.913043, 59.130435),
                VectorCommand::Cubic(
                    85.913043, 57.78887, 84.819826, 56.695652, 83.478261, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 56.695652),
                VectorCommand::Line(90.782609, 64.0),
                VectorCommand::Cubic(
                    90.782609, 65.351304, 91.875826, 66.434783, 93.217391, 66.434783,
                ),
                VectorCommand::Line(98.086957, 66.434783),
                VectorCommand::Cubic(
                    99.428522, 66.434783, 100.521739, 65.351304, 100.521739, 64.0,
                ),
                VectorCommand::Line(100.521739, 59.130435),
                VectorCommand::Cubic(
                    100.521739, 57.78887, 99.428522, 56.695652, 98.086957, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 71.304348),
                VectorCommand::Line(32.347826, 78.608696),
                VectorCommand::Cubic(32.347826, 79.96, 33.441043, 81.043478, 34.782609, 81.043478),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Cubic(40.993739, 81.043478, 42.086957, 79.96, 42.086957, 78.608696),
                VectorCommand::Line(42.086957, 73.73913),
                VectorCommand::Cubic(
                    42.086957, 72.397565, 40.993739, 71.304348, 39.652174, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 71.304348),
                VectorCommand::Line(46.956522, 78.608696),
                VectorCommand::Cubic(46.956522, 79.96, 48.049739, 81.043478, 49.391304, 81.043478),
                VectorCommand::Line(54.26087, 81.043478),
                VectorCommand::Cubic(55.602435, 81.043478, 56.695652, 79.96, 56.695652, 78.608696),
                VectorCommand::Line(56.695652, 73.73913),
                VectorCommand::Cubic(
                    56.695652, 72.397565, 55.602435, 71.304348, 54.26087, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 71.304348),
                VectorCommand::Line(61.565217, 78.608696),
                VectorCommand::Cubic(61.565217, 79.96, 62.658435, 81.043478, 64.0, 81.043478),
                VectorCommand::Line(68.869565, 81.043478),
                VectorCommand::Cubic(70.21113, 81.043478, 71.304348, 79.96, 71.304348, 78.608696),
                VectorCommand::Line(71.304348, 73.73913),
                VectorCommand::Cubic(
                    71.304348, 72.397565, 70.21113, 71.304348, 68.869565, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 71.304348),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Cubic(76.173913, 79.96, 77.26713, 81.043478, 78.608696, 81.043478),
                VectorCommand::Line(83.478261, 81.043478),
                VectorCommand::Cubic(84.819826, 81.043478, 85.913043, 79.96, 85.913043, 78.608696),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Cubic(
                    85.913043, 72.397565, 84.819826, 71.304348, 83.478261, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 71.304348),
                VectorCommand::Line(90.782609, 78.608696),
                VectorCommand::Cubic(90.782609, 79.96, 91.875826, 81.043478, 93.217391, 81.043478),
                VectorCommand::Line(98.086957, 81.043478),
                VectorCommand::Cubic(
                    99.428522, 81.043478, 100.521739, 79.96, 100.521739, 78.608696,
                ),
                VectorCommand::Line(100.521739, 73.73913),
                VectorCommand::Cubic(
                    100.521739, 72.397565, 99.428522, 71.304348, 98.086957, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 85.913043),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Cubic(
                    46.956522, 94.568696, 48.049739, 95.652174, 49.391304, 95.652174,
                ),
                VectorCommand::Line(83.478261, 95.661694),
                VectorCommand::Cubic(
                    84.819826, 95.662069, 85.913043, 94.578216, 85.913043, 93.226911,
                ),
                VectorCommand::Line(85.913043, 88.357346),
                VectorCommand::Cubic(
                    85.913043, 87.015781, 84.819826, 85.922563, 83.478261, 85.922563,
                ),
                VectorCommand::Line(83.478261, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 39.652174),
                VectorCommand::Cubic(
                    31.006261, 39.652174, 29.913043, 40.745391, 29.913043, 42.086957,
                ),
                VectorCommand::Line(29.913043, 46.956522),
                VectorCommand::Cubic(
                    29.913043, 48.307826, 31.006261, 49.391304, 32.347826, 49.391304,
                ),
                VectorCommand::Line(37.217391, 49.391304),
                VectorCommand::Cubic(
                    38.558957, 49.391304, 39.652174, 48.307826, 39.652174, 46.956522,
                ),
                VectorCommand::Line(39.652174, 42.086957),
                VectorCommand::Cubic(
                    39.652174, 40.745391, 38.558957, 39.652174, 37.217391, 39.652174,
                ),
                VectorCommand::Line(32.347826, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 39.652174),
                VectorCommand::Cubic(
                    45.614957, 39.652174, 44.521739, 40.745391, 44.521739, 42.086957,
                ),
                VectorCommand::Line(44.521739, 46.956522),
                VectorCommand::Cubic(
                    44.521739, 48.307826, 45.614957, 49.391304, 46.956522, 49.391304,
                ),
                VectorCommand::Line(51.826087, 49.391304),
                VectorCommand::Cubic(
                    53.167652, 49.391304, 54.26087, 48.307826, 54.26087, 46.956522,
                ),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Cubic(
                    54.26087, 40.745391, 53.167652, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Line(46.956522, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 39.652174),
                VectorCommand::Cubic(
                    60.223652, 39.652174, 59.130435, 40.745391, 59.130435, 42.086957,
                ),
                VectorCommand::Line(59.130435, 46.956522),
                VectorCommand::Cubic(
                    59.130435, 48.307826, 60.223652, 49.391304, 61.565217, 49.391304,
                ),
                VectorCommand::Line(66.434783, 49.391304),
                VectorCommand::Cubic(
                    67.776348, 49.391304, 68.869565, 48.307826, 68.869565, 46.956522,
                ),
                VectorCommand::Line(68.869565, 42.086957),
                VectorCommand::Cubic(
                    68.869565, 40.745391, 67.776348, 39.652174, 66.434783, 39.652174,
                ),
                VectorCommand::Line(61.565217, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 39.652174),
                VectorCommand::Cubic(
                    74.832348, 39.652174, 73.73913, 40.745391, 73.73913, 42.086957,
                ),
                VectorCommand::Line(73.73913, 46.956522),
                VectorCommand::Cubic(
                    73.73913, 48.307826, 74.832348, 49.391304, 76.173913, 49.391304,
                ),
                VectorCommand::Line(81.043478, 49.391304),
                VectorCommand::Cubic(
                    82.385043, 49.391304, 83.478261, 48.307826, 83.478261, 46.956522,
                ),
                VectorCommand::Line(83.478261, 42.086957),
                VectorCommand::Cubic(
                    83.478261, 40.745391, 82.385043, 39.652174, 81.043478, 39.652174,
                ),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 39.652174),
                VectorCommand::Cubic(
                    89.441043, 39.652174, 88.347826, 40.745391, 88.347826, 42.086957,
                ),
                VectorCommand::Line(88.347826, 46.956522),
                VectorCommand::Cubic(
                    88.347826, 48.307826, 89.441043, 49.391304, 90.782609, 49.391304,
                ),
                VectorCommand::Line(95.652174, 49.391304),
                VectorCommand::Cubic(
                    96.993739, 49.391304, 98.086957, 48.307826, 98.086957, 46.956522,
                ),
                VectorCommand::Line(98.086957, 42.086957),
                VectorCommand::Cubic(
                    98.086957, 40.745391, 96.993739, 39.652174, 95.652174, 39.652174,
                ),
                VectorCommand::Line(90.782609, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 54.26087),
                VectorCommand::Cubic(
                    31.006261, 54.26087, 29.913043, 55.354087, 29.913043, 56.695652,
                ),
                VectorCommand::Line(29.913043, 61.565217),
                VectorCommand::Cubic(29.913043, 62.916522, 31.006261, 64.0, 32.347826, 64.0),
                VectorCommand::Line(37.217391, 64.0),
                VectorCommand::Cubic(38.558957, 64.0, 39.652174, 62.916522, 39.652174, 61.565217),
                VectorCommand::Line(39.652174, 56.695652),
                VectorCommand::Cubic(
                    39.652174, 55.354087, 38.558957, 54.26087, 37.217391, 54.26087,
                ),
                VectorCommand::Line(32.347826, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 54.26087),
                VectorCommand::Cubic(
                    45.614957, 54.26087, 44.521739, 55.354087, 44.521739, 56.695652,
                ),
                VectorCommand::Line(44.521739, 61.565217),
                VectorCommand::Cubic(44.521739, 62.916522, 45.614957, 64.0, 46.956522, 64.0),
                VectorCommand::Line(51.826087, 64.0),
                VectorCommand::Cubic(53.167652, 64.0, 54.26087, 62.916522, 54.26087, 61.565217),
                VectorCommand::Line(54.26087, 56.695652),
                VectorCommand::Cubic(
                    54.26087, 55.354087, 53.167652, 54.26087, 51.826087, 54.26087,
                ),
                VectorCommand::Line(46.956522, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 54.26087),
                VectorCommand::Cubic(
                    60.223652, 54.26087, 59.130435, 55.354087, 59.130435, 56.695652,
                ),
                VectorCommand::Line(59.130435, 61.565217),
                VectorCommand::Cubic(59.130435, 62.916522, 60.223652, 64.0, 61.565217, 64.0),
                VectorCommand::Line(66.434783, 64.0),
                VectorCommand::Cubic(67.776348, 64.0, 68.869565, 62.916522, 68.869565, 61.565217),
                VectorCommand::Line(68.869565, 56.695652),
                VectorCommand::Cubic(
                    68.869565, 55.354087, 67.776348, 54.26087, 66.434783, 54.26087,
                ),
                VectorCommand::Line(61.565217, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 54.26087),
                VectorCommand::Cubic(
                    74.832348, 54.26087, 73.73913, 55.354087, 73.73913, 56.695652,
                ),
                VectorCommand::Line(73.73913, 61.565217),
                VectorCommand::Cubic(73.73913, 62.916522, 74.832348, 64.0, 76.173913, 64.0),
                VectorCommand::Line(81.043478, 64.0),
                VectorCommand::Cubic(82.385043, 64.0, 83.478261, 62.916522, 83.478261, 61.565217),
                VectorCommand::Line(83.478261, 56.695652),
                VectorCommand::Cubic(
                    83.478261, 55.354087, 82.385043, 54.26087, 81.043478, 54.26087,
                ),
                VectorCommand::Line(76.173913, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 54.26087),
                VectorCommand::Cubic(
                    89.441043, 54.26087, 88.347826, 55.354087, 88.347826, 56.695652,
                ),
                VectorCommand::Line(88.347826, 61.565217),
                VectorCommand::Cubic(88.347826, 62.916522, 89.441043, 64.0, 90.782609, 64.0),
                VectorCommand::Line(95.652174, 64.0),
                VectorCommand::Cubic(96.993739, 64.0, 98.086957, 62.916522, 98.086957, 61.565217),
                VectorCommand::Line(98.086957, 56.695652),
                VectorCommand::Cubic(
                    98.086957, 55.354087, 96.993739, 54.26087, 95.652174, 54.26087,
                ),
                VectorCommand::Line(90.782609, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 68.869565),
                VectorCommand::Cubic(
                    31.006261, 68.869565, 29.913043, 69.962783, 29.913043, 71.304348,
                ),
                VectorCommand::Line(29.913043, 76.173913),
                VectorCommand::Cubic(
                    29.913043, 77.525217, 31.006261, 78.608696, 32.347826, 78.608696,
                ),
                VectorCommand::Line(37.217391, 78.608696),
                VectorCommand::Cubic(
                    38.558957, 78.608696, 39.652174, 77.525217, 39.652174, 76.173913,
                ),
                VectorCommand::Line(39.652174, 71.304348),
                VectorCommand::Cubic(
                    39.652174, 69.962783, 38.558957, 68.869565, 37.217391, 68.869565,
                ),
                VectorCommand::Line(32.347826, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 68.869565),
                VectorCommand::Cubic(
                    45.614957, 68.869565, 44.521739, 69.962783, 44.521739, 71.304348,
                ),
                VectorCommand::Line(44.521739, 76.173913),
                VectorCommand::Cubic(
                    44.521739, 77.525217, 45.614957, 78.608696, 46.956522, 78.608696,
                ),
                VectorCommand::Line(51.826087, 78.608696),
                VectorCommand::Cubic(
                    53.167652, 78.608696, 54.26087, 77.525217, 54.26087, 76.173913,
                ),
                VectorCommand::Line(54.26087, 71.304348),
                VectorCommand::Cubic(
                    54.26087, 69.962783, 53.167652, 68.869565, 51.826087, 68.869565,
                ),
                VectorCommand::Line(46.956522, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 68.869565),
                VectorCommand::Cubic(
                    60.223652, 68.869565, 59.130435, 69.962783, 59.130435, 71.304348,
                ),
                VectorCommand::Line(59.130435, 76.173913),
                VectorCommand::Cubic(
                    59.130435, 77.525217, 60.223652, 78.608696, 61.565217, 78.608696,
                ),
                VectorCommand::Line(66.434783, 78.608696),
                VectorCommand::Cubic(
                    67.776348, 78.608696, 68.869565, 77.525217, 68.869565, 76.173913,
                ),
                VectorCommand::Line(68.869565, 71.304348),
                VectorCommand::Cubic(
                    68.869565, 69.962783, 67.776348, 68.869565, 66.434783, 68.869565,
                ),
                VectorCommand::Line(61.565217, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 68.869565),
                VectorCommand::Cubic(
                    74.832348, 68.869565, 73.73913, 69.962783, 73.73913, 71.304348,
                ),
                VectorCommand::Line(73.73913, 76.173913),
                VectorCommand::Cubic(
                    73.73913, 77.525217, 74.832348, 78.608696, 76.173913, 78.608696,
                ),
                VectorCommand::Line(81.043478, 78.608696),
                VectorCommand::Cubic(
                    82.385043, 78.608696, 83.478261, 77.525217, 83.478261, 76.173913,
                ),
                VectorCommand::Line(83.478261, 71.304348),
                VectorCommand::Cubic(
                    83.478261, 69.962783, 82.385043, 68.869565, 81.043478, 68.869565,
                ),
                VectorCommand::Line(76.173913, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 68.869565),
                VectorCommand::Cubic(
                    89.441043, 68.869565, 88.347826, 69.962783, 88.347826, 71.304348,
                ),
                VectorCommand::Line(88.347826, 76.173913),
                VectorCommand::Cubic(
                    88.347826, 77.525217, 89.441043, 78.608696, 90.782609, 78.608696,
                ),
                VectorCommand::Line(95.652174, 78.608696),
                VectorCommand::Cubic(
                    96.993739, 78.608696, 98.086957, 77.525217, 98.086957, 76.173913,
                ),
                VectorCommand::Line(98.086957, 71.304348),
                VectorCommand::Cubic(
                    98.086957, 69.962783, 96.993739, 68.869565, 95.652174, 68.869565,
                ),
                VectorCommand::Line(90.782609, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 83.478261),
                VectorCommand::Cubic(
                    45.614957, 83.477886, 44.521739, 84.571478, 44.521739, 85.913043,
                ),
                VectorCommand::Line(44.521739, 90.782609),
                VectorCommand::Cubic(
                    44.521739, 92.133913, 45.614957, 93.217391, 46.956522, 93.217391,
                ),
                VectorCommand::Line(81.043478, 93.226887),
                VectorCommand::Cubic(
                    82.385043, 93.227262, 83.478261, 92.143409, 83.478261, 90.792104,
                ),
                VectorCommand::Line(83.478261, 85.922539),
                VectorCommand::Cubic(
                    83.478261, 84.580974, 82.385043, 83.487757, 81.043478, 83.487757,
                ),
                VectorCommand::Line(46.956522, 83.478261),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(213, 229, 251, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_SETTINGS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x34443d),
                bottom: Color::from_hex(0x42564a),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 46.956522),
                VectorCommand::Cubic(
                    34.782609, 55.015652, 41.332174, 61.565217, 49.391304, 61.565217,
                ),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Cubic(
                    91.537391, 61.565217, 98.086957, 55.015652, 98.086957, 46.956522,
                ),
                VectorCommand::Cubic(
                    98.086957, 38.897391, 91.537391, 32.347826, 83.478261, 32.347826,
                ),
                VectorCommand::Line(34.782609, 46.956522),
                VectorCommand::Close,
                VectorCommand::Move(34.782609, 85.913043),
                VectorCommand::Cubic(
                    34.782609, 93.972174, 41.332174, 100.521739, 49.391304, 100.521739,
                ),
                VectorCommand::Line(83.478261, 100.521739),
                VectorCommand::Cubic(
                    91.537391, 100.521739, 98.086957, 93.972174, 98.086957, 85.913043,
                ),
                VectorCommand::Cubic(
                    98.086957, 77.853913, 91.537391, 71.304348, 83.478261, 71.304348,
                ),
                VectorCommand::Line(34.782609, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 29.913043),
                VectorCommand::Cubic(
                    38.897391, 29.913043, 32.347826, 36.462609, 32.347826, 44.521739,
                ),
                VectorCommand::Cubic(
                    32.347826, 52.58087, 38.897391, 59.130435, 46.956522, 59.130435,
                ),
                VectorCommand::Line(81.043478, 59.130435),
                VectorCommand::Cubic(
                    89.102609, 59.130435, 95.652174, 52.58087, 95.652174, 44.521739,
                ),
                VectorCommand::Cubic(
                    95.652174, 36.462609, 89.102609, 29.913043, 81.043478, 29.913043,
                ),
                VectorCommand::Line(46.956522, 29.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(78, 172, 124, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 68.869565),
                VectorCommand::Cubic(
                    38.897391, 68.869565, 32.347826, 75.41913, 32.347826, 83.478261,
                ),
                VectorCommand::Cubic(
                    32.347826, 91.537391, 38.897391, 98.086957, 46.956522, 98.086957,
                ),
                VectorCommand::Line(81.043478, 98.086957),
                VectorCommand::Cubic(
                    89.102609, 98.086957, 95.652174, 91.537391, 95.652174, 83.478261,
                ),
                VectorCommand::Cubic(
                    95.652174, 75.41913, 89.102609, 68.869565, 81.043478, 68.869565,
                ),
                VectorCommand::Line(46.956522, 68.869565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(199, 120, 114, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.043478, 34.782609),
                VectorCommand::Cubic(
                    75.662609, 34.782609, 71.304348, 39.14087, 71.304348, 44.521739,
                ),
                VectorCommand::Cubic(
                    71.304348, 49.902609, 75.662609, 54.26087, 81.043478, 54.26087,
                ),
                VectorCommand::Cubic(
                    86.424348, 54.26087, 90.782609, 49.902609, 90.782609, 44.521739,
                ),
                VectorCommand::Cubic(
                    90.782609, 39.14087, 86.424348, 34.782609, 81.043478, 34.782609,
                ),
                VectorCommand::Close,
                VectorCommand::Move(51.826087, 39.652174),
                VectorCommand::Line(45.73913, 45.73913),
                VectorCommand::Line(42.086957, 42.086957),
                VectorCommand::Line(39.652174, 44.521739),
                VectorCommand::Line(45.73913, 50.608696),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Line(51.826087, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 73.73913),
                VectorCommand::Cubic(
                    41.575652, 73.73913, 37.217391, 78.097391, 37.217391, 83.478261,
                ),
                VectorCommand::Cubic(
                    37.217391, 88.85913, 41.575652, 93.217391, 46.956522, 93.217391,
                ),
                VectorCommand::Cubic(
                    52.337391, 93.217391, 56.695652, 88.85913, 56.695652, 83.478261,
                ),
                VectorCommand::Cubic(
                    56.695652, 78.097391, 52.337391, 73.73913, 46.956522, 73.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(77.391304, 77.391304),
                VectorCommand::Line(74.956522, 79.826087),
                VectorCommand::Line(78.608696, 83.478261),
                VectorCommand::Line(74.956522, 87.130435),
                VectorCommand::Line(77.391304, 89.565217),
                VectorCommand::Line(81.043478, 85.913043),
                VectorCommand::Line(84.695652, 89.565217),
                VectorCommand::Line(87.130435, 87.130435),
                VectorCommand::Line(83.478261, 83.478261),
                VectorCommand::Line(87.130435, 79.826087),
                VectorCommand::Line(84.695652, 77.391304),
                VectorCommand::Line(81.043478, 81.043478),
                VectorCommand::Line(77.391304, 77.391304),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(246, 250, 243, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_DISPLAY: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x2f4057),
                bottom: Color::from_hex(0x405572),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.0, 39.652174),
                VectorCommand::Line(34.70713, 41.098435),
                VectorCommand::Line(34.70713, 67.394087),
                VectorCommand::Cubic(
                    34.70713, 68.163478, 35.381565, 68.840348, 36.153391, 68.840348,
                ),
                VectorCommand::Line(49.325565, 68.840348),
                VectorCommand::Line(49.325565, 73.709913),
                VectorCommand::Line(39.586435, 73.709913),
                VectorCommand::Line(39.586435, 78.579478),
                VectorCommand::Line(56.629913, 78.579478),
                VectorCommand::Line(56.629913, 63.970783),
                VectorCommand::Line(40.657739, 63.970783),
                VectorCommand::Cubic(40.144, 63.970783, 39.659478, 63.496, 39.659478, 62.972522),
                VectorCommand::Line(39.659478, 45.490783),
                VectorCommand::Cubic(
                    39.649739, 45.033043, 39.97113, 44.587478, 40.41913, 44.502261,
                ),
                VectorCommand::Cubic(
                    40.47513, 44.492522, 40.579339, 44.492522, 40.647513, 44.502261,
                ),
                VectorCommand::Line(67.6736, 44.502261),
                VectorCommand::Cubic(
                    68.1776, 44.502261, 68.662122, 44.967304, 68.662122, 45.490783,
                ),
                VectorCommand::Line(68.662122, 51.79687),
                VectorCommand::Line(73.60473, 51.79687),
                VectorCommand::Line(73.60473, 41.059478),
                VectorCommand::Cubic(
                    73.60473, 40.297391, 72.920557, 39.613217, 72.15847, 39.613217,
                ),
                VectorCommand::Line(35.880209, 39.613217),
                VectorCommand::Line(36.0, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(60.347826, 54.26087),
                VectorCommand::Line(59.035478, 55.71687),
                VectorCommand::Line(59.035478, 82.012522),
                VectorCommand::Cubic(
                    59.035478, 82.774609, 59.73913, 83.473391, 60.501217, 83.473391,
                ),
                VectorCommand::Line(73.649043, 83.473391),
                VectorCommand::Line(73.649043, 88.318609),
                VectorCommand::Line(63.909913, 88.318609),
                VectorCommand::Line(63.909913, 93.188174),
                VectorCommand::Line(93.127304, 93.188174),
                VectorCommand::Line(93.127304, 88.318609),
                VectorCommand::Line(83.388174, 88.318609),
                VectorCommand::Line(83.388174, 83.473391),
                VectorCommand::Line(96.536, 83.473391),
                VectorCommand::Cubic(97.298087, 83.473391, 97.992, 82.774609, 97.992, 82.012522),
                VectorCommand::Line(97.992, 55.71687),
                VectorCommand::Cubic(97.992, 54.947478, 97.288348, 54.26087, 96.536, 54.26087),
                VectorCommand::Line(60.257739, 54.26087),
                VectorCommand::Line(60.347826, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(64.754783, 59.130435),
                VectorCommand::Cubic(
                    64.810783, 59.120696, 64.869704, 59.120696, 64.935443, 59.130435,
                ),
                VectorCommand::Line(91.96153, 59.130435),
                VectorCommand::Cubic(
                    92.47527, 59.130435, 92.940313, 59.595478, 92.940313, 60.109217,
                ),
                VectorCommand::Line(92.940313, 77.639652),
                VectorCommand::Cubic(
                    92.940313, 78.143652, 92.47527, 78.608696, 91.96153, 78.608696,
                ),
                VectorCommand::Line(64.935443, 78.608696),
                VectorCommand::Cubic(
                    64.421704, 78.608696, 63.956661, 78.143652, 63.956661, 77.639652,
                ),
                VectorCommand::Line(63.956661, 60.109217),
                VectorCommand::Cubic(
                    63.946922, 59.651478, 64.309704, 59.215652, 64.75527, 59.130435,
                ),
                VectorCommand::Line(64.754783, 59.130435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(33.565217, 37.217391),
                VectorCommand::Cubic(
                    32.842087, 37.29287, 32.272348, 37.940522, 32.272348, 38.663652,
                ),
                VectorCommand::Line(32.272348, 64.959304),
                VectorCommand::Cubic(
                    32.272348, 65.728696, 32.946783, 66.405565, 33.718609, 66.405565,
                ),
                VectorCommand::Line(46.890783, 66.405565),
                VectorCommand::Line(46.890783, 71.27513),
                VectorCommand::Line(37.151652, 71.27513),
                VectorCommand::Line(37.151652, 76.144696),
                VectorCommand::Line(54.19513, 76.144696),
                VectorCommand::Line(54.19513, 61.536),
                VectorCommand::Line(38.222957, 61.536),
                VectorCommand::Cubic(
                    37.709217, 61.536, 37.224696, 61.061217, 37.224696, 60.537739,
                ),
                VectorCommand::Line(37.224696, 43.056),
                VectorCommand::Cubic(
                    37.214957, 42.598261, 37.538783, 42.152696, 37.986783, 42.067478,
                ),
                VectorCommand::Cubic(
                    38.042783, 42.057739, 38.147478, 42.057739, 38.215652, 42.067478,
                ),
                VectorCommand::Line(65.241739, 42.067478),
                VectorCommand::Cubic(
                    65.745739, 42.067478, 66.230261, 42.532522, 66.230261, 43.056,
                ),
                VectorCommand::Line(66.230261, 49.362087),
                VectorCommand::Line(71.17287, 49.362087),
                VectorCommand::Line(71.17287, 38.624696),
                VectorCommand::Cubic(
                    71.17287, 37.862609, 70.488696, 37.178435, 69.726609, 37.178435,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(227, 149, 170, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(57.913043, 51.826087),
                VectorCommand::Cubic(
                    57.199652, 51.901565, 56.600696, 52.568696, 56.600696, 53.282087,
                ),
                VectorCommand::Line(56.600696, 79.577739),
                VectorCommand::Cubic(
                    56.600696, 80.339826, 57.304348, 81.033739, 58.066435, 81.033739,
                ),
                VectorCommand::Line(71.214261, 81.033739),
                VectorCommand::Line(71.214261, 88.313739),
                VectorCommand::Line(80.953391, 88.313739),
                VectorCommand::Line(80.953391, 81.033739),
                VectorCommand::Line(94.101217, 81.033739),
                VectorCommand::Cubic(
                    94.863304, 81.033739, 95.557217, 80.339826, 95.557217, 79.577739,
                ),
                VectorCommand::Line(95.557217, 53.282087),
                VectorCommand::Cubic(
                    95.557217, 52.512696, 94.853565, 51.826087, 94.101217, 51.826087,
                ),
                VectorCommand::Move(62.449043, 56.695652),
                VectorCommand::Cubic(
                    62.505043, 56.685913, 62.563478, 56.685913, 62.629217, 56.695652,
                ),
                VectorCommand::Line(89.655304, 56.695652),
                VectorCommand::Cubic(
                    90.169043, 56.695652, 90.634087, 57.160696, 90.634087, 57.674435,
                ),
                VectorCommand::Line(90.634087, 75.20487),
                VectorCommand::Cubic(
                    90.634087, 75.70887, 90.169043, 76.173913, 89.655304, 76.173913,
                ),
                VectorCommand::Line(62.629217, 76.173913),
                VectorCommand::Cubic(
                    62.115478, 76.173913, 61.650435, 75.70887, 61.650435, 75.20487,
                ),
                VectorCommand::Line(61.650435, 57.674435),
                VectorCommand::Cubic(
                    61.640696, 57.216696, 62.003478, 56.78087, 62.449043, 56.695652,
                ),
                VectorCommand::Move(61.592, 85.913043),
                VectorCommand::Line(61.592, 90.782609),
                VectorCommand::Line(90.809391, 90.782609),
                VectorCommand::Line(90.809391, 85.913043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(175, 204, 233, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_SCREEN: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0x6f3d45),
                bottom: Color::from_hex(0x552d36),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 29.913043),
                VectorCommand::Cubic(
                    63.732174, 29.913043, 61.565217, 32.08487, 61.565217, 34.782609,
                ),
                VectorCommand::Line(61.565217, 61.565217),
                VectorCommand::Cubic(
                    61.565217, 64.267826, 63.737043, 66.434783, 66.434783, 66.434783,
                ),
                VectorCommand::Cubic(
                    69.132522, 66.434783, 71.304348, 64.262957, 71.304348, 61.565217,
                ),
                VectorCommand::Line(71.304348, 34.782609),
                VectorCommand::Cubic(71.304348, 32.08, 69.132522, 29.913043, 66.434783, 29.913043),
                VectorCommand::Close,
                VectorCommand::Move(51.387826, 33.516522),
                VectorCommand::Cubic(
                    50.749995, 33.57248, 50.12946, 33.753676, 49.561739, 34.049739,
                ),
                VectorCommand::Cubic(
                    37.290435, 40.42887, 29.913043, 53.065391, 29.913043, 66.432348,
                ),
                VectorCommand::Line(29.903548, 66.432348),
                VectorCommand::Cubic(
                    29.894247, 86.592348, 46.265287, 102.954087, 66.425287, 102.954087,
                ),
                VectorCommand::Cubic(
                    86.585287, 102.954087, 102.947026, 86.592348, 102.947026, 66.432348,
                ),
                VectorCommand::Line(102.93753, 66.432348),
                VectorCommand::Cubic(
                    102.930713, 53.065391, 95.560139, 40.380174, 83.288835, 34.049739,
                ),
                VectorCommand::Cubic(
                    82.722604, 33.754287, 82.103783, 33.573108, 81.467617, 33.516522,
                ),
                VectorCommand::Cubic(
                    79.503529, 33.34365, 77.62903, 34.372221, 76.719791, 36.121739,
                ),
                VectorCommand::Cubic(
                    76.121801, 37.268337, 76.004442, 38.605705, 76.393592, 39.838929,
                ),
                VectorCommand::Cubic(
                    76.782743, 41.072153, 77.646441, 42.099943, 78.794226, 42.695652,
                ),
                VectorCommand::Cubic(
                    87.82727, 47.394783, 93.208139, 56.622609, 93.208139, 66.45913,
                ),
                VectorCommand::Line(93.213009, 66.45913),
                VectorCommand::Cubic(
                    93.220313, 81.262609, 81.233878, 93.241739, 66.4304, 93.241739,
                ),
                VectorCommand::Cubic(
                    51.626922, 93.241739, 39.647791, 81.262609, 39.647791, 66.45913,
                ),
                VectorCommand::Line(39.652661, 66.45913),
                VectorCommand::Cubic(
                    39.641461, 56.646957, 45.03353, 47.394783, 54.066574, 42.695652,
                ),
                VectorCommand::Cubic(
                    55.214359, 42.099943, 56.078057, 41.072153, 56.467208, 39.838929,
                ),
                VectorCommand::Cubic(
                    56.856358, 38.605705, 56.738999, 37.268337, 56.141009, 36.121739,
                ),
                VectorCommand::Cubic(
                    55.546177, 34.977403, 54.521678, 34.115774, 53.292313, 33.725913,
                ),
                VectorCommand::Cubic(
                    52.682831, 33.534633, 52.041668, 33.465139, 51.405357, 33.521391,
                ),
                VectorCommand::Line(51.387826, 33.516522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 27.478261),
                VectorCommand::Cubic(
                    61.297391, 27.478261, 59.130435, 29.650087, 59.130435, 32.347826,
                ),
                VectorCommand::Line(59.130435, 59.130435),
                VectorCommand::Cubic(59.130435, 61.833043, 61.302261, 64.0, 64.0, 64.0),
                VectorCommand::Cubic(66.697739, 64.0, 68.869565, 61.828174, 68.869565, 59.130435),
                VectorCommand::Line(68.869565, 32.347826),
                VectorCommand::Cubic(68.869565, 29.645217, 66.697739, 27.478261, 64.0, 27.478261),
                VectorCommand::Close,
                VectorCommand::Move(48.953043, 31.081739),
                VectorCommand::Cubic(
                    48.315212, 31.137697, 47.694677, 31.318893, 47.126957, 31.614957,
                ),
                VectorCommand::Cubic(
                    34.855652, 37.994087, 27.478261, 50.630609, 27.478261, 63.997565,
                ),
                VectorCommand::Line(27.468741, 63.997565),
                VectorCommand::Cubic(
                    27.45944, 84.157565, 43.83048, 100.519304, 63.99048, 100.519304,
                ),
                VectorCommand::Cubic(
                    84.15048, 100.519304, 100.512219, 84.157565, 100.512219, 63.997565,
                ),
                VectorCommand::Line(100.502699, 63.997565),
                VectorCommand::Cubic(
                    100.495979, 50.630609, 93.125308, 37.945391, 80.854003, 31.614957,
                ),
                VectorCommand::Cubic(
                    80.287773, 31.319505, 79.668952, 31.138326, 79.032786, 31.081739,
                ),
                VectorCommand::Cubic(
                    77.068698, 30.908868, 75.194199, 31.937439, 74.28496, 33.686957,
                ),
                VectorCommand::Cubic(
                    73.68697, 34.833555, 73.56961, 36.170923, 73.958761, 37.404147,
                ),
                VectorCommand::Cubic(74.347912, 38.63737, 75.21161, 39.66516, 76.359395, 40.26087),
                VectorCommand::Cubic(85.392438, 44.96, 90.773308, 54.187826, 90.773308, 64.024348),
                VectorCommand::Line(90.778056, 64.024348),
                VectorCommand::Cubic(
                    90.78536, 78.827826, 78.798925, 90.806957, 63.995447, 90.806957,
                ),
                VectorCommand::Cubic(
                    49.191969, 90.806957, 37.212838, 78.827826, 37.212838, 64.024348,
                ),
                VectorCommand::Line(37.217586, 64.024348),
                VectorCommand::Cubic(37.206435, 54.212174, 42.598456, 44.96, 51.631499, 40.26087),
                VectorCommand::Cubic(
                    52.779284, 39.66516, 53.642982, 38.63737, 54.032133, 37.404147,
                ),
                VectorCommand::Cubic(
                    54.421283, 36.170923, 54.303924, 34.833555, 53.705934, 33.686957,
                ),
                VectorCommand::Cubic(
                    53.111102, 32.542621, 52.086603, 31.680991, 50.857238, 31.29113,
                ),
                VectorCommand::Cubic(
                    50.247756, 31.09985, 49.606594, 31.030356, 48.970282, 31.086609,
                ),
                VectorCommand::Line(48.953043, 31.081739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(243, 188, 192, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_FILE_MANAGER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x42374f),
                bottom: Color::from_hex(0x584969),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 84.695652),
                VectorCommand::Cubic(
                    37.217391, 88.055652, 39.944348, 90.782609, 43.304348, 90.782609,
                ),
                VectorCommand::Line(89.565217, 90.782609),
                VectorCommand::Cubic(
                    92.925217, 90.782609, 95.652174, 88.055652, 95.652174, 84.695652,
                ),
                VectorCommand::Line(95.652174, 48.173913),
                VectorCommand::Cubic(
                    95.652174, 44.813913, 92.925217, 42.086957, 89.565217, 42.086957,
                ),
                VectorCommand::Line(37.217391, 84.695652),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(40.869565, 44.521739),
                VectorCommand::Cubic(
                    37.509565, 44.521739, 34.782609, 47.248696, 34.782609, 50.608696,
                ),
                VectorCommand::Line(34.782609, 73.73913),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Line(83.478261, 44.521739),
                VectorCommand::Line(40.869565, 44.521739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(197, 139, 75, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(40.869565, 44.521739),
                VectorCommand::Cubic(
                    37.509565, 44.521739, 34.782609, 47.248696, 34.782609, 50.608696,
                ),
                VectorCommand::Line(34.782609, 53.043478),
                VectorCommand::Cubic(
                    34.782609, 49.683478, 37.509565, 46.956522, 40.869565, 46.956522,
                ),
                VectorCommand::Line(83.478261, 46.956522),
                VectorCommand::Line(83.478261, 44.521739),
                VectorCommand::Line(40.869565, 44.521739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 240, 218, 51))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(71.304348, 39.652174),
                VectorCommand::Cubic(
                    61.565217, 39.652174, 66.434783, 54.26087, 51.826087, 54.26087,
                ),
                VectorCommand::Line(40.869565, 54.26087),
                VectorCommand::Cubic(
                    37.509565, 54.26087, 34.782609, 56.987826, 34.782609, 60.347826,
                ),
                VectorCommand::Line(34.782609, 82.26087),
                VectorCommand::Cubic(
                    34.782609, 85.62087, 37.509565, 88.347826, 40.869565, 88.347826,
                ),
                VectorCommand::Line(87.130435, 88.347826),
                VectorCommand::Cubic(
                    90.490435, 88.347826, 93.217391, 85.62087, 93.217391, 82.26087,
                ),
                VectorCommand::Line(93.217391, 45.73913),
                VectorCommand::Cubic(
                    93.217391, 42.37913, 90.490435, 39.652174, 87.130435, 39.652174,
                ),
                VectorCommand::Line(71.304348, 39.652174),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(240, 201, 128, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 79.826087),
                VectorCommand::Line(34.782609, 82.26087),
                VectorCommand::Cubic(
                    34.782609, 85.62087, 37.509565, 88.347826, 40.869565, 88.347826,
                ),
                VectorCommand::Line(87.130435, 88.347826),
                VectorCommand::Cubic(
                    90.490435, 88.347826, 93.217391, 85.62087, 93.217391, 82.26087,
                ),
                VectorCommand::Line(93.217391, 79.826087),
                VectorCommand::Cubic(
                    93.217391, 83.186087, 90.490435, 85.913043, 87.130435, 85.913043,
                ),
                VectorCommand::Line(40.869565, 85.913043),
                VectorCommand::Cubic(
                    37.509565, 85.913043, 34.782609, 83.186087, 34.782609, 79.826087,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 25))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_OFFICE_VIEWER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0x6a4356),
                bottom: Color::from_hex(0x513243),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Rect {
                x: 42.086957,
                y: 34.782609,
                width: 48.695652,
                height: 63.304348,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 32.347826),
                VectorCommand::Line(42.086957, 64.0),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Cubic(
                    86.18087, 95.652174, 88.347826, 93.480348, 88.347826, 90.782609,
                ),
                VectorCommand::Line(88.347826, 37.217391),
                VectorCommand::Cubic(
                    88.347826, 34.514783, 86.176, 32.347826, 83.478261, 32.347826,
                ),
                VectorCommand::Line(81.043478, 32.347826),
                VectorCommand::Line(76.173913, 34.782609),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Line(46.956522, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(223, 235, 238, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.521739, 32.347826),
                VectorCommand::Cubic(
                    41.81913, 32.347826, 39.652174, 34.519652, 39.652174, 37.217391,
                ),
                VectorCommand::Line(39.652174, 90.782609),
                VectorCommand::Cubic(
                    39.652174, 93.485217, 41.824, 95.652174, 44.521739, 95.652174,
                ),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(46.956522, 32.347826),
                VectorCommand::Line(44.521739, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(114, 152, 169, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(73.73913, 32.347826),
                VectorCommand::Line(83.478261, 32.347826),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Line(78.608696, 56.695652),
                VectorCommand::Line(73.73913, 61.565217),
                VectorCommand::Line(73.73913, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(71.304348, 32.347826),
                VectorCommand::Line(81.043478, 32.347826),
                VectorCommand::Line(81.043478, 59.130435),
                VectorCommand::Line(76.173913, 54.26087),
                VectorCommand::Line(71.304348, 59.130435),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(210, 120, 153, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(55.478261, 73.73913),
                VectorCommand::Cubic(
                    54.803826, 73.73913, 54.26087, 74.282087, 54.26087, 74.956522,
                ),
                VectorCommand::Cubic(
                    54.26087, 75.630957, 54.803826, 76.173913, 55.478261, 76.173913,
                ),
                VectorCommand::Line(79.826087, 76.173913),
                VectorCommand::Cubic(
                    80.500522, 76.173913, 81.043478, 75.630957, 81.043478, 74.956522,
                ),
                VectorCommand::Cubic(
                    81.043478, 74.282087, 80.500522, 73.73913, 79.826087, 73.73913,
                ),
                VectorCommand::Line(55.478261, 73.73913),
                VectorCommand::Close,
                VectorCommand::Move(55.478261, 78.608696),
                VectorCommand::Cubic(
                    54.803826, 78.608696, 54.26087, 79.151652, 54.26087, 79.826087,
                ),
                VectorCommand::Cubic(
                    54.26087, 80.500522, 54.803826, 81.043478, 55.478261, 81.043478,
                ),
                VectorCommand::Line(79.826087, 81.043478),
                VectorCommand::Cubic(
                    80.500522, 81.043478, 81.043478, 80.500522, 81.043478, 79.826087,
                ),
                VectorCommand::Cubic(
                    81.043478, 79.151652, 80.500522, 78.608696, 79.826087, 78.608696,
                ),
                VectorCommand::Line(55.478261, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(55.478261, 83.478261),
                VectorCommand::Cubic(
                    54.803826, 83.478261, 54.26087, 84.021217, 54.26087, 84.695652,
                ),
                VectorCommand::Cubic(
                    54.26087, 85.370087, 54.803826, 85.913043, 55.478261, 85.913043,
                ),
                VectorCommand::Line(79.826087, 85.913043),
                VectorCommand::Cubic(
                    80.500522, 85.913043, 81.043478, 85.370087, 81.043478, 84.695652,
                ),
                VectorCommand::Cubic(
                    81.043478, 84.021217, 80.500522, 83.478261, 79.826087, 83.478261,
                ),
                VectorCommand::Line(55.478261, 83.478261),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(64, 95, 112, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_SUB2API_MONITOR: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0x2a6054),
                bottom: Color::from_hex(0x20493f),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 34.782609),
                VectorCommand::Line(44.521739, 64.0),
                VectorCommand::Line(29.913043, 64.0),
                VectorCommand::Line(29.913043, 68.869565),
                VectorCommand::Line(48.173913, 68.869565),
                VectorCommand::Line(56.695652, 48.173913),
                VectorCommand::Line(76.173913, 98.086957),
                VectorCommand::Line(88.347826, 68.869565),
                VectorCommand::Line(102.956522, 68.869565),
                VectorCommand::Line(102.956522, 64.0),
                VectorCommand::Line(84.695652, 64.0),
                VectorCommand::Line(76.173913, 84.695652),
                VectorCommand::Line(56.695652, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(54.26087, 32.347826),
                VectorCommand::Line(42.086957, 61.565217),
                VectorCommand::Line(27.478261, 61.565217),
                VectorCommand::Line(27.478261, 66.434783),
                VectorCommand::Line(45.73913, 66.434783),
                VectorCommand::Line(54.26087, 45.73913),
                VectorCommand::Line(73.73913, 95.652174),
                VectorCommand::Line(85.913043, 66.434783),
                VectorCommand::Line(100.521739, 66.434783),
                VectorCommand::Line(100.521739, 61.565217),
                VectorCommand::Line(82.26087, 61.565217),
                VectorCommand::Line(73.73913, 82.26087),
                VectorCommand::Line(54.26087, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(171, 225, 206, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_WIFI: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x2c442f),
                bottom: Color::from_hex(0x3b5b3e),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(88.808, 34.782609),
                VectorCommand::Line(82.944557, 40.522365),
                VectorCommand::Cubic(
                    92.0896, 49.463374, 92.164591, 63.91527, 82.996865, 72.873322,
                ),
                VectorCommand::Line(88.865178, 78.608452),
                VectorCommand::Cubic(
                    101.199787, 66.548974, 101.135509, 46.834539, 88.808112, 34.782365,
                ),
                VectorCommand::Line(88.808, 34.782609),
                VectorCommand::Close,
                VectorCommand::Move(82.944557, 40.522365),
                VectorCommand::Line(82.939801, 40.51761),
                VectorCommand::Line(82.939801, 40.52712),
                VectorCommand::Line(82.944557, 40.522365),
                VectorCommand::Close,
                VectorCommand::Move(44.063513, 34.830087),
                VectorCommand::Cubic(
                    31.719165, 46.889565, 31.679235, 66.506609, 44.006447, 78.551478,
                ),
                VectorCommand::Line(49.87476, 72.816348),
                VectorCommand::Cubic(
                    40.724847, 63.866087, 40.767455, 49.529843, 49.927069, 40.570087,
                ),
                VectorCommand::Line(44.063625, 34.83033),
                VectorCommand::Line(44.063513, 34.830087),
                VectorCommand::Close,
                VectorCommand::Move(78.703165, 44.7024),
                VectorCommand::Line(75.479026, 47.874191),
                VectorCommand::Cubic(
                    80.501983, 52.809496, 80.537774, 60.776348, 75.507557, 65.721391,
                ),
                VectorCommand::Line(78.7222, 68.893183),
                VectorCommand::Cubic(
                    85.483592, 62.246226, 85.457296, 51.36153, 78.703177, 44.7024,
                ),
                VectorCommand::Line(78.703165, 44.7024),
                VectorCommand::Close,
                VectorCommand::Move(54.165426, 44.764219),
                VectorCommand::Cubic(
                    47.404035, 51.403871, 47.375548, 62.246445, 54.136893, 68.883663,
                ),
                VectorCommand::Line(57.351536, 65.697506),
                VectorCommand::Cubic(
                    52.338319, 60.779245, 52.357067, 52.866932, 57.38007, 47.91215,
                ),
                VectorCommand::Line(54.165426, 44.763976),
                VectorCommand::Line(54.165426, 44.764219),
                VectorCommand::Close,
                VectorCommand::Move(66.434539, 51.835558),
                VectorCommand::Cubic(
                    63.744104, 51.835558, 61.564974, 54.012473, 61.564974, 56.695628,
                ),
                VectorCommand::Cubic(
                    61.564974, 58.498341, 62.544803, 60.065123, 63.999757, 60.90415,
                ),
                VectorCommand::Line(64.0378, 95.650932),
                VectorCommand::Line(46.956339, 95.650932),
                VectorCommand::Line(46.956339, 100.520497),
                VectorCommand::Line(85.912861, 100.520497),
                VectorCommand::Line(85.912861, 95.650932),
                VectorCommand::Line(68.859887, 95.650932),
                VectorCommand::Line(68.821843, 60.930932),
                VectorCommand::Cubic(
                    70.30319, 60.098967, 71.304104, 58.516261, 71.304104, 56.693923,
                ),
                VectorCommand::Cubic(
                    71.304104, 54.013228, 69.124974, 51.833854, 66.434539, 51.833854,
                ),
                VectorCommand::Line(66.434539, 51.835558),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.73113, 42.330435),
                VectorCommand::Cubic(
                    44.969739, 48.970087, 44.940522, 59.812174, 51.701913, 66.449391,
                ),
                VectorCommand::Line(54.915826, 63.262261),
                VectorCommand::Cubic(
                    49.902609, 58.344, 49.922087, 50.430957, 54.945043, 45.476174,
                ),
                VectorCommand::Move(73.045217, 45.437217),
                VectorCommand::Cubic(
                    78.068174, 50.372522, 78.104696, 58.344, 73.074435, 63.289043,
                ),
                VectorCommand::Line(76.288348, 66.456696),
                VectorCommand::Cubic(
                    83.049739, 59.809739, 83.022957, 48.928696, 76.26887, 42.269565,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(164, 214, 153, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(41.631652, 32.396522),
                VectorCommand::Cubic(
                    29.287304, 44.456, 29.248348, 64.073043, 41.575652, 76.117913,
                ),
                VectorCommand::Line(47.443478, 70.384),
                VectorCommand::Cubic(
                    38.293565, 61.433739, 38.332522, 47.09287, 47.492174, 38.13287,
                ),
                VectorCommand::Move(80.505391, 38.084174),
                VectorCommand::Cubic(89.655304, 47.024696, 89.730783, 61.48, 80.561391, 70.44),
                VectorCommand::Line(86.429217, 76.173913),
                VectorCommand::Cubic(98.763826, 64.114435, 98.700522, 44.4, 86.373217, 32.347826),
                VectorCommand::Line(80.505391, 38.091478),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(164, 214, 153, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 49.401043),
                VectorCommand::Cubic(
                    61.309565, 49.401043, 59.130435, 51.577958, 59.130435, 54.261113,
                ),
                VectorCommand::Cubic(
                    59.130435, 56.063826, 60.110264, 57.630609, 61.565217, 58.469635,
                ),
                VectorCommand::Line(61.603261, 93.216417),
                VectorCommand::Line(44.5218, 93.216417),
                VectorCommand::Line(44.5218, 98.085983),
                VectorCommand::Line(83.478322, 98.085983),
                VectorCommand::Line(83.478322, 93.216417),
                VectorCommand::Line(66.425348, 93.216417),
                VectorCommand::Line(66.387304, 58.496417),
                VectorCommand::Cubic(
                    67.86865, 57.664452, 68.869565, 56.081746, 68.869565, 54.259409,
                ),
                VectorCommand::Cubic(68.869565, 51.578713, 66.690435, 49.399339, 64.0, 49.399339),
                VectorCommand::Line(64.0, 49.401043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(225, 236, 220, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_BLUETOOTH: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x264457),
                bottom: Color::from_hex(0x355b70),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.565217, 29.913043),
                VectorCommand::Line(60.347826, 66.434783),
                VectorCommand::Line(42.086957, 83.478261),
                VectorCommand::Line(46.956522, 88.347826),
                VectorCommand::Line(61.565217, 73.73913),
                VectorCommand::Line(61.565217, 102.956522),
                VectorCommand::Line(90.782609, 81.043478),
                VectorCommand::Line(72.05913, 66.434783),
                VectorCommand::Line(90.782609, 51.826087),
                VectorCommand::Move(68.869565, 44.521739),
                VectorCommand::Line(78.608696, 51.826087),
                VectorCommand::Line(68.869565, 59.130435),
                VectorCommand::Move(68.869565, 73.73913),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Line(68.869565, 88.347826),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.130435, 27.478261),
                VectorCommand::Line(59.130435, 56.695652),
                VectorCommand::Line(44.521739, 42.086957),
                VectorCommand::Line(39.652174, 46.956522),
                VectorCommand::Line(57.913043, 64.0),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Line(44.521739, 85.913043),
                VectorCommand::Line(59.130435, 71.304348),
                VectorCommand::Line(59.130435, 100.521739),
                VectorCommand::Line(88.347826, 78.608696),
                VectorCommand::Line(69.624348, 64.0),
                VectorCommand::Line(88.347826, 49.391304),
                VectorCommand::Move(66.434783, 42.086957),
                VectorCommand::Line(76.173913, 49.391304),
                VectorCommand::Line(66.434783, 56.695652),
                VectorCommand::Move(66.434783, 71.304348),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Line(66.434783, 85.913043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(185, 223, 242, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_DISPLAY_SETTINGS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x254b45),
                bottom: Color::from_hex(0x345f56),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.094609, 34.782609),
                VectorCommand::Cubic(
                    36.698783, 34.782609, 34.777739, 36.703652, 34.777739, 39.099478,
                ),
                VectorCommand::Line(34.782609, 83.478261),
                VectorCommand::Line(59.130435, 83.478261),
                VectorCommand::Cubic(
                    59.130435, 90.782609, 59.130435, 90.782609, 54.26087, 95.652174,
                ),
                VectorCommand::Line(49.391304, 95.652174),
                VectorCommand::Line(49.391304, 98.086957),
                VectorCommand::Line(83.478261, 98.086957),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(78.608696, 95.652174),
                VectorCommand::Cubic(
                    73.73913, 90.782609, 73.73913, 90.782609, 73.73913, 83.478261,
                ),
                VectorCommand::Line(98.086957, 83.478261),
                VectorCommand::Line(98.082087, 39.099478),
                VectorCommand::Cubic(
                    98.082087, 36.703652, 96.165913, 34.782609, 93.770087, 34.782609,
                ),
                VectorCommand::Line(39.101913, 34.782609),
                VectorCommand::Line(39.094609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.659826, 32.347826),
                VectorCommand::Cubic(34.264, 32.347826, 32.341983, 34.269941, 32.341983, 36.66567),
                VectorCommand::Line(32.341983, 73.740104),
                VectorCommand::Line(63.994157, 78.60967),
                VectorCommand::Line(95.64633, 73.740104),
                VectorCommand::Line(95.64633, 36.66567),
                VectorCommand::Cubic(
                    95.64633, 34.269843, 93.728988, 32.347826, 91.333113, 32.347826,
                ),
                VectorCommand::Line(36.662504, 32.347826),
                VectorCommand::Line(36.659826, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(144, 174, 181, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 81.043478),
                VectorCommand::Cubic(
                    56.695652, 88.347826, 56.695652, 88.347826, 51.826087, 93.217391,
                ),
                VectorCommand::Line(64.0, 95.652174),
                VectorCommand::Line(76.173913, 93.217391),
                VectorCommand::Cubic(
                    71.304348, 88.347826, 71.304348, 88.347826, 71.304348, 81.043478,
                ),
                VectorCommand::Line(64.0, 73.73913),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(139, 168, 172, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 93.217391),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(81.043478, 95.652174),
                VectorCommand::Line(81.043478, 93.217391),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(185, 208, 208, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 73.73913),
                VectorCommand::Line(32.347826, 81.043478),
                VectorCommand::Line(95.652174, 81.043478),
                VectorCommand::Line(95.652174, 73.73913),
                VectorCommand::Line(32.347826, 73.73913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(185, 208, 208, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 37.217391),
                VectorCommand::Line(90.782609, 37.217391),
                VectorCommand::Line(90.782609, 68.869565),
                VectorCommand::Line(37.217391, 68.869565),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(220, 233, 233, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.684174, 32.347826),
                VectorCommand::Cubic(
                    34.298087, 32.347826, 32.347339, 34.298696, 32.347339, 36.684661,
                ),
                VectorCommand::Line(32.347339, 73.739617),
                VectorCommand::Line(78.608209, 73.739617),
                VectorCommand::Line(49.390817, 32.348313),
                VectorCommand::Line(36.684174, 32.348313),
                VectorCommand::Line(36.684174, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 76.173913),
                VectorCommand::Line(71.304348, 76.173913),
                VectorCommand::Line(71.304348, 78.608696),
                VectorCommand::Line(56.695652, 78.608696),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(144, 174, 181, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_DATE_TIME: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.682157, 46.956522),
                VectorCommand::Cubic(8.913092, 52.464974, 8.008278, 58.21447, 8.0, 64.0),
                VectorCommand::Cubic(8.0, 94.929043, 33.070957, 120.0, 64.0, 120.0),
                VectorCommand::Cubic(94.929043, 120.0, 120.0, 94.929043, 120.0, 64.0),
                VectorCommand::Cubic(
                    119.977934, 58.212278, 119.058737, 52.462783, 117.275235, 46.956522,
                ),
                VectorCommand::Cubic(
                    112.695652, 42.086957, 15.304104, 42.086957, 10.682887, 46.956522,
                ),
                VectorCommand::Line(10.682157, 46.956522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0x445467),
                bottom: Color::from_hex(0x334151),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.682157, 46.956522),
                VectorCommand::Cubic(
                    10.438783, 47.762922, 10.213631, 48.574727, 10.006894, 49.391304,
                ),
                VectorCommand::Line(118.045503, 49.391304),
                VectorCommand::Cubic(
                    117.807249, 48.573948, 117.55039, 47.762143, 117.275113, 46.956522,
                ),
                VectorCommand::Cubic(
                    112.69553, 42.086957, 15.303983, 42.086957, 10.682765, 46.956522,
                ),
                VectorCommand::Line(10.682157, 46.956522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(
                    39.647842, 8.011544, 18.094655, 23.759603, 10.68313, 46.956522,
                ),
                VectorCommand::Line(117.275478, 46.956522),
                VectorCommand::Cubic(109.868612, 23.774129, 88.336889, 8.02953, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(155, 85, 86, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 29.913043),
                VectorCommand::Cubic(
                    36.962787, 29.913043, 34.782609, 32.093222, 34.782609, 34.782609,
                ),
                VectorCommand::Cubic(
                    34.782609, 37.471995, 36.962787, 39.652174, 39.652174, 39.652174,
                ),
                VectorCommand::Cubic(
                    42.341561, 39.652174, 44.521739, 37.471995, 44.521739, 34.782609,
                ),
                VectorCommand::Cubic(
                    44.521739, 32.093222, 42.341561, 29.913043, 39.652174, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(51.826087, 29.913043),
                VectorCommand::Cubic(
                    49.1367, 29.913043, 46.956522, 32.093222, 46.956522, 34.782609,
                ),
                VectorCommand::Cubic(
                    46.956522, 37.471995, 49.1367, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Cubic(
                    54.515474, 39.652174, 56.695652, 37.471995, 56.695652, 34.782609,
                ),
                VectorCommand::Cubic(
                    56.695652, 32.093222, 54.515474, 29.913043, 51.826087, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 29.913043),
                VectorCommand::Cubic(
                    73.484526, 29.913043, 71.304348, 32.093222, 71.304348, 34.782609,
                ),
                VectorCommand::Cubic(
                    71.304348, 37.471995, 73.484526, 39.652174, 76.173913, 39.652174,
                ),
                VectorCommand::Cubic(
                    78.8633, 39.652174, 81.043478, 37.471995, 81.043478, 34.782609,
                ),
                VectorCommand::Cubic(
                    81.043478, 32.093222, 78.8633, 29.913043, 76.173913, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(88.347826, 29.913043),
                VectorCommand::Cubic(
                    85.658439, 29.913043, 83.478261, 32.093222, 83.478261, 34.782609,
                ),
                VectorCommand::Cubic(
                    83.478261, 37.471995, 85.658439, 39.652174, 88.347826, 39.652174,
                ),
                VectorCommand::Cubic(
                    91.037213, 39.652174, 93.217391, 37.471995, 93.217391, 34.782609,
                ),
                VectorCommand::Cubic(
                    93.217391, 32.093222, 91.037213, 29.913043, 88.347826, 29.913043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Cubic(
                    38.303304, 32.347826, 37.217391, 33.433739, 37.217391, 34.782609,
                ),
                VectorCommand::Cubic(
                    37.217391, 36.131478, 38.303304, 37.217391, 39.652174, 37.217391,
                ),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Cubic(
                    53.174957, 37.217391, 54.26087, 36.131478, 54.26087, 34.782609,
                ),
                VectorCommand::Cubic(
                    54.26087, 33.433739, 53.174957, 32.347826, 51.826087, 32.347826,
                ),
                VectorCommand::Line(39.652174, 32.347826),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 32.347826),
                VectorCommand::Cubic(
                    74.825043, 32.347826, 73.73913, 33.433739, 73.73913, 34.782609,
                ),
                VectorCommand::Cubic(
                    73.73913, 36.131478, 74.825043, 37.217391, 76.173913, 37.217391,
                ),
                VectorCommand::Line(88.347826, 37.217391),
                VectorCommand::Cubic(
                    89.696696, 37.217391, 90.782609, 36.131478, 90.782609, 34.782609,
                ),
                VectorCommand::Cubic(
                    90.782609, 33.433739, 89.696696, 32.347826, 88.347826, 32.347826,
                ),
                VectorCommand::Line(76.173913, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(234, 202, 198, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.616068, 72.230978),
                VectorCommand::Quad(61.616068, 73.887739, 61.077058, 75.247194),
                VectorCommand::Quad(60.538048, 76.609047, 59.556457, 77.60646),
                VectorCommand::Quad(58.597119, 78.603873, 57.242177, 79.263221),
                VectorCommand::Quad(55.884762, 79.898592, 54.2257, 80.169524),
                VectorCommand::Line(54.2257, 80.306189),
                VectorCommand::Quad(58.364702, 80.804896, 60.515796, 82.869253),
                VectorCommand::Quad(62.66689, 84.909634, 62.66689, 88.19918),
                VectorCommand::Quad(62.66689, 90.376226, 61.895463, 92.19123),
                VectorCommand::Quad(61.146289, 94.006234, 59.603435, 95.320134),
                VectorCommand::Quad(58.060582, 96.636432, 55.696851, 97.360516),
                VectorCommand::Quad(53.335593, 98.086997, 50.106479, 98.086997),
                VectorCommand::Quad(47.557309, 98.086997, 45.287534, 97.700979),
                VectorCommand::Quad(43.042485, 97.314961, 41.054577, 96.295969),
                VectorCommand::Line(41.054577, 92.440583),
                VectorCommand::Quad(43.089462, 93.483552, 45.450721, 94.051789),
                VectorCommand::Quad(47.836704, 94.617629, 50.012523, 94.617629),
                VectorCommand::Quad(52.163617, 94.617629, 53.731196, 94.164478),
                VectorCommand::Quad(55.298775, 93.711326, 56.302619, 92.872156),
                VectorCommand::Quad(57.331188, 92.032987, 57.798494, 90.807799),
                VectorCommand::Quad(58.290526, 89.582611, 58.290526, 88.062515),
                VectorCommand::Quad(58.290526, 86.52084, 57.660033, 85.432317),
                VectorCommand::Quad(57.051793, 84.322216, 55.907015, 83.617313),
                VectorCommand::Quad(54.762238, 82.890832, 53.100703, 82.550369),
                VectorCommand::Quad(51.463894, 82.209905, 49.38203, 82.209905),
                VectorCommand::Line(46.271598, 82.209905),
                VectorCommand::Line(46.271598, 78.786093),
                VectorCommand::Line(49.38203, 78.786093),
                VectorCommand::Quad(51.275982, 78.786093, 52.749605, 78.332941),
                VectorCommand::Quad(54.223228, 77.87979, 55.204819, 77.04062),
                VectorCommand::Quad(56.211135, 76.20145, 56.72542, 75.045794),
                VectorCommand::Quad(57.239704, 73.890137, 57.239704, 72.506706),
                VectorCommand::Quad(57.239704, 71.327073, 56.819376, 70.396793),
                VectorCommand::Quad(56.399047, 69.466514, 55.62762, 68.831142),
                VectorCommand::Quad(54.856193, 68.174192, 53.780646, 67.833729),
                VectorCommand::Quad(52.705099, 67.493266, 51.394663, 67.493266),
                VectorCommand::Quad(48.892471, 67.493266, 46.973794, 68.265302),
                VectorCommand::Quad(45.079842, 69.013362, 43.349077, 70.23855),
                VectorCommand::Line(41.197983, 67.404554),
                VectorCommand::Quad(42.085619, 66.702049, 43.163638, 66.088256),
                VectorCommand::Quad(44.26391, 65.476861, 45.549622, 65.021312),
                VectorCommand::Quad(46.835333, 64.544184, 48.286703, 64.273252),
                VectorCommand::Quad(49.760326, 63.999922, 51.397135, 63.999922),
                VectorCommand::Quad(53.924053, 63.999922, 55.818004, 64.611317),
                VectorCommand::Quad(57.736681, 65.222712, 59.022393, 66.335212),
                VectorCommand::Quad(60.308104, 67.423735, 60.963322, 68.943831),
                VectorCommand::Quad(61.61854, 70.43995, 61.61854, 72.233376),
                VectorCommand::Line(61.616068, 72.230978),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(82.150361, 97.631448),
                VectorCommand::Line(78.033612, 97.631448),
                VectorCommand::Line(78.033612, 76.925534),
                VectorCommand::Quad(78.033612, 75.949699, 78.033612, 74.839598),
                VectorCommand::Quad(78.055865, 73.729496, 78.08059, 72.638575),
                VectorCommand::Quad(78.127568, 71.528474, 78.149821, 70.528663),
                VectorCommand::Quad(78.196799, 69.507273, 78.219051, 68.737635),
                VectorCommand::Quad(77.820975, 69.145231, 77.516855, 69.44014),
                VectorCommand::Quad(77.212735, 69.735048, 76.886362, 70.00598),
                VectorCommand::Quad(76.582242, 70.279309, 76.231144, 70.595796),
                VectorCommand::Quad(75.880046, 70.890705, 75.388014, 71.298301),
                VectorCommand::Line(71.926483, 74.043585),
                VectorCommand::Line(69.681434, 71.255144),
                VectorCommand::Line(78.63938, 64.474652),
                VectorCommand::Line(82.147888, 64.474652),
                VectorCommand::Line(82.147888, 97.631448),
                VectorCommand::Line(82.150361, 97.631448),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.181379, 69.794989),
                VectorCommand::Quad(59.181379, 71.451749, 58.642369, 72.811204),
                VectorCommand::Quad(58.103359, 74.173057, 57.121768, 75.17047),
                VectorCommand::Quad(56.16243, 76.167883, 54.807488, 76.827231),
                VectorCommand::Quad(53.450073, 77.462602, 51.791011, 77.733534),
                VectorCommand::Line(51.791011, 77.870199),
                VectorCommand::Quad(55.930013, 78.368906, 58.081107, 80.433263),
                VectorCommand::Quad(60.2322, 82.473645, 60.2322, 85.76319),
                VectorCommand::Quad(60.2322, 87.940236, 59.460774, 89.75524),
                VectorCommand::Quad(58.7116, 91.570245, 57.168746, 92.884145),
                VectorCommand::Quad(55.625892, 94.200442, 53.262162, 94.924526),
                VectorCommand::Quad(50.900903, 95.651007, 47.67179, 95.651007),
                VectorCommand::Quad(45.12262, 95.651007, 42.852845, 95.264989),
                VectorCommand::Quad(40.607795, 94.878971, 38.619888, 93.859979),
                VectorCommand::Line(38.619888, 90.004594),
                VectorCommand::Quad(40.654773, 91.047562, 43.016032, 91.615799),
                VectorCommand::Quad(45.402015, 92.18164, 47.577834, 92.18164),
                VectorCommand::Quad(49.728928, 92.18164, 51.296507, 91.728488),
                VectorCommand::Quad(52.864086, 91.275336, 53.867929, 90.436167),
                VectorCommand::Quad(54.896499, 89.596997, 55.363805, 88.371809),
                VectorCommand::Quad(55.855837, 87.146621, 55.855837, 85.626525),
                VectorCommand::Quad(55.855837, 84.08485, 55.225344, 82.996327),
                VectorCommand::Quad(54.617104, 81.886226, 53.472326, 81.181323),
                VectorCommand::Quad(52.327548, 80.454842, 50.666014, 80.114379),
                VectorCommand::Quad(49.029204, 79.773916, 46.947341, 79.773916),
                VectorCommand::Line(43.836909, 79.773916),
                VectorCommand::Line(43.836909, 76.350103),
                VectorCommand::Line(46.947341, 76.350103),
                VectorCommand::Quad(48.841293, 76.350103, 50.314916, 75.896951),
                VectorCommand::Quad(51.788539, 75.4438, 52.77013, 74.60463),
                VectorCommand::Quad(53.776446, 73.76546, 54.290731, 72.609804),
                VectorCommand::Quad(54.805015, 71.454147, 54.805015, 70.070716),
                VectorCommand::Quad(54.805015, 68.891083, 54.384687, 67.960803),
                VectorCommand::Quad(53.964358, 67.030524, 53.192931, 66.395152),
                VectorCommand::Quad(52.421504, 65.738202, 51.345957, 65.397739),
                VectorCommand::Quad(50.27041, 65.057276, 48.959974, 65.057276),
                VectorCommand::Quad(46.457782, 65.057276, 44.539105, 65.829312),
                VectorCommand::Quad(42.645153, 66.577372, 40.914388, 67.80256),
                VectorCommand::Line(38.763294, 64.968564),
                VectorCommand::Quad(39.650929, 64.266059, 40.728949, 63.652266),
                VectorCommand::Quad(41.829221, 63.040871, 43.114932, 62.585322),
                VectorCommand::Quad(44.400644, 62.108194, 45.852014, 61.837262),
                VectorCommand::Quad(47.325637, 61.563932, 48.962446, 61.563932),
                VectorCommand::Quad(51.489364, 61.563932, 53.383315, 62.175327),
                VectorCommand::Quad(55.301992, 62.786722, 56.587703, 63.899222),
                VectorCommand::Quad(57.873415, 64.987745, 58.528633, 66.507841),
                VectorCommand::Quad(59.183851, 68.003961, 59.183851, 69.797386),
                VectorCommand::Line(59.181379, 69.794989),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(224, 233, 240, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(79.715672, 95.185867),
                VectorCommand::Line(75.598923, 95.185867),
                VectorCommand::Line(75.598923, 74.479953),
                VectorCommand::Quad(75.598923, 73.504119, 75.598923, 72.394017),
                VectorCommand::Quad(75.621176, 71.283915, 75.645901, 70.192995),
                VectorCommand::Quad(75.692879, 69.082893, 75.715132, 68.083082),
                VectorCommand::Quad(75.762109, 67.061693, 75.784362, 66.292054),
                VectorCommand::Quad(75.386286, 66.699651, 75.082166, 66.994559),
                VectorCommand::Quad(74.778046, 67.289468, 74.451673, 67.560399),
                VectorCommand::Quad(74.147553, 67.833729, 73.796455, 68.150216),
                VectorCommand::Quad(73.445357, 68.445124, 72.953325, 68.852721),
                VectorCommand::Line(69.491794, 71.598005),
                VectorCommand::Line(67.246745, 68.809564),
                VectorCommand::Line(76.204691, 62.029072),
                VectorCommand::Line(79.713199, 62.029072),
                VectorCommand::Line(79.713199, 95.185867),
                VectorCommand::Line(79.715672, 95.185867),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(224, 233, 240, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_STORAGE: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0x36586b),
                bottom: Color::from_hex(0x294452),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(90.782609, 34.782609),
                VectorCommand::Cubic(
                    92.131478, 34.782609, 93.217391, 35.868522, 93.217391, 37.217391,
                ),
                VectorCommand::Line(93.217391, 95.652174),
                VectorCommand::Cubic(
                    93.217391, 97.001043, 92.131478, 98.086957, 90.782609, 98.086957,
                ),
                VectorCommand::Line(42.086957, 98.086957),
                VectorCommand::Cubic(
                    40.738087, 98.086957, 39.652174, 97.001043, 39.652174, 95.652174,
                ),
                VectorCommand::Line(90.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(90.782609, 85.913043),
                VectorCommand::Line(90.782609, 93.217391),
                VectorCommand::Cubic(
                    90.782609, 94.566261, 89.696696, 95.652174, 88.347826, 95.652174,
                ),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(64.0, 93.217391),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(39.652174, 95.652174),
                VectorCommand::Cubic(
                    38.303304, 95.652174, 37.217391, 94.566261, 37.217391, 93.217391,
                ),
                VectorCommand::Line(37.217391, 85.913043),
                VectorCommand::Line(90.782609, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(123, 154, 166, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.521739, 85.913043),
                VectorCommand::Line(83.478261, 85.913043),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(44.521739, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(83, 107, 121, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Line(88.347826, 32.347826),
                VectorCommand::Cubic(
                    89.696696, 32.347826, 90.782609, 33.433739, 90.782609, 34.782609,
                ),
                VectorCommand::Line(90.782609, 85.913043),
                VectorCommand::Cubic(
                    90.782609, 87.261913, 89.696696, 88.347826, 88.347826, 88.347826,
                ),
                VectorCommand::Line(39.652174, 88.347826),
                VectorCommand::Cubic(
                    38.303304, 88.347826, 37.217391, 87.261913, 37.217391, 85.913043,
                ),
                VectorCommand::Line(37.217391, 34.782609),
                VectorCommand::Cubic(
                    37.217391, 33.433739, 38.303304, 32.347826, 39.652174, 32.347826,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(141, 170, 185, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(50.608696, 90.782609),
                VectorCommand::Cubic(49.934261, 90.782609, 49.391304, 91.325565, 49.391304, 92.0),
                VectorCommand::Cubic(
                    49.391304, 92.674435, 49.934261, 93.217391, 50.608696, 93.217391,
                ),
                VectorCommand::Line(55.478261, 93.217391),
                VectorCommand::Cubic(56.152696, 93.217391, 56.695652, 92.674435, 56.695652, 92.0),
                VectorCommand::Cubic(
                    56.695652, 91.325565, 56.152696, 90.782609, 55.478261, 90.782609,
                ),
                VectorCommand::Line(50.608696, 90.782609),
                VectorCommand::Close,
                VectorCommand::Move(60.347826, 90.782609),
                VectorCommand::Cubic(59.673391, 90.782609, 59.130435, 91.325565, 59.130435, 92.0),
                VectorCommand::Cubic(
                    59.130435, 92.674435, 59.673391, 93.217391, 60.347826, 93.217391,
                ),
                VectorCommand::Line(77.391304, 93.217391),
                VectorCommand::Cubic(78.065739, 93.217391, 78.608696, 92.674435, 78.608696, 92.0),
                VectorCommand::Cubic(
                    78.608696, 91.325565, 78.065739, 90.782609, 77.391304, 90.782609,
                ),
                VectorCommand::Line(60.347826, 90.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(225, 186, 121, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.826087, 37.217391),
                VectorCommand::Cubic(
                    46.42087, 37.217391, 42.086957, 41.551304, 42.086957, 46.956522,
                ),
                VectorCommand::Line(42.086957, 57.353043),
                VectorCommand::Cubic(
                    43.542957, 58.193043, 44.521739, 59.763478, 44.521739, 61.565217,
                ),
                VectorCommand::Cubic(
                    44.521739, 63.366957, 43.542957, 64.949565, 42.086957, 65.777391,
                ),
                VectorCommand::Line(42.086957, 76.173913),
                VectorCommand::Line(49.391304, 83.478261),
                VectorCommand::Line(78.608696, 83.478261),
                VectorCommand::Line(85.913043, 76.173913),
                VectorCommand::Line(85.913043, 65.777391),
                VectorCommand::Cubic(
                    84.457043, 64.937391, 83.478261, 63.366957, 83.478261, 61.565217,
                ),
                VectorCommand::Cubic(
                    83.478261, 59.763478, 84.457043, 58.18087, 85.913043, 57.353043,
                ),
                VectorCommand::Line(85.913043, 46.956522),
                VectorCommand::Cubic(
                    85.913043, 41.551304, 81.57913, 37.217391, 76.173913, 37.217391,
                ),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(232, 240, 242, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(76.173913, 44.521739),
                VectorCommand::Cubic(
                    72.132174, 44.521739, 68.869565, 47.784348, 68.869565, 51.826087,
                ),
                VectorCommand::Line(56.695652, 64.0),
                VectorCommand::Cubic(52.653913, 64.0, 49.391304, 67.262609, 49.391304, 71.304348),
                VectorCommand::Cubic(
                    49.391334, 72.392696, 49.634783, 73.46887, 50.104696, 74.445217,
                ),
                VectorCommand::Line(54.974261, 69.575652),
                VectorCommand::Cubic(
                    55.926261, 68.626087, 57.457739, 68.626087, 58.407304, 69.575652,
                ),
                VectorCommand::Line(58.407304, 73.008696),
                VectorCommand::Line(53.537739, 77.878261),
                VectorCommand::Cubic(
                    54.521391, 78.348174, 55.597565, 78.591652, 56.678609, 78.591652,
                ),
                VectorCommand::Cubic(
                    60.720348, 78.591652, 63.982957, 75.329043, 63.982957, 71.287304,
                ),
                VectorCommand::Line(63.982957, 68.852522),
                VectorCommand::Line(73.722087, 59.113391),
                VectorCommand::Line(76.15687, 59.113391),
                VectorCommand::Cubic(
                    80.198609, 59.113391, 83.461217, 55.850783, 83.461217, 51.809043,
                ),
                VectorCommand::Cubic(
                    83.461188, 50.720696, 83.217739, 49.644522, 82.747826, 48.668174,
                ),
                VectorCommand::Line(77.878261, 53.537739),
                VectorCommand::Line(74.445217, 53.537739),
                VectorCommand::Cubic(
                    73.495652, 52.585739, 73.495652, 51.054261, 74.445217, 50.104696,
                ),
                VectorCommand::Line(79.314783, 45.23513),
                VectorCommand::Cubic(
                    78.33113, 44.765217, 77.254957, 44.521739, 76.173913, 44.521739,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(73.73913, 42.086957),
                VectorCommand::Cubic(
                    69.697391, 42.086957, 66.434783, 45.349565, 66.434783, 49.391304,
                ),
                VectorCommand::Line(66.434783, 51.826087),
                VectorCommand::Line(56.695652, 61.565217),
                VectorCommand::Line(54.26087, 61.565217),
                VectorCommand::Cubic(
                    50.21913, 61.565217, 46.956522, 64.827826, 46.956522, 68.869565,
                ),
                VectorCommand::Cubic(46.956551, 69.957913, 47.2, 71.034087, 47.669913, 72.010435),
                VectorCommand::Line(52.539478, 67.14087),
                VectorCommand::Cubic(
                    53.491478, 66.191304, 55.022957, 66.191304, 55.972522, 67.14087,
                ),
                VectorCommand::Cubic(
                    56.922087, 68.09287, 56.922087, 69.624348, 55.972522, 70.573913,
                ),
                VectorCommand::Line(51.102957, 75.443478),
                VectorCommand::Cubic(
                    52.086609, 75.913391, 53.162783, 76.15687, 54.243826, 76.15687,
                ),
                VectorCommand::Cubic(
                    58.285565, 76.15687, 61.548174, 72.894261, 61.548174, 68.852522,
                ),
                VectorCommand::Line(61.548174, 66.417739),
                VectorCommand::Line(71.287304, 56.678609),
                VectorCommand::Line(73.722087, 56.678609),
                VectorCommand::Cubic(
                    77.763826, 56.678609, 81.026435, 53.416, 81.026435, 49.374261,
                ),
                VectorCommand::Cubic(
                    81.026406, 48.285913, 80.782957, 47.209739, 80.313043, 46.233391,
                ),
                VectorCommand::Line(75.443478, 51.102957),
                VectorCommand::Cubic(74.491478, 52.052522, 72.96, 52.052522, 72.010435, 51.102957),
                VectorCommand::Cubic(
                    71.06087, 50.150957, 71.06087, 48.619478, 72.010435, 47.669913,
                ),
                VectorCommand::Line(76.88, 42.800348),
                VectorCommand::Cubic(
                    75.896348, 42.330435, 74.820174, 42.086957, 73.73913, 42.086957,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(86, 118, 138, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.678261, 54.504348),
                VectorCommand::Line(59.373913, 61.808696),
                VectorCommand::Cubic(
                    58.898665, 62.284087, 58.898665, 63.054696, 59.373913, 63.530087,
                ),
                VectorCommand::Cubic(
                    59.849304, 64.005335, 60.619913, 64.005335, 61.095304, 63.530087,
                ),
                VectorCommand::Line(68.399652, 56.225739),
                VectorCommand::Cubic(68.8749, 55.750348, 68.8749, 54.979739, 68.399652, 54.504348),
                VectorCommand::Cubic(67.924261, 54.0291, 67.153652, 54.0291, 66.678261, 54.504348),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(127, 158, 175, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_ABOUT: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x294d3a),
                bottom: Color::from_hex(0x3a654a),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.473739, 29.917913),
                VectorCommand::Cubic(
                    46.30327, 29.917913, 29.954435, 46.266504, 29.954435, 66.437217,
                ),
                VectorCommand::Cubic(
                    29.954435, 86.607687, 46.303026, 102.956522, 66.473739, 102.956522,
                ),
                VectorCommand::Cubic(
                    86.644209, 102.956522, 102.993043, 86.60793, 102.993043, 66.437217,
                ),
                VectorCommand::Cubic(
                    102.993043, 46.266748, 86.644452, 29.917913, 66.473739, 29.917913,
                ),
                VectorCommand::Line(66.473739, 29.917913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 36.521739,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(194, 223, 206, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 29.217391,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(59, 122, 133, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 46.956522),
                VectorCommand::Cubic(
                    63.745322, 46.956522, 61.565217, 49.136699, 61.565217, 51.826087,
                ),
                VectorCommand::Cubic(
                    61.565217, 54.515475, 63.745395, 56.695652, 66.434783, 56.695652,
                ),
                VectorCommand::Cubic(
                    69.12417, 56.695652, 71.304348, 54.515475, 71.304348, 51.826087,
                ),
                VectorCommand::Cubic(
                    71.304348, 49.136699, 69.12417, 46.956522, 66.434783, 46.956522,
                ),
                VectorCommand::Close,
                VectorCommand::Move(64.0, 61.565217),
                VectorCommand::Cubic(62.65113, 61.565217, 61.565217, 62.65113, 61.565217, 64.0),
                VectorCommand::Line(61.565217, 83.478261),
                VectorCommand::Cubic(61.565217, 84.82713, 62.65113, 85.913043, 64.0, 85.913043),
                VectorCommand::Line(68.869565, 85.913043),
                VectorCommand::Cubic(
                    70.218435, 85.913043, 71.304348, 84.82713, 71.304348, 83.478261,
                ),
                VectorCommand::Line(71.304348, 64.0),
                VectorCommand::Cubic(
                    71.304348, 62.65113, 70.218435, 61.565217, 68.869565, 61.565217,
                ),
                VectorCommand::Line(64.0, 61.565217),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 44.521739),
                VectorCommand::Cubic(
                    61.310539, 44.521739, 59.130435, 46.701917, 59.130435, 49.391304,
                ),
                VectorCommand::Cubic(59.130435, 52.080692, 61.310612, 54.26087, 64.0, 54.26087),
                VectorCommand::Cubic(
                    66.689388, 54.26087, 68.869565, 52.080692, 68.869565, 49.391304,
                ),
                VectorCommand::Cubic(68.869565, 46.701917, 66.689388, 44.521739, 64.0, 44.521739),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 59.130435),
                VectorCommand::Cubic(
                    60.216348, 59.130435, 59.130435, 60.216348, 59.130435, 61.565217,
                ),
                VectorCommand::Line(59.130435, 81.043478),
                VectorCommand::Cubic(
                    59.130435, 82.392348, 60.216348, 83.478261, 61.565217, 83.478261,
                ),
                VectorCommand::Line(66.434783, 83.478261),
                VectorCommand::Cubic(
                    67.783652, 83.478261, 68.869565, 82.392348, 68.869565, 81.043478,
                ),
                VectorCommand::Line(68.869565, 61.565217),
                VectorCommand::Cubic(
                    68.869565, 60.216348, 67.783652, 59.130435, 66.434783, 59.130435,
                ),
                VectorCommand::Line(61.565217, 59.130435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(240, 250, 247, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 34.782609),
                VectorCommand::Cubic(47.86368, 34.782609, 34.782609, 47.86368, 34.782609, 64.0),
                VectorCommand::Cubic(
                    34.788539, 72.523188, 38.515921, 80.618864, 44.987757, 86.165043,
                ),
                VectorCommand::Cubic(
                    40.018196, 80.793446, 37.245284, 73.752556, 37.217391, 66.434783,
                ),
                VectorCommand::Cubic(
                    37.217391, 50.298463, 50.298463, 37.217391, 66.434783, 37.217391,
                ),
                VectorCommand::Cubic(
                    73.590449, 37.221143, 80.495944, 39.850719, 85.841704, 44.607443,
                ),
                VectorCommand::Cubic(80.299534, 38.361902, 72.349995, 34.786041, 64.0, 34.782609),
                VectorCommand::Line(64.0, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_USB: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x2a465f),
                bottom: Color::from_hex(0x3b6080),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 66.434783),
                VectorCommand::Cubic(
                    32.347826, 70.469217, 35.617739, 73.73913, 39.652174, 73.73913,
                ),
                VectorCommand::Cubic(42.744348, 73.73913, 45.498087, 71.784, 46.528, 68.869565),
                VectorCommand::Line(55.478261, 68.869565),
                VectorCommand::Line(64.77913, 80.050087),
                VectorCommand::Cubic(
                    65.149217, 80.593043, 65.79687, 81.043478, 66.434783, 81.043478,
                ),
                VectorCommand::Line(71.28487, 81.043478),
                VectorCommand::Line(71.304348, 83.478261),
                VectorCommand::Line(81.043478, 83.478261),
                VectorCommand::Line(81.043478, 73.73913),
                VectorCommand::Line(71.304348, 73.73913),
                VectorCommand::Line(71.304348, 76.173913),
                VectorCommand::Line(67.652174, 76.173913),
                VectorCommand::Line(61.565217, 68.869565),
                VectorCommand::Line(85.913043, 68.869565),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Line(98.086957, 66.434783),
                VectorCommand::Line(85.913043, 59.130435),
                VectorCommand::Line(85.913043, 64.0),
                VectorCommand::Line(55.478261, 64.0),
                VectorCommand::Line(61.565217, 56.695652),
                VectorCommand::Line(67.094609, 56.695652),
                VectorCommand::Cubic(
                    67.963826, 58.200348, 69.565913, 59.130435, 71.304348, 59.130435,
                ),
                VectorCommand::Cubic(
                    73.994783, 59.13287, 76.173913, 56.951304, 76.173913, 54.26087,
                ),
                VectorCommand::Cubic(
                    76.173913, 51.570435, 73.994783, 49.391304, 71.304348, 49.391304,
                ),
                VectorCommand::Cubic(
                    69.565913, 49.391304, 67.961391, 50.321391, 67.092174, 51.826087,
                ),
                VectorCommand::Line(60.347826, 51.826087),
                VectorCommand::Cubic(
                    59.74887, 51.826087, 59.084174, 52.176696, 58.692174, 52.653913,
                ),
                VectorCommand::Line(49.391304, 64.0),
                VectorCommand::Line(46.53287, 64.0),
                VectorCommand::Cubic(
                    45.500522, 61.08313, 42.744348, 59.130435, 39.652174, 59.130435,
                ),
                VectorCommand::Cubic(
                    35.617739, 59.128, 32.347826, 62.400348, 32.347826, 66.434783,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(29.913043, 64.0),
                VectorCommand::Cubic(
                    29.913043, 68.034435, 33.182957, 71.304348, 37.217391, 71.304348,
                ),
                VectorCommand::Cubic(
                    40.309565, 71.304348, 43.063304, 69.349217, 44.093217, 66.434783,
                ),
                VectorCommand::Line(53.043478, 66.434783),
                VectorCommand::Line(62.344348, 77.615304),
                VectorCommand::Cubic(62.714435, 78.158261, 63.362087, 78.608696, 64.0, 78.608696),
                VectorCommand::Line(68.850087, 78.608696),
                VectorCommand::Line(68.869565, 81.043478),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Line(78.608696, 71.304348),
                VectorCommand::Line(68.869565, 71.304348),
                VectorCommand::Line(68.869565, 73.73913),
                VectorCommand::Line(65.217391, 73.73913),
                VectorCommand::Line(59.130435, 66.434783),
                VectorCommand::Line(83.478261, 66.434783),
                VectorCommand::Line(83.478261, 71.304348),
                VectorCommand::Line(95.652174, 64.0),
                VectorCommand::Line(83.478261, 56.695652),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Line(53.043478, 61.565217),
                VectorCommand::Line(59.130435, 54.26087),
                VectorCommand::Line(64.659826, 54.26087),
                VectorCommand::Cubic(
                    65.529043, 55.765565, 67.13113, 56.695652, 68.869565, 56.695652,
                ),
                VectorCommand::Cubic(71.56, 56.698087, 73.73913, 54.516522, 73.73913, 51.826087),
                VectorCommand::Cubic(73.73913, 49.135652, 71.56, 46.956522, 68.869565, 46.956522),
                VectorCommand::Cubic(
                    67.13113, 46.956522, 65.526609, 47.886609, 64.657391, 49.391304,
                ),
                VectorCommand::Line(57.913043, 49.391304),
                VectorCommand::Cubic(
                    57.314087, 49.391304, 56.649391, 49.741913, 56.257391, 50.21913,
                ),
                VectorCommand::Line(46.956522, 61.565217),
                VectorCommand::Line(44.098087, 61.565217),
                VectorCommand::Cubic(
                    43.065739, 58.648348, 40.309565, 56.695652, 37.217391, 56.695652,
                ),
                VectorCommand::Cubic(33.182957, 56.693217, 29.913043, 59.965565, 29.913043, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(179, 213, 244, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_BATTERY: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x234941),
                bottom: Color::from_hex(0x326055),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.565217, 32.347826),
                VectorCommand::Cubic(
                    60.216348, 32.347826, 59.130435, 33.433739, 59.130435, 34.782609,
                ),
                VectorCommand::Line(59.130435, 37.217391),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Cubic(
                    49.128348, 37.217391, 46.956522, 39.389217, 46.956522, 42.086957,
                ),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Cubic(
                    46.956522, 98.349913, 49.128348, 100.521739, 51.826087, 100.521739,
                ),
                VectorCommand::Line(81.043478, 100.521739),
                VectorCommand::Cubic(
                    83.741217, 100.521739, 85.913043, 98.349913, 85.913043, 95.652174,
                ),
                VectorCommand::Line(85.913043, 42.086957),
                VectorCommand::Cubic(
                    85.913043, 39.389217, 83.741217, 37.217391, 81.043478, 37.217391,
                ),
                VectorCommand::Line(73.73913, 37.217391),
                VectorCommand::Line(73.73913, 34.782609),
                VectorCommand::Cubic(
                    73.73913, 33.433739, 72.653217, 32.347826, 71.304348, 32.347826,
                ),
                VectorCommand::Line(61.565217, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.130435, 29.913043),
                VectorCommand::Cubic(
                    57.781565, 29.913043, 56.695652, 30.998957, 56.695652, 32.347826,
                ),
                VectorCommand::Line(56.695652, 34.782609),
                VectorCommand::Line(49.391304, 34.782609),
                VectorCommand::Cubic(
                    46.693565, 34.782609, 44.521739, 36.954435, 44.521739, 39.652174,
                ),
                VectorCommand::Line(44.521739, 93.217391),
                VectorCommand::Cubic(
                    44.521739, 95.91513, 46.693565, 98.086957, 49.391304, 98.086957,
                ),
                VectorCommand::Line(78.608696, 98.086957),
                VectorCommand::Cubic(
                    81.306435, 98.086957, 83.478261, 95.91513, 83.478261, 93.217391,
                ),
                VectorCommand::Line(83.478261, 39.652174),
                VectorCommand::Cubic(
                    83.478261, 36.954435, 81.306435, 34.782609, 78.608696, 34.782609,
                ),
                VectorCommand::Line(71.304348, 34.782609),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Cubic(
                    71.304348, 30.998957, 70.218435, 29.913043, 68.869565, 29.913043,
                ),
                VectorCommand::Line(59.130435, 29.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(223, 233, 217, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(49.391304, 78.608696),
                VectorCommand::Line(49.391304, 42.086957),
                VectorCommand::Cubic(
                    49.391304, 40.738087, 50.477217, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Cubic(
                    77.522783, 39.652174, 78.608696, 40.738087, 78.608696, 42.086957,
                ),
                VectorCommand::Line(78.608696, 49.391304),
                VectorCommand::Line(64.0, 68.869565),
                VectorCommand::Line(49.391304, 78.608696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(88, 119, 98, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(78.608696, 49.391304),
                VectorCommand::Line(78.608696, 90.782609),
                VectorCommand::Cubic(
                    78.608696, 92.131478, 77.522783, 93.217391, 76.173913, 93.217391,
                ),
                VectorCommand::Line(51.826087, 93.217391),
                VectorCommand::Cubic(
                    50.477217, 93.217391, 49.391304, 92.131478, 49.391304, 90.782609,
                ),
                VectorCommand::Line(49.391304, 78.608696),
                VectorCommand::Line(78.608696, 49.391304),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(158, 191, 153, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_BRIGHTNESS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 53.565217,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 117.565169,
                y2: 10.434929,
                top: Color::from_hex(0xd2dfe6),
                bottom: Color::from_hex(0xe5edf1),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.940522, 86.122435),
                VectorCommand::Cubic(
                    40.051235, 80.79367, 37.2208, 73.805357, 37.217635, 66.435026,
                ),
                VectorCommand::Cubic(
                    37.216386, 63.369148, 37.710849, 60.323235, 38.663287, 57.409287,
                ),
                VectorCommand::Cubic(
                    42.571843, 45.375617, 53.782557, 37.216174, 66.434417, 37.217635,
                ),
                VectorCommand::Cubic(
                    73.790139, 37.218565, 80.644052, 40.003026, 85.869826, 44.679026,
                ),
                VectorCommand::Cubic(
                    74.93473, 27.557148, 41.724783, 19.863722, 29.913652, 32.34807,
                ),
                VectorCommand::Cubic(
                    18.101791, 44.832904, 27.688115, 77.496243, 44.940887, 86.122678,
                ),
                VectorCommand::Line(44.940522, 86.122435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(39.114087, 8.0, 18.031304, 24.236348, 10.73913, 46.69113),
                VectorCommand::Cubic(
                    12.869565, 54.26087, 27.478261, 56.695652, 36.22887, 54.974991,
                ),
                VectorCommand::Cubic(40.137446, 42.941302, 51.347467, 34.790794, 64.0, 34.783339),
                VectorCommand::Cubic(68.869565, 29.913043, 68.869565, 12.869565, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(39, 59, 80, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Line(64.0, 34.782609),
                VectorCommand::Cubic(
                    76.647895, 34.799896, 87.848625, 42.953133, 91.751652, 54.983757,
                ),
                VectorCommand::Cubic(
                    98.086957, 59.130435, 112.695652, 54.26087, 117.26087, 46.695026,
                ),
                VectorCommand::Cubic(109.969426, 24.238783, 88.888348, 7.999026, 64.0, 7.999026),
                VectorCommand::Line(64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(69, 96, 120, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(117.26087, 46.696),
                VectorCommand::Line(91.751652, 54.98473),
                VectorCommand::Cubic(
                    92.709569, 57.894783, 93.20383, 60.937287, 93.21632, 64.000974,
                ),
                VectorCommand::Cubic(
                    93.202929, 73.347861, 88.718546, 82.124522, 81.151729, 87.611791,
                ),
                VectorCommand::Cubic(
                    78.608598, 95.652174, 88.347729, 110.26087, 96.911346, 109.301322,
                ),
                VectorCommand::Cubic(
                    110.899659, 99.120522, 119.998929, 82.625843, 119.998929, 64.002191,
                ),
                VectorCommand::Cubic(
                    119.998929, 57.960765, 119.029788, 52.148452, 117.259798, 46.697217,
                ),
                VectorCommand::Line(117.26087, 46.696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(102, 132, 156, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.842087, 87.610087),
                VectorCommand::Cubic(
                    37.217391, 88.347826, 29.913043, 102.956522, 31.087339, 109.294748,
                ),
                VectorCommand::Cubic(
                    40.327583, 116.020591, 51.695583, 119.99927, 64.00073, 119.99927,
                ),
                VectorCommand::Cubic(
                    76.30393, 119.99927, 87.673148, 116.024487, 96.914122, 109.299617,
                ),
                VectorCommand::Line(81.154504, 87.610087),
                VectorCommand::Cubic(
                    76.171722, 91.242052, 70.167548, 93.204487, 64.001704, 93.216661,
                ),
                VectorCommand::Cubic(
                    57.834157, 93.205704, 51.828035, 91.24305, 46.844035, 87.610087,
                ),
                VectorCommand::Line(46.842087, 87.610087),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(141, 168, 189, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.73913, 46.69113),
                VectorCommand::Cubic(8.96824, 52.143583, 8.0, 57.957843, 8.0, 64.000974),
                VectorCommand::Cubic(8.0, 82.625843, 17.100487, 99.115409, 31.087583, 109.29767),
                VectorCommand::Line(46.84233, 87.613009),
                VectorCommand::Cubic(
                    39.277456, 82.124578, 34.794795, 73.348316, 34.782609, 64.002191,
                ),
                VectorCommand::Cubic(
                    34.78797, 60.936398, 35.275823, 57.890552, 36.228261, 54.976452,
                ),
                VectorCommand::Line(10.73913, 46.69113),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(181, 198, 212, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_DARK_AUDIO: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 8))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 33))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0x584533),
                bottom: Color::from_hex(0x735a40),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782334, 93.216534),
                VectorCommand::Cubic(
                    34.782334, 95.905968, 36.962344, 98.086051, 39.651851, 98.086051,
                ),
                VectorCommand::Cubic(
                    40.351625, 98.086051, 41.009692, 97.928571, 41.611077, 97.662817,
                ),
                VectorCommand::Cubic(
                    58.159888, 104.692939, 74.709185, 104.696104, 91.25337, 97.667687,
                ),
                VectorCommand::Cubic(
                    91.854195, 97.932856, 92.518373, 98.086173, 93.217343, 98.086173,
                ),
                VectorCommand::Cubic(
                    95.906777, 98.086173, 98.08686, 95.906139, 98.08686, 93.216656,
                ),
                VectorCommand::Cubic(
                    98.08686, 92.447686, 97.890667, 91.728117, 97.573272, 91.081494,
                ),
                VectorCommand::Cubic(
                    104.735115, 73.120524, 104.735115, 59.748586, 97.573272, 41.787372,
                ),
                VectorCommand::Cubic(
                    97.891812, 41.139799, 98.08686, 40.422666, 98.08686, 39.652211,
                ),
                VectorCommand::Cubic(
                    98.08686, 36.962776, 95.906851, 34.782694, 93.217343, 34.782694,
                ),
                VectorCommand::Cubic(
                    93.214178, 34.783112, 34.783138, 93.216899, 34.783138, 93.216899,
                ),
                VectorCommand::Line(34.782334, 93.216534),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 34.782609),
                VectorCommand::Cubic(
                    25.042261, 56.696139, 25.042261, 71.301913, 34.782609, 93.214957,
                ),
                VectorCommand::Cubic(
                    54.263061, 102.955304, 73.744, 102.955304, 93.214957, 93.214957,
                ),
                VectorCommand::Cubic(
                    102.955304, 71.301426, 102.955304, 56.695652, 93.214957, 34.782609,
                ),
                VectorCommand::Cubic(
                    73.734504, 25.042261, 54.253565, 25.042261, 34.782609, 34.782609,
                ),
                VectorCommand::Line(34.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 229, 224, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 31.652174,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(107, 131, 139, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 24.347826,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 186, 122, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 37.214956,
                y: 37.217391,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 229, 224, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 37.214956,
                y: 90.782609,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 229, 224, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 90.780174,
                y: 90.782609,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 229, 224, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 90.780174,
                y: 37.217391,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(226, 229, 224, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 17.043478,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(51, 75, 93, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 61.562782,
                y: 59.130435,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(243, 233, 212, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.032348, 34.787478),
                VectorCommand::Cubic(
                    35.760893, 34.882144, 34.777753, 35.942032, 34.778275, 37.217513,
                ),
                VectorCommand::Cubic(
                    34.778275, 38.562744, 35.868364, 39.65327, 37.213057, 39.65327,
                ),
                VectorCommand::Cubic(
                    38.557751, 39.65327, 39.64784, 38.562744, 39.64784, 37.217513,
                ),
                VectorCommand::Cubic(
                    39.648116, 36.539936, 39.366252, 35.892865, 38.869968, 35.431763,
                ),
                VectorCommand::Cubic(
                    38.373684, 34.97066, 37.707836, 34.737202, 37.032397, 34.787478,
                ),
                VectorCommand::Line(37.032348, 34.787478),
                VectorCommand::Close,
                VectorCommand::Move(90.597565, 34.787478),
                VectorCommand::Cubic(
                    89.32611, 34.882144, 88.342971, 35.942032, 88.343492, 37.217513,
                ),
                VectorCommand::Cubic(
                    88.343492, 38.562744, 89.433581, 39.65327, 90.778275, 39.65327,
                ),
                VectorCommand::Cubic(
                    92.122968, 39.65327, 93.213057, 38.562744, 93.213057, 37.217513,
                ),
                VectorCommand::Cubic(
                    93.213333, 36.539936, 92.93147, 35.892865, 92.435186, 35.431763,
                ),
                VectorCommand::Cubic(
                    91.938901, 34.97066, 91.273053, 34.737202, 90.597614, 34.787478,
                ),
                VectorCommand::Line(90.597565, 34.787478),
                VectorCommand::Close,
                VectorCommand::Move(37.032348, 88.352696),
                VectorCommand::Cubic(
                    35.760893, 88.447362, 34.777753, 89.507249, 34.778275, 90.78273,
                ),
                VectorCommand::Cubic(
                    34.778275, 92.127962, 35.868364, 93.218487, 37.213057, 93.218487,
                ),
                VectorCommand::Cubic(
                    38.557751, 93.218487, 39.64784, 92.127962, 39.64784, 90.78273,
                ),
                VectorCommand::Cubic(
                    39.648116, 90.105153, 39.366252, 89.458083, 38.869968, 88.99698,
                ),
                VectorCommand::Cubic(
                    38.373684, 88.535877, 37.707836, 88.302419, 37.032397, 88.352696,
                ),
                VectorCommand::Line(37.032348, 88.352696),
                VectorCommand::Close,
                VectorCommand::Move(90.597565, 88.352696),
                VectorCommand::Cubic(
                    89.32611, 88.447362, 88.342971, 89.507249, 88.343492, 90.78273,
                ),
                VectorCommand::Cubic(
                    88.343492, 92.127962, 89.433581, 93.218487, 90.778275, 93.218487,
                ),
                VectorCommand::Cubic(
                    92.122968, 93.218487, 93.213057, 92.127962, 93.213057, 90.78273,
                ),
                VectorCommand::Cubic(
                    93.213333, 90.105153, 92.93147, 89.458083, 92.435186, 88.99698,
                ),
                VectorCommand::Cubic(
                    91.938901, 88.535877, 91.273053, 88.302419, 90.597614, 88.352696,
                ),
                VectorCommand::Line(90.597565, 88.352696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(153, 176, 183, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(212, 231, 239, 36))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_CLOCK: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xc8e5dc),
                bottom: Color::from_hex(0xd8eee6),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 66.434782,
                y: 66.434782,
                radius: 38.956522,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 38.956522,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 254, 249, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_TIMER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xeed1d5),
                bottom: Color::from_hex(0xf7e3e5),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 37.217391),
                VectorCommand::Cubic(
                    48.977391, 37.217391, 34.782609, 51.424348, 34.782609, 68.869565,
                ),
                VectorCommand::Cubic(
                    34.782609, 86.324522, 48.982261, 100.519304, 66.434783, 100.519304,
                ),
                VectorCommand::Cubic(
                    83.709565, 100.519304, 97.738783, 86.604522, 98.030957, 69.400348,
                ),
                VectorCommand::Cubic(
                    98.068377, 69.225121, 98.086341, 69.046299, 98.084522, 68.86713,
                ),
                VectorCommand::Cubic(
                    98.086883, 68.685523, 98.068916, 68.504221, 98.030957, 68.326609,
                ),
                VectorCommand::Cubic(
                    97.733913, 51.137043, 83.704696, 37.214957, 66.434783, 37.214957,
                ),
                VectorCommand::Move(66.434783, 42.084522),
                VectorCommand::Cubic(
                    81.257739, 42.084522, 93.217391, 54.053913, 93.217391, 68.86713,
                ),
                VectorCommand::Cubic(
                    93.217391, 83.692522, 81.262609, 95.649739, 66.434783, 95.649739,
                ),
                VectorCommand::Cubic(
                    51.609391, 95.649739, 39.652174, 83.694957, 39.652174, 68.86713,
                ),
                VectorCommand::Cubic(
                    39.652174, 54.051478, 51.611826, 42.084522, 66.434783, 42.084522,
                ),
                VectorCommand::Line(66.434783, 42.084522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.398261, 50.993391),
                VectorCommand::Cubic(65.054467, 51.013417, 63.981036, 52.118452, 64.0, 53.462261),
                VectorCommand::Line(64.0, 69.144696),
                VectorCommand::Line(77.834435, 83.198261),
                VectorCommand::Cubic(
                    78.441238, 83.834047, 79.343589, 84.093462, 80.195401, 83.877011,
                ),
                VectorCommand::Cubic(
                    81.047213, 83.66056, 81.71626, 83.001843, 81.945937, 82.153501,
                ),
                VectorCommand::Cubic(82.175614, 81.30516, 81.930269, 80.398882, 81.304, 79.782261),
                VectorCommand::Line(68.869565, 67.150609),
                VectorCommand::Line(68.869565, 53.462261),
                VectorCommand::Cubic(
                    68.878853, 52.804249, 68.621388, 52.170499, 68.155828, 51.705397,
                ),
                VectorCommand::Cubic(
                    67.690267, 51.240295, 67.056264, 50.983455, 66.398261, 50.993391,
                ),
                VectorCommand::Line(66.398261, 50.993391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.570435, 50.104696),
                VectorCommand::Line(47.833043, 46.832348),
                VectorCommand::Line(42.125913, 40.954783),
                VectorCommand::Line(38.853565, 44.387826),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(54.26087, 34.782609),
                VectorCommand::Cubic(
                    60.004522, 31.938783, 70.296348, 31.834087, 76.173913, 34.782609,
                ),
                VectorCommand::Cubic(
                    67.272348, 25.043478, 63.077217, 25.043478, 54.26087, 34.782609,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(63.998812, 34.782609),
                VectorCommand::Cubic(
                    46.541421, 34.782609, 32.346638, 48.989565, 32.346638, 66.434783,
                ),
                VectorCommand::Cubic(
                    32.346638, 83.889739, 46.54629, 98.084522, 63.998812, 98.084522,
                ),
                VectorCommand::Cubic(
                    81.273595, 98.084522, 95.302812, 84.169739, 95.594986, 66.965565,
                ),
                VectorCommand::Cubic(
                    95.632407, 66.790338, 95.65037, 66.611516, 95.648551, 66.432348,
                ),
                VectorCommand::Cubic(
                    95.650912, 66.25074, 95.632945, 66.069438, 95.594986, 65.891826,
                ),
                VectorCommand::Cubic(
                    95.297942, 48.702261, 81.268725, 34.780174, 63.998812, 34.780174,
                ),
                VectorCommand::Move(63.998812, 39.649739),
                VectorCommand::Cubic(
                    78.821769, 39.649739, 90.781421, 51.61913, 90.781421, 66.432348,
                ),
                VectorCommand::Cubic(
                    90.781421, 81.257739, 78.826638, 93.214957, 63.998812, 93.214957,
                ),
                VectorCommand::Cubic(
                    49.173421, 93.214957, 37.216203, 81.260174, 37.216203, 66.432348,
                ),
                VectorCommand::Cubic(
                    37.216203, 51.616696, 49.175855, 39.649739, 63.998812, 39.649739,
                ),
                VectorCommand::Line(63.998812, 39.649739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(147, 78, 96, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(63.96229, 48.558609),
                VectorCommand::Cubic(
                    62.618497, 48.578634, 61.545065, 49.683669, 61.564029, 51.027478,
                ),
                VectorCommand::Line(61.564029, 66.709913),
                VectorCommand::Line(75.398464, 80.763478),
                VectorCommand::Cubic(
                    76.005268, 81.399264, 76.907618, 81.65868, 77.75943, 81.442229,
                ),
                VectorCommand::Cubic(
                    78.611242, 81.225778, 79.280289, 80.56706, 79.509966, 79.718719,
                ),
                VectorCommand::Cubic(
                    79.739644, 78.870377, 79.494298, 77.9641, 78.868029, 77.347478,
                ),
                VectorCommand::Line(66.433595, 64.715826),
                VectorCommand::Line(66.433595, 51.027478),
                VectorCommand::Cubic(
                    66.442883, 50.369466, 66.185418, 49.735716, 65.719857, 49.270614,
                ),
                VectorCommand::Cubic(
                    65.254297, 48.805512, 64.620293, 48.548672, 63.96229, 48.558609,
                ),
                VectorCommand::Line(63.96229, 48.558609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(147, 78, 96, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.134464, 47.669913),
                VectorCommand::Line(45.397073, 44.397565),
                VectorCommand::Line(39.689942, 38.52),
                VectorCommand::Line(36.417595, 41.953043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(147, 78, 96, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.824899, 32.347826),
                VectorCommand::Cubic(
                    57.568551, 29.504, 67.860377, 29.399304, 73.737942, 32.347826,
                ),
                VectorCommand::Cubic(
                    64.836377, 22.608696, 60.641247, 22.608696, 51.824899, 32.347826,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(147, 78, 96, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_NOTES: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xeddfbb),
                bottom: Color::from_hex(0xf7ecca),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 37.217391),
                VectorCommand::Line(95.652174, 37.217391),
                VectorCommand::Line(95.652174, 95.652174),
                VectorCommand::Line(37.217391, 95.652174),
                VectorCommand::Line(37.217391, 37.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 34.782609),
                VectorCommand::Line(93.217391, 34.782609),
                VectorCommand::Line(93.217391, 93.217391),
                VectorCommand::Line(34.782609, 93.217391),
                VectorCommand::Line(34.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(228, 187, 93, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Rect {
                x: 34.782609,
                y: 34.782609,
                width: 58.434783,
                height: 9.73913,
                radius: 0.0,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.086957, 51.826087),
                VectorCommand::Line(42.086957, 56.695652),
                VectorCommand::Line(46.956522, 56.695652),
                VectorCommand::Line(46.956522, 51.826087),
                VectorCommand::Line(42.086957, 51.826087),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 51.826087),
                VectorCommand::Line(54.26087, 56.695652),
                VectorCommand::Line(85.913043, 56.695652),
                VectorCommand::Line(85.913043, 51.826087),
                VectorCommand::Line(54.26087, 51.826087),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 66.434783),
                VectorCommand::Line(42.086957, 71.304348),
                VectorCommand::Line(46.956522, 71.304348),
                VectorCommand::Line(46.956522, 66.434783),
                VectorCommand::Line(42.086957, 66.434783),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 66.434783),
                VectorCommand::Line(54.26087, 71.304348),
                VectorCommand::Line(85.913043, 71.304348),
                VectorCommand::Line(85.913043, 66.434783),
                VectorCommand::Line(54.26087, 66.434783),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 81.043478),
                VectorCommand::Line(42.086957, 85.913043),
                VectorCommand::Line(46.956522, 85.913043),
                VectorCommand::Line(46.956522, 81.043478),
                VectorCommand::Line(42.086957, 81.043478),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 81.043478),
                VectorCommand::Line(54.26087, 85.913043),
                VectorCommand::Line(85.913043, 85.913043),
                VectorCommand::Line(85.913043, 81.043478),
                VectorCommand::Line(54.26087, 81.043478),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(95, 73, 42, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_CALCULATOR: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(33.070957, 8.0, 8.0, 33.070957, 8.0, 64.0),
                VectorCommand::Line(81.043478, 81.043478),
                VectorCommand::Line(64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(159, 207, 215, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Line(90.782609, 71.304348),
                VectorCommand::Line(120.0, 64.0),
                VectorCommand::Cubic(120.0, 33.070957, 94.929043, 8.0, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(196, 226, 229, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 64.0),
                VectorCommand::Line(59.130435, 90.782609),
                VectorCommand::Line(64.0, 120.0),
                VectorCommand::Cubic(94.929043, 120.0, 120.0, 94.929043, 120.0, 64.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(237, 204, 142, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(8.0, 64.0),
                VectorCommand::Cubic(8.0, 94.929043, 33.070957, 120.0, 64.0, 120.0),
                VectorCommand::Line(64.0, 64.0),
                VectorCommand::Line(8.0, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(125, 188, 201, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(42.086957, 34.782609),
                VectorCommand::Line(42.086957, 42.086957),
                VectorCommand::Line(34.782609, 42.086957),
                VectorCommand::Line(34.782609, 46.956522),
                VectorCommand::Line(42.086957, 46.956522),
                VectorCommand::Line(42.086957, 54.26087),
                VectorCommand::Line(46.956522, 54.26087),
                VectorCommand::Line(46.956522, 46.956522),
                VectorCommand::Line(54.26087, 46.956522),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Line(46.956522, 42.086957),
                VectorCommand::Line(46.956522, 34.782609),
                VectorCommand::Line(42.086957, 34.782609),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 42.086957),
                VectorCommand::Line(78.608696, 46.956522),
                VectorCommand::Line(98.086957, 46.956522),
                VectorCommand::Line(98.086957, 42.086957),
                VectorCommand::Line(78.608696, 42.086957),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 78.608696),
                VectorCommand::Line(42.086957, 83.478261),
                VectorCommand::Line(46.956522, 83.478261),
                VectorCommand::Line(46.956522, 78.608696),
                VectorCommand::Line(42.086957, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 81.043478),
                VectorCommand::Line(78.608696, 85.913043),
                VectorCommand::Line(98.086957, 85.913043),
                VectorCommand::Line(98.086957, 81.043478),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Close,
                VectorCommand::Move(34.782609, 85.913043),
                VectorCommand::Line(34.782609, 90.782609),
                VectorCommand::Line(54.26087, 90.782609),
                VectorCommand::Line(54.26087, 85.913043),
                VectorCommand::Line(34.782609, 85.913043),
                VectorCommand::Close,
                VectorCommand::Move(78.608696, 90.782609),
                VectorCommand::Line(78.608696, 95.652174),
                VectorCommand::Line(98.086957, 95.652174),
                VectorCommand::Line(98.086957, 90.782609),
                VectorCommand::Line(78.608696, 90.782609),
                VectorCommand::Close,
                VectorCommand::Move(42.086957, 93.217391),
                VectorCommand::Line(42.086957, 98.086957),
                VectorCommand::Line(46.956522, 98.086957),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Line(42.086957, 93.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Line(39.652174, 39.652174),
                VectorCommand::Line(32.347826, 39.652174),
                VectorCommand::Line(32.347826, 44.521739),
                VectorCommand::Line(39.652174, 44.521739),
                VectorCommand::Line(39.652174, 51.826087),
                VectorCommand::Line(44.521739, 51.826087),
                VectorCommand::Line(44.521739, 44.521739),
                VectorCommand::Line(51.826087, 44.521739),
                VectorCommand::Line(51.826087, 39.652174),
                VectorCommand::Line(44.521739, 39.652174),
                VectorCommand::Line(44.521739, 32.347826),
                VectorCommand::Line(39.652174, 32.347826),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 39.652174),
                VectorCommand::Line(76.173913, 44.521739),
                VectorCommand::Line(95.652174, 44.521739),
                VectorCommand::Line(95.652174, 39.652174),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 76.173913),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Line(44.521739, 81.043478),
                VectorCommand::Line(44.521739, 76.173913),
                VectorCommand::Line(39.652174, 76.173913),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 78.608696),
                VectorCommand::Line(76.173913, 83.478261),
                VectorCommand::Line(95.652174, 83.478261),
                VectorCommand::Line(95.652174, 78.608696),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 83.478261),
                VectorCommand::Line(32.347826, 88.347826),
                VectorCommand::Line(51.826087, 88.347826),
                VectorCommand::Line(51.826087, 83.478261),
                VectorCommand::Line(32.347826, 83.478261),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 88.347826),
                VectorCommand::Line(76.173913, 93.217391),
                VectorCommand::Line(95.652174, 93.217391),
                VectorCommand::Line(95.652174, 88.347826),
                VectorCommand::Line(76.173913, 88.347826),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 90.782609),
                VectorCommand::Line(39.652174, 95.652174),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(44.521739, 90.782609),
                VectorCommand::Line(39.652174, 90.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(39, 75, 94, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_MAC: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xd4dde9),
                bottom: Color::from_hex(0xe4eaf2),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 42.086957),
                VectorCommand::Line(32.347826, 49.391304),
                VectorCommand::Cubic(
                    32.347826, 50.742609, 33.441043, 51.826087, 34.782609, 51.826087,
                ),
                VectorCommand::Line(39.652174, 51.826087),
                VectorCommand::Cubic(
                    40.993739, 51.826087, 42.086957, 50.742609, 42.086957, 49.391304,
                ),
                VectorCommand::Line(42.086957, 44.521739),
                VectorCommand::Cubic(
                    42.086957, 43.180174, 40.993739, 42.086957, 39.652174, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 42.086957),
                VectorCommand::Line(46.956522, 49.391304),
                VectorCommand::Cubic(
                    46.956522, 50.742609, 48.049739, 51.826087, 49.391304, 51.826087,
                ),
                VectorCommand::Line(54.26087, 51.826087),
                VectorCommand::Cubic(
                    55.602435, 51.826087, 56.695652, 50.742609, 56.695652, 49.391304,
                ),
                VectorCommand::Line(56.695652, 44.521739),
                VectorCommand::Cubic(
                    56.695652, 43.180174, 55.602435, 42.086957, 54.26087, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 42.086957),
                VectorCommand::Line(61.565217, 49.391304),
                VectorCommand::Cubic(61.565217, 50.742609, 62.658435, 51.826087, 64.0, 51.826087),
                VectorCommand::Line(68.869565, 51.826087),
                VectorCommand::Cubic(
                    70.21113, 51.826087, 71.304348, 50.742609, 71.304348, 49.391304,
                ),
                VectorCommand::Line(71.304348, 44.521739),
                VectorCommand::Cubic(
                    71.304348, 43.180174, 70.21113, 42.086957, 68.869565, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 42.086957),
                VectorCommand::Line(76.173913, 49.391304),
                VectorCommand::Cubic(
                    76.173913, 50.742609, 77.26713, 51.826087, 78.608696, 51.826087,
                ),
                VectorCommand::Line(83.478261, 51.826087),
                VectorCommand::Cubic(
                    84.819826, 51.826087, 85.913043, 50.742609, 85.913043, 49.391304,
                ),
                VectorCommand::Line(85.913043, 44.521739),
                VectorCommand::Cubic(
                    85.913043, 43.180174, 84.819826, 42.086957, 83.478261, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 42.086957),
                VectorCommand::Line(90.782609, 49.391304),
                VectorCommand::Cubic(
                    90.782609, 50.742609, 91.875826, 51.826087, 93.217391, 51.826087,
                ),
                VectorCommand::Line(98.086957, 51.826087),
                VectorCommand::Cubic(
                    99.428522, 51.826087, 100.521739, 50.742609, 100.521739, 49.391304,
                ),
                VectorCommand::Line(100.521739, 44.521739),
                VectorCommand::Cubic(
                    100.521739, 43.180174, 99.428522, 42.086957, 98.086957, 42.086957,
                ),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 56.695652),
                VectorCommand::Line(32.347826, 64.0),
                VectorCommand::Cubic(
                    32.347826, 65.351304, 33.441043, 66.434783, 34.782609, 66.434783,
                ),
                VectorCommand::Line(39.652174, 66.434783),
                VectorCommand::Cubic(40.993739, 66.434783, 42.086957, 65.351304, 42.086957, 64.0),
                VectorCommand::Line(42.086957, 59.130435),
                VectorCommand::Cubic(
                    42.086957, 57.78887, 40.993739, 56.695652, 39.652174, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 56.695652),
                VectorCommand::Line(46.956522, 64.0),
                VectorCommand::Cubic(
                    46.956522, 65.351304, 48.049739, 66.434783, 49.391304, 66.434783,
                ),
                VectorCommand::Line(54.26087, 66.434783),
                VectorCommand::Cubic(55.602435, 66.434783, 56.695652, 65.351304, 56.695652, 64.0),
                VectorCommand::Line(56.695652, 59.130435),
                VectorCommand::Cubic(
                    56.695652, 57.78887, 55.602435, 56.695652, 54.26087, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 56.695652),
                VectorCommand::Line(61.565217, 64.0),
                VectorCommand::Cubic(61.565217, 65.351304, 62.658435, 66.434783, 64.0, 66.434783),
                VectorCommand::Line(68.869565, 66.434783),
                VectorCommand::Cubic(70.21113, 66.434783, 71.304348, 65.351304, 71.304348, 64.0),
                VectorCommand::Line(71.304348, 59.130435),
                VectorCommand::Cubic(
                    71.304348, 57.78887, 70.21113, 56.695652, 68.869565, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 56.695652),
                VectorCommand::Line(76.173913, 64.0),
                VectorCommand::Cubic(
                    76.173913, 65.351304, 77.26713, 66.434783, 78.608696, 66.434783,
                ),
                VectorCommand::Line(83.478261, 66.434783),
                VectorCommand::Cubic(84.819826, 66.434783, 85.913043, 65.351304, 85.913043, 64.0),
                VectorCommand::Line(85.913043, 59.130435),
                VectorCommand::Cubic(
                    85.913043, 57.78887, 84.819826, 56.695652, 83.478261, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 56.695652),
                VectorCommand::Line(90.782609, 64.0),
                VectorCommand::Cubic(
                    90.782609, 65.351304, 91.875826, 66.434783, 93.217391, 66.434783,
                ),
                VectorCommand::Line(98.086957, 66.434783),
                VectorCommand::Cubic(
                    99.428522, 66.434783, 100.521739, 65.351304, 100.521739, 64.0,
                ),
                VectorCommand::Line(100.521739, 59.130435),
                VectorCommand::Cubic(
                    100.521739, 57.78887, 99.428522, 56.695652, 98.086957, 56.695652,
                ),
                VectorCommand::Close,
                VectorCommand::Move(39.652174, 71.304348),
                VectorCommand::Line(32.347826, 78.608696),
                VectorCommand::Cubic(32.347826, 79.96, 33.441043, 81.043478, 34.782609, 81.043478),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Cubic(40.993739, 81.043478, 42.086957, 79.96, 42.086957, 78.608696),
                VectorCommand::Line(42.086957, 73.73913),
                VectorCommand::Cubic(
                    42.086957, 72.397565, 40.993739, 71.304348, 39.652174, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(54.26087, 71.304348),
                VectorCommand::Line(46.956522, 78.608696),
                VectorCommand::Cubic(46.956522, 79.96, 48.049739, 81.043478, 49.391304, 81.043478),
                VectorCommand::Line(54.26087, 81.043478),
                VectorCommand::Cubic(55.602435, 81.043478, 56.695652, 79.96, 56.695652, 78.608696),
                VectorCommand::Line(56.695652, 73.73913),
                VectorCommand::Cubic(
                    56.695652, 72.397565, 55.602435, 71.304348, 54.26087, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(68.869565, 71.304348),
                VectorCommand::Line(61.565217, 78.608696),
                VectorCommand::Cubic(61.565217, 79.96, 62.658435, 81.043478, 64.0, 81.043478),
                VectorCommand::Line(68.869565, 81.043478),
                VectorCommand::Cubic(70.21113, 81.043478, 71.304348, 79.96, 71.304348, 78.608696),
                VectorCommand::Line(71.304348, 73.73913),
                VectorCommand::Cubic(
                    71.304348, 72.397565, 70.21113, 71.304348, 68.869565, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 71.304348),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Cubic(76.173913, 79.96, 77.26713, 81.043478, 78.608696, 81.043478),
                VectorCommand::Line(83.478261, 81.043478),
                VectorCommand::Cubic(84.819826, 81.043478, 85.913043, 79.96, 85.913043, 78.608696),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Cubic(
                    85.913043, 72.397565, 84.819826, 71.304348, 83.478261, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(98.086957, 71.304348),
                VectorCommand::Line(90.782609, 78.608696),
                VectorCommand::Cubic(90.782609, 79.96, 91.875826, 81.043478, 93.217391, 81.043478),
                VectorCommand::Line(98.086957, 81.043478),
                VectorCommand::Cubic(
                    99.428522, 81.043478, 100.521739, 79.96, 100.521739, 78.608696,
                ),
                VectorCommand::Line(100.521739, 73.73913),
                VectorCommand::Cubic(
                    100.521739, 72.397565, 99.428522, 71.304348, 98.086957, 71.304348,
                ),
                VectorCommand::Close,
                VectorCommand::Move(83.478261, 85.913043),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Cubic(
                    46.956522, 94.568696, 48.049739, 95.652174, 49.391304, 95.652174,
                ),
                VectorCommand::Line(83.478261, 95.661694),
                VectorCommand::Cubic(
                    84.819826, 95.662069, 85.913043, 94.578216, 85.913043, 93.226911,
                ),
                VectorCommand::Line(85.913043, 88.357346),
                VectorCommand::Cubic(
                    85.913043, 87.015781, 84.819826, 85.922563, 83.478261, 85.922563,
                ),
                VectorCommand::Line(83.478261, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 39.652174),
                VectorCommand::Cubic(
                    31.006261, 39.652174, 29.913043, 40.745391, 29.913043, 42.086957,
                ),
                VectorCommand::Line(29.913043, 46.956522),
                VectorCommand::Cubic(
                    29.913043, 48.307826, 31.006261, 49.391304, 32.347826, 49.391304,
                ),
                VectorCommand::Line(37.217391, 49.391304),
                VectorCommand::Cubic(
                    38.558957, 49.391304, 39.652174, 48.307826, 39.652174, 46.956522,
                ),
                VectorCommand::Line(39.652174, 42.086957),
                VectorCommand::Cubic(
                    39.652174, 40.745391, 38.558957, 39.652174, 37.217391, 39.652174,
                ),
                VectorCommand::Line(32.347826, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 39.652174),
                VectorCommand::Cubic(
                    45.614957, 39.652174, 44.521739, 40.745391, 44.521739, 42.086957,
                ),
                VectorCommand::Line(44.521739, 46.956522),
                VectorCommand::Cubic(
                    44.521739, 48.307826, 45.614957, 49.391304, 46.956522, 49.391304,
                ),
                VectorCommand::Line(51.826087, 49.391304),
                VectorCommand::Cubic(
                    53.167652, 49.391304, 54.26087, 48.307826, 54.26087, 46.956522,
                ),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Cubic(
                    54.26087, 40.745391, 53.167652, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Line(46.956522, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 39.652174),
                VectorCommand::Cubic(
                    60.223652, 39.652174, 59.130435, 40.745391, 59.130435, 42.086957,
                ),
                VectorCommand::Line(59.130435, 46.956522),
                VectorCommand::Cubic(
                    59.130435, 48.307826, 60.223652, 49.391304, 61.565217, 49.391304,
                ),
                VectorCommand::Line(66.434783, 49.391304),
                VectorCommand::Cubic(
                    67.776348, 49.391304, 68.869565, 48.307826, 68.869565, 46.956522,
                ),
                VectorCommand::Line(68.869565, 42.086957),
                VectorCommand::Cubic(
                    68.869565, 40.745391, 67.776348, 39.652174, 66.434783, 39.652174,
                ),
                VectorCommand::Line(61.565217, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 39.652174),
                VectorCommand::Cubic(
                    74.832348, 39.652174, 73.73913, 40.745391, 73.73913, 42.086957,
                ),
                VectorCommand::Line(73.73913, 46.956522),
                VectorCommand::Cubic(
                    73.73913, 48.307826, 74.832348, 49.391304, 76.173913, 49.391304,
                ),
                VectorCommand::Line(81.043478, 49.391304),
                VectorCommand::Cubic(
                    82.385043, 49.391304, 83.478261, 48.307826, 83.478261, 46.956522,
                ),
                VectorCommand::Line(83.478261, 42.086957),
                VectorCommand::Cubic(
                    83.478261, 40.745391, 82.385043, 39.652174, 81.043478, 39.652174,
                ),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 39.652174),
                VectorCommand::Cubic(
                    89.441043, 39.652174, 88.347826, 40.745391, 88.347826, 42.086957,
                ),
                VectorCommand::Line(88.347826, 46.956522),
                VectorCommand::Cubic(
                    88.347826, 48.307826, 89.441043, 49.391304, 90.782609, 49.391304,
                ),
                VectorCommand::Line(95.652174, 49.391304),
                VectorCommand::Cubic(
                    96.993739, 49.391304, 98.086957, 48.307826, 98.086957, 46.956522,
                ),
                VectorCommand::Line(98.086957, 42.086957),
                VectorCommand::Cubic(
                    98.086957, 40.745391, 96.993739, 39.652174, 95.652174, 39.652174,
                ),
                VectorCommand::Line(90.782609, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 54.26087),
                VectorCommand::Cubic(
                    31.006261, 54.26087, 29.913043, 55.354087, 29.913043, 56.695652,
                ),
                VectorCommand::Line(29.913043, 61.565217),
                VectorCommand::Cubic(29.913043, 62.916522, 31.006261, 64.0, 32.347826, 64.0),
                VectorCommand::Line(37.217391, 64.0),
                VectorCommand::Cubic(38.558957, 64.0, 39.652174, 62.916522, 39.652174, 61.565217),
                VectorCommand::Line(39.652174, 56.695652),
                VectorCommand::Cubic(
                    39.652174, 55.354087, 38.558957, 54.26087, 37.217391, 54.26087,
                ),
                VectorCommand::Line(32.347826, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 54.26087),
                VectorCommand::Cubic(
                    45.614957, 54.26087, 44.521739, 55.354087, 44.521739, 56.695652,
                ),
                VectorCommand::Line(44.521739, 61.565217),
                VectorCommand::Cubic(44.521739, 62.916522, 45.614957, 64.0, 46.956522, 64.0),
                VectorCommand::Line(51.826087, 64.0),
                VectorCommand::Cubic(53.167652, 64.0, 54.26087, 62.916522, 54.26087, 61.565217),
                VectorCommand::Line(54.26087, 56.695652),
                VectorCommand::Cubic(
                    54.26087, 55.354087, 53.167652, 54.26087, 51.826087, 54.26087,
                ),
                VectorCommand::Line(46.956522, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 54.26087),
                VectorCommand::Cubic(
                    60.223652, 54.26087, 59.130435, 55.354087, 59.130435, 56.695652,
                ),
                VectorCommand::Line(59.130435, 61.565217),
                VectorCommand::Cubic(59.130435, 62.916522, 60.223652, 64.0, 61.565217, 64.0),
                VectorCommand::Line(66.434783, 64.0),
                VectorCommand::Cubic(67.776348, 64.0, 68.869565, 62.916522, 68.869565, 61.565217),
                VectorCommand::Line(68.869565, 56.695652),
                VectorCommand::Cubic(
                    68.869565, 55.354087, 67.776348, 54.26087, 66.434783, 54.26087,
                ),
                VectorCommand::Line(61.565217, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 54.26087),
                VectorCommand::Cubic(
                    74.832348, 54.26087, 73.73913, 55.354087, 73.73913, 56.695652,
                ),
                VectorCommand::Line(73.73913, 61.565217),
                VectorCommand::Cubic(73.73913, 62.916522, 74.832348, 64.0, 76.173913, 64.0),
                VectorCommand::Line(81.043478, 64.0),
                VectorCommand::Cubic(82.385043, 64.0, 83.478261, 62.916522, 83.478261, 61.565217),
                VectorCommand::Line(83.478261, 56.695652),
                VectorCommand::Cubic(
                    83.478261, 55.354087, 82.385043, 54.26087, 81.043478, 54.26087,
                ),
                VectorCommand::Line(76.173913, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 54.26087),
                VectorCommand::Cubic(
                    89.441043, 54.26087, 88.347826, 55.354087, 88.347826, 56.695652,
                ),
                VectorCommand::Line(88.347826, 61.565217),
                VectorCommand::Cubic(88.347826, 62.916522, 89.441043, 64.0, 90.782609, 64.0),
                VectorCommand::Line(95.652174, 64.0),
                VectorCommand::Cubic(96.993739, 64.0, 98.086957, 62.916522, 98.086957, 61.565217),
                VectorCommand::Line(98.086957, 56.695652),
                VectorCommand::Cubic(
                    98.086957, 55.354087, 96.993739, 54.26087, 95.652174, 54.26087,
                ),
                VectorCommand::Line(90.782609, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(32.347826, 68.869565),
                VectorCommand::Cubic(
                    31.006261, 68.869565, 29.913043, 69.962783, 29.913043, 71.304348,
                ),
                VectorCommand::Line(29.913043, 76.173913),
                VectorCommand::Cubic(
                    29.913043, 77.525217, 31.006261, 78.608696, 32.347826, 78.608696,
                ),
                VectorCommand::Line(37.217391, 78.608696),
                VectorCommand::Cubic(
                    38.558957, 78.608696, 39.652174, 77.525217, 39.652174, 76.173913,
                ),
                VectorCommand::Line(39.652174, 71.304348),
                VectorCommand::Cubic(
                    39.652174, 69.962783, 38.558957, 68.869565, 37.217391, 68.869565,
                ),
                VectorCommand::Line(32.347826, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 68.869565),
                VectorCommand::Cubic(
                    45.614957, 68.869565, 44.521739, 69.962783, 44.521739, 71.304348,
                ),
                VectorCommand::Line(44.521739, 76.173913),
                VectorCommand::Cubic(
                    44.521739, 77.525217, 45.614957, 78.608696, 46.956522, 78.608696,
                ),
                VectorCommand::Line(51.826087, 78.608696),
                VectorCommand::Cubic(
                    53.167652, 78.608696, 54.26087, 77.525217, 54.26087, 76.173913,
                ),
                VectorCommand::Line(54.26087, 71.304348),
                VectorCommand::Cubic(
                    54.26087, 69.962783, 53.167652, 68.869565, 51.826087, 68.869565,
                ),
                VectorCommand::Line(46.956522, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 68.869565),
                VectorCommand::Cubic(
                    60.223652, 68.869565, 59.130435, 69.962783, 59.130435, 71.304348,
                ),
                VectorCommand::Line(59.130435, 76.173913),
                VectorCommand::Cubic(
                    59.130435, 77.525217, 60.223652, 78.608696, 61.565217, 78.608696,
                ),
                VectorCommand::Line(66.434783, 78.608696),
                VectorCommand::Cubic(
                    67.776348, 78.608696, 68.869565, 77.525217, 68.869565, 76.173913,
                ),
                VectorCommand::Line(68.869565, 71.304348),
                VectorCommand::Cubic(
                    68.869565, 69.962783, 67.776348, 68.869565, 66.434783, 68.869565,
                ),
                VectorCommand::Line(61.565217, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 68.869565),
                VectorCommand::Cubic(
                    74.832348, 68.869565, 73.73913, 69.962783, 73.73913, 71.304348,
                ),
                VectorCommand::Line(73.73913, 76.173913),
                VectorCommand::Cubic(
                    73.73913, 77.525217, 74.832348, 78.608696, 76.173913, 78.608696,
                ),
                VectorCommand::Line(81.043478, 78.608696),
                VectorCommand::Cubic(
                    82.385043, 78.608696, 83.478261, 77.525217, 83.478261, 76.173913,
                ),
                VectorCommand::Line(83.478261, 71.304348),
                VectorCommand::Cubic(
                    83.478261, 69.962783, 82.385043, 68.869565, 81.043478, 68.869565,
                ),
                VectorCommand::Line(76.173913, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(90.782609, 68.869565),
                VectorCommand::Cubic(
                    89.441043, 68.869565, 88.347826, 69.962783, 88.347826, 71.304348,
                ),
                VectorCommand::Line(88.347826, 76.173913),
                VectorCommand::Cubic(
                    88.347826, 77.525217, 89.441043, 78.608696, 90.782609, 78.608696,
                ),
                VectorCommand::Line(95.652174, 78.608696),
                VectorCommand::Cubic(
                    96.993739, 78.608696, 98.086957, 77.525217, 98.086957, 76.173913,
                ),
                VectorCommand::Line(98.086957, 71.304348),
                VectorCommand::Cubic(
                    98.086957, 69.962783, 96.993739, 68.869565, 95.652174, 68.869565,
                ),
                VectorCommand::Line(90.782609, 68.869565),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 83.478261),
                VectorCommand::Cubic(
                    45.614957, 83.477886, 44.521739, 84.571478, 44.521739, 85.913043,
                ),
                VectorCommand::Line(44.521739, 90.782609),
                VectorCommand::Cubic(
                    44.521739, 92.133913, 45.614957, 93.217391, 46.956522, 93.217391,
                ),
                VectorCommand::Line(81.043478, 93.226887),
                VectorCommand::Cubic(
                    82.385043, 93.227262, 83.478261, 92.143409, 83.478261, 90.792104,
                ),
                VectorCommand::Line(83.478261, 85.922539),
                VectorCommand::Cubic(
                    83.478261, 84.580974, 82.385043, 83.487757, 81.043478, 83.487757,
                ),
                VectorCommand::Line(46.956522, 83.478261),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(74, 96, 123, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_SETTINGS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xdce5df),
                bottom: Color::from_hex(0xeaf0e8),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 46.956522),
                VectorCommand::Cubic(
                    34.782609, 55.015652, 41.332174, 61.565217, 49.391304, 61.565217,
                ),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Cubic(
                    91.537391, 61.565217, 98.086957, 55.015652, 98.086957, 46.956522,
                ),
                VectorCommand::Cubic(
                    98.086957, 38.897391, 91.537391, 32.347826, 83.478261, 32.347826,
                ),
                VectorCommand::Line(34.782609, 46.956522),
                VectorCommand::Close,
                VectorCommand::Move(34.782609, 85.913043),
                VectorCommand::Cubic(
                    34.782609, 93.972174, 41.332174, 100.521739, 49.391304, 100.521739,
                ),
                VectorCommand::Line(83.478261, 100.521739),
                VectorCommand::Cubic(
                    91.537391, 100.521739, 98.086957, 93.972174, 98.086957, 85.913043,
                ),
                VectorCommand::Cubic(
                    98.086957, 77.853913, 91.537391, 71.304348, 83.478261, 71.304348,
                ),
                VectorCommand::Line(34.782609, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 29.913043),
                VectorCommand::Cubic(
                    38.897391, 29.913043, 32.347826, 36.462609, 32.347826, 44.521739,
                ),
                VectorCommand::Cubic(
                    32.347826, 52.58087, 38.897391, 59.130435, 46.956522, 59.130435,
                ),
                VectorCommand::Line(81.043478, 59.130435),
                VectorCommand::Cubic(
                    89.102609, 59.130435, 95.652174, 52.58087, 95.652174, 44.521739,
                ),
                VectorCommand::Cubic(
                    95.652174, 36.462609, 89.102609, 29.913043, 81.043478, 29.913043,
                ),
                VectorCommand::Line(46.956522, 29.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(59, 136, 97, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 68.869565),
                VectorCommand::Cubic(
                    38.897391, 68.869565, 32.347826, 75.41913, 32.347826, 83.478261,
                ),
                VectorCommand::Cubic(
                    32.347826, 91.537391, 38.897391, 98.086957, 46.956522, 98.086957,
                ),
                VectorCommand::Line(81.043478, 98.086957),
                VectorCommand::Cubic(
                    89.102609, 98.086957, 95.652174, 91.537391, 95.652174, 83.478261,
                ),
                VectorCommand::Cubic(
                    95.652174, 75.41913, 89.102609, 68.869565, 81.043478, 68.869565,
                ),
                VectorCommand::Line(46.956522, 68.869565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(189, 98, 93, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.043478, 34.782609),
                VectorCommand::Cubic(
                    75.662609, 34.782609, 71.304348, 39.14087, 71.304348, 44.521739,
                ),
                VectorCommand::Cubic(
                    71.304348, 49.902609, 75.662609, 54.26087, 81.043478, 54.26087,
                ),
                VectorCommand::Cubic(
                    86.424348, 54.26087, 90.782609, 49.902609, 90.782609, 44.521739,
                ),
                VectorCommand::Cubic(
                    90.782609, 39.14087, 86.424348, 34.782609, 81.043478, 34.782609,
                ),
                VectorCommand::Close,
                VectorCommand::Move(51.826087, 39.652174),
                VectorCommand::Line(45.73913, 45.73913),
                VectorCommand::Line(42.086957, 42.086957),
                VectorCommand::Line(39.652174, 44.521739),
                VectorCommand::Line(45.73913, 50.608696),
                VectorCommand::Line(54.26087, 42.086957),
                VectorCommand::Line(51.826087, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(46.956522, 73.73913),
                VectorCommand::Cubic(
                    41.575652, 73.73913, 37.217391, 78.097391, 37.217391, 83.478261,
                ),
                VectorCommand::Cubic(
                    37.217391, 88.85913, 41.575652, 93.217391, 46.956522, 93.217391,
                ),
                VectorCommand::Cubic(
                    52.337391, 93.217391, 56.695652, 88.85913, 56.695652, 83.478261,
                ),
                VectorCommand::Cubic(
                    56.695652, 78.097391, 52.337391, 73.73913, 46.956522, 73.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(77.391304, 77.391304),
                VectorCommand::Line(74.956522, 79.826087),
                VectorCommand::Line(78.608696, 83.478261),
                VectorCommand::Line(74.956522, 87.130435),
                VectorCommand::Line(77.391304, 89.565217),
                VectorCommand::Line(81.043478, 85.913043),
                VectorCommand::Line(84.695652, 89.565217),
                VectorCommand::Line(87.130435, 87.130435),
                VectorCommand::Line(83.478261, 83.478261),
                VectorCommand::Line(87.130435, 79.826087),
                VectorCommand::Line(84.695652, 77.391304),
                VectorCommand::Line(81.043478, 81.043478),
                VectorCommand::Line(77.391304, 77.391304),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 255, 248, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_DISPLAY: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xd8e1ed),
                bottom: Color::from_hex(0xe9eef6),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.0, 39.652174),
                VectorCommand::Line(34.70713, 41.098435),
                VectorCommand::Line(34.70713, 67.394087),
                VectorCommand::Cubic(
                    34.70713, 68.163478, 35.381565, 68.840348, 36.153391, 68.840348,
                ),
                VectorCommand::Line(49.325565, 68.840348),
                VectorCommand::Line(49.325565, 73.709913),
                VectorCommand::Line(39.586435, 73.709913),
                VectorCommand::Line(39.586435, 78.579478),
                VectorCommand::Line(56.629913, 78.579478),
                VectorCommand::Line(56.629913, 63.970783),
                VectorCommand::Line(40.657739, 63.970783),
                VectorCommand::Cubic(40.144, 63.970783, 39.659478, 63.496, 39.659478, 62.972522),
                VectorCommand::Line(39.659478, 45.490783),
                VectorCommand::Cubic(
                    39.649739, 45.033043, 39.97113, 44.587478, 40.41913, 44.502261,
                ),
                VectorCommand::Cubic(
                    40.47513, 44.492522, 40.579339, 44.492522, 40.647513, 44.502261,
                ),
                VectorCommand::Line(67.6736, 44.502261),
                VectorCommand::Cubic(
                    68.1776, 44.502261, 68.662122, 44.967304, 68.662122, 45.490783,
                ),
                VectorCommand::Line(68.662122, 51.79687),
                VectorCommand::Line(73.60473, 51.79687),
                VectorCommand::Line(73.60473, 41.059478),
                VectorCommand::Cubic(
                    73.60473, 40.297391, 72.920557, 39.613217, 72.15847, 39.613217,
                ),
                VectorCommand::Line(35.880209, 39.613217),
                VectorCommand::Line(36.0, 39.652174),
                VectorCommand::Close,
                VectorCommand::Move(60.347826, 54.26087),
                VectorCommand::Line(59.035478, 55.71687),
                VectorCommand::Line(59.035478, 82.012522),
                VectorCommand::Cubic(
                    59.035478, 82.774609, 59.73913, 83.473391, 60.501217, 83.473391,
                ),
                VectorCommand::Line(73.649043, 83.473391),
                VectorCommand::Line(73.649043, 88.318609),
                VectorCommand::Line(63.909913, 88.318609),
                VectorCommand::Line(63.909913, 93.188174),
                VectorCommand::Line(93.127304, 93.188174),
                VectorCommand::Line(93.127304, 88.318609),
                VectorCommand::Line(83.388174, 88.318609),
                VectorCommand::Line(83.388174, 83.473391),
                VectorCommand::Line(96.536, 83.473391),
                VectorCommand::Cubic(97.298087, 83.473391, 97.992, 82.774609, 97.992, 82.012522),
                VectorCommand::Line(97.992, 55.71687),
                VectorCommand::Cubic(97.992, 54.947478, 97.288348, 54.26087, 96.536, 54.26087),
                VectorCommand::Line(60.257739, 54.26087),
                VectorCommand::Line(60.347826, 54.26087),
                VectorCommand::Close,
                VectorCommand::Move(64.754783, 59.130435),
                VectorCommand::Cubic(
                    64.810783, 59.120696, 64.869704, 59.120696, 64.935443, 59.130435,
                ),
                VectorCommand::Line(91.96153, 59.130435),
                VectorCommand::Cubic(
                    92.47527, 59.130435, 92.940313, 59.595478, 92.940313, 60.109217,
                ),
                VectorCommand::Line(92.940313, 77.639652),
                VectorCommand::Cubic(
                    92.940313, 78.143652, 92.47527, 78.608696, 91.96153, 78.608696,
                ),
                VectorCommand::Line(64.935443, 78.608696),
                VectorCommand::Cubic(
                    64.421704, 78.608696, 63.956661, 78.143652, 63.956661, 77.639652,
                ),
                VectorCommand::Line(63.956661, 60.109217),
                VectorCommand::Cubic(
                    63.946922, 59.651478, 64.309704, 59.215652, 64.75527, 59.130435,
                ),
                VectorCommand::Line(64.754783, 59.130435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(33.565217, 37.217391),
                VectorCommand::Cubic(
                    32.842087, 37.29287, 32.272348, 37.940522, 32.272348, 38.663652,
                ),
                VectorCommand::Line(32.272348, 64.959304),
                VectorCommand::Cubic(
                    32.272348, 65.728696, 32.946783, 66.405565, 33.718609, 66.405565,
                ),
                VectorCommand::Line(46.890783, 66.405565),
                VectorCommand::Line(46.890783, 71.27513),
                VectorCommand::Line(37.151652, 71.27513),
                VectorCommand::Line(37.151652, 76.144696),
                VectorCommand::Line(54.19513, 76.144696),
                VectorCommand::Line(54.19513, 61.536),
                VectorCommand::Line(38.222957, 61.536),
                VectorCommand::Cubic(
                    37.709217, 61.536, 37.224696, 61.061217, 37.224696, 60.537739,
                ),
                VectorCommand::Line(37.224696, 43.056),
                VectorCommand::Cubic(
                    37.214957, 42.598261, 37.538783, 42.152696, 37.986783, 42.067478,
                ),
                VectorCommand::Cubic(
                    38.042783, 42.057739, 38.147478, 42.057739, 38.215652, 42.067478,
                ),
                VectorCommand::Line(65.241739, 42.067478),
                VectorCommand::Cubic(
                    65.745739, 42.067478, 66.230261, 42.532522, 66.230261, 43.056,
                ),
                VectorCommand::Line(66.230261, 49.362087),
                VectorCommand::Line(71.17287, 49.362087),
                VectorCommand::Line(71.17287, 38.624696),
                VectorCommand::Cubic(
                    71.17287, 37.862609, 70.488696, 37.178435, 69.726609, 37.178435,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(160, 79, 105, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(57.913043, 51.826087),
                VectorCommand::Cubic(
                    57.199652, 51.901565, 56.600696, 52.568696, 56.600696, 53.282087,
                ),
                VectorCommand::Line(56.600696, 79.577739),
                VectorCommand::Cubic(
                    56.600696, 80.339826, 57.304348, 81.033739, 58.066435, 81.033739,
                ),
                VectorCommand::Line(71.214261, 81.033739),
                VectorCommand::Line(71.214261, 88.313739),
                VectorCommand::Line(80.953391, 88.313739),
                VectorCommand::Line(80.953391, 81.033739),
                VectorCommand::Line(94.101217, 81.033739),
                VectorCommand::Cubic(
                    94.863304, 81.033739, 95.557217, 80.339826, 95.557217, 79.577739,
                ),
                VectorCommand::Line(95.557217, 53.282087),
                VectorCommand::Cubic(
                    95.557217, 52.512696, 94.853565, 51.826087, 94.101217, 51.826087,
                ),
                VectorCommand::Move(62.449043, 56.695652),
                VectorCommand::Cubic(
                    62.505043, 56.685913, 62.563478, 56.685913, 62.629217, 56.695652,
                ),
                VectorCommand::Line(89.655304, 56.695652),
                VectorCommand::Cubic(
                    90.169043, 56.695652, 90.634087, 57.160696, 90.634087, 57.674435,
                ),
                VectorCommand::Line(90.634087, 75.20487),
                VectorCommand::Cubic(
                    90.634087, 75.70887, 90.169043, 76.173913, 89.655304, 76.173913,
                ),
                VectorCommand::Line(62.629217, 76.173913),
                VectorCommand::Cubic(
                    62.115478, 76.173913, 61.650435, 75.70887, 61.650435, 75.20487,
                ),
                VectorCommand::Line(61.650435, 57.674435),
                VectorCommand::Cubic(
                    61.640696, 57.216696, 62.003478, 56.78087, 62.449043, 56.695652,
                ),
                VectorCommand::Move(61.592, 85.913043),
                VectorCommand::Line(61.592, 90.782609),
                VectorCommand::Line(90.809391, 90.782609),
                VectorCommand::Line(90.809391, 85.913043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(61, 97, 138, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_SCREEN: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0xf6dedd),
                bottom: Color::from_hex(0xedc8c7),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 29.913043),
                VectorCommand::Cubic(
                    63.732174, 29.913043, 61.565217, 32.08487, 61.565217, 34.782609,
                ),
                VectorCommand::Line(61.565217, 61.565217),
                VectorCommand::Cubic(
                    61.565217, 64.267826, 63.737043, 66.434783, 66.434783, 66.434783,
                ),
                VectorCommand::Cubic(
                    69.132522, 66.434783, 71.304348, 64.262957, 71.304348, 61.565217,
                ),
                VectorCommand::Line(71.304348, 34.782609),
                VectorCommand::Cubic(71.304348, 32.08, 69.132522, 29.913043, 66.434783, 29.913043),
                VectorCommand::Close,
                VectorCommand::Move(51.387826, 33.516522),
                VectorCommand::Cubic(
                    50.749995, 33.57248, 50.12946, 33.753676, 49.561739, 34.049739,
                ),
                VectorCommand::Cubic(
                    37.290435, 40.42887, 29.913043, 53.065391, 29.913043, 66.432348,
                ),
                VectorCommand::Line(29.903548, 66.432348),
                VectorCommand::Cubic(
                    29.894247, 86.592348, 46.265287, 102.954087, 66.425287, 102.954087,
                ),
                VectorCommand::Cubic(
                    86.585287, 102.954087, 102.947026, 86.592348, 102.947026, 66.432348,
                ),
                VectorCommand::Line(102.93753, 66.432348),
                VectorCommand::Cubic(
                    102.930713, 53.065391, 95.560139, 40.380174, 83.288835, 34.049739,
                ),
                VectorCommand::Cubic(
                    82.722604, 33.754287, 82.103783, 33.573108, 81.467617, 33.516522,
                ),
                VectorCommand::Cubic(
                    79.503529, 33.34365, 77.62903, 34.372221, 76.719791, 36.121739,
                ),
                VectorCommand::Cubic(
                    76.121801, 37.268337, 76.004442, 38.605705, 76.393592, 39.838929,
                ),
                VectorCommand::Cubic(
                    76.782743, 41.072153, 77.646441, 42.099943, 78.794226, 42.695652,
                ),
                VectorCommand::Cubic(
                    87.82727, 47.394783, 93.208139, 56.622609, 93.208139, 66.45913,
                ),
                VectorCommand::Line(93.213009, 66.45913),
                VectorCommand::Cubic(
                    93.220313, 81.262609, 81.233878, 93.241739, 66.4304, 93.241739,
                ),
                VectorCommand::Cubic(
                    51.626922, 93.241739, 39.647791, 81.262609, 39.647791, 66.45913,
                ),
                VectorCommand::Line(39.652661, 66.45913),
                VectorCommand::Cubic(
                    39.641461, 56.646957, 45.03353, 47.394783, 54.066574, 42.695652,
                ),
                VectorCommand::Cubic(
                    55.214359, 42.099943, 56.078057, 41.072153, 56.467208, 39.838929,
                ),
                VectorCommand::Cubic(
                    56.856358, 38.605705, 56.738999, 37.268337, 56.141009, 36.121739,
                ),
                VectorCommand::Cubic(
                    55.546177, 34.977403, 54.521678, 34.115774, 53.292313, 33.725913,
                ),
                VectorCommand::Cubic(
                    52.682831, 33.534633, 52.041668, 33.465139, 51.405357, 33.521391,
                ),
                VectorCommand::Line(51.387826, 33.516522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 27.478261),
                VectorCommand::Cubic(
                    61.297391, 27.478261, 59.130435, 29.650087, 59.130435, 32.347826,
                ),
                VectorCommand::Line(59.130435, 59.130435),
                VectorCommand::Cubic(59.130435, 61.833043, 61.302261, 64.0, 64.0, 64.0),
                VectorCommand::Cubic(66.697739, 64.0, 68.869565, 61.828174, 68.869565, 59.130435),
                VectorCommand::Line(68.869565, 32.347826),
                VectorCommand::Cubic(68.869565, 29.645217, 66.697739, 27.478261, 64.0, 27.478261),
                VectorCommand::Close,
                VectorCommand::Move(48.953043, 31.081739),
                VectorCommand::Cubic(
                    48.315212, 31.137697, 47.694677, 31.318893, 47.126957, 31.614957,
                ),
                VectorCommand::Cubic(
                    34.855652, 37.994087, 27.478261, 50.630609, 27.478261, 63.997565,
                ),
                VectorCommand::Line(27.468741, 63.997565),
                VectorCommand::Cubic(
                    27.45944, 84.157565, 43.83048, 100.519304, 63.99048, 100.519304,
                ),
                VectorCommand::Cubic(
                    84.15048, 100.519304, 100.512219, 84.157565, 100.512219, 63.997565,
                ),
                VectorCommand::Line(100.502699, 63.997565),
                VectorCommand::Cubic(
                    100.495979, 50.630609, 93.125308, 37.945391, 80.854003, 31.614957,
                ),
                VectorCommand::Cubic(
                    80.287773, 31.319505, 79.668952, 31.138326, 79.032786, 31.081739,
                ),
                VectorCommand::Cubic(
                    77.068698, 30.908868, 75.194199, 31.937439, 74.28496, 33.686957,
                ),
                VectorCommand::Cubic(
                    73.68697, 34.833555, 73.56961, 36.170923, 73.958761, 37.404147,
                ),
                VectorCommand::Cubic(74.347912, 38.63737, 75.21161, 39.66516, 76.359395, 40.26087),
                VectorCommand::Cubic(85.392438, 44.96, 90.773308, 54.187826, 90.773308, 64.024348),
                VectorCommand::Line(90.778056, 64.024348),
                VectorCommand::Cubic(
                    90.78536, 78.827826, 78.798925, 90.806957, 63.995447, 90.806957,
                ),
                VectorCommand::Cubic(
                    49.191969, 90.806957, 37.212838, 78.827826, 37.212838, 64.024348,
                ),
                VectorCommand::Line(37.217586, 64.024348),
                VectorCommand::Cubic(37.206435, 54.212174, 42.598456, 44.96, 51.631499, 40.26087),
                VectorCommand::Cubic(
                    52.779284, 39.66516, 53.642982, 38.63737, 54.032133, 37.404147,
                ),
                VectorCommand::Cubic(
                    54.421283, 36.170923, 54.303924, 34.833555, 53.705934, 33.686957,
                ),
                VectorCommand::Cubic(
                    53.111102, 32.542621, 52.086603, 31.680991, 50.857238, 31.29113,
                ),
                VectorCommand::Cubic(
                    50.247756, 31.09985, 49.606594, 31.030356, 48.970282, 31.086609,
                ),
                VectorCommand::Line(48.953043, 31.081739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(163, 77, 81, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_FILE_MANAGER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xdad2e9),
                bottom: Color::from_hex(0xece5f4),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 84.695652),
                VectorCommand::Cubic(
                    37.217391, 88.055652, 39.944348, 90.782609, 43.304348, 90.782609,
                ),
                VectorCommand::Line(89.565217, 90.782609),
                VectorCommand::Cubic(
                    92.925217, 90.782609, 95.652174, 88.055652, 95.652174, 84.695652,
                ),
                VectorCommand::Line(95.652174, 48.173913),
                VectorCommand::Cubic(
                    95.652174, 44.813913, 92.925217, 42.086957, 89.565217, 42.086957,
                ),
                VectorCommand::Line(37.217391, 84.695652),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(40.869565, 44.521739),
                VectorCommand::Cubic(
                    37.509565, 44.521739, 34.782609, 47.248696, 34.782609, 50.608696,
                ),
                VectorCommand::Line(34.782609, 73.73913),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Line(83.478261, 44.521739),
                VectorCommand::Line(40.869565, 44.521739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(200, 138, 61, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(40.869565, 44.521739),
                VectorCommand::Cubic(
                    37.509565, 44.521739, 34.782609, 47.248696, 34.782609, 50.608696,
                ),
                VectorCommand::Line(34.782609, 53.043478),
                VectorCommand::Cubic(
                    34.782609, 49.683478, 37.509565, 46.956522, 40.869565, 46.956522,
                ),
                VectorCommand::Line(83.478261, 46.956522),
                VectorCommand::Line(83.478261, 44.521739),
                VectorCommand::Line(40.869565, 44.521739),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 250, 241, 51))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(71.304348, 39.652174),
                VectorCommand::Cubic(
                    61.565217, 39.652174, 66.434783, 54.26087, 51.826087, 54.26087,
                ),
                VectorCommand::Line(40.869565, 54.26087),
                VectorCommand::Cubic(
                    37.509565, 54.26087, 34.782609, 56.987826, 34.782609, 60.347826,
                ),
                VectorCommand::Line(34.782609, 82.26087),
                VectorCommand::Cubic(
                    34.782609, 85.62087, 37.509565, 88.347826, 40.869565, 88.347826,
                ),
                VectorCommand::Line(87.130435, 88.347826),
                VectorCommand::Cubic(
                    90.490435, 88.347826, 93.217391, 85.62087, 93.217391, 82.26087,
                ),
                VectorCommand::Line(93.217391, 45.73913),
                VectorCommand::Cubic(
                    93.217391, 42.37913, 90.490435, 39.652174, 87.130435, 39.652174,
                ),
                VectorCommand::Line(71.304348, 39.652174),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(229, 182, 95, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 79.826087),
                VectorCommand::Line(34.782609, 82.26087),
                VectorCommand::Cubic(
                    34.782609, 85.62087, 37.509565, 88.347826, 40.869565, 88.347826,
                ),
                VectorCommand::Line(87.130435, 88.347826),
                VectorCommand::Cubic(
                    90.490435, 88.347826, 93.217391, 85.62087, 93.217391, 82.26087,
                ),
                VectorCommand::Line(93.217391, 79.826087),
                VectorCommand::Cubic(
                    93.217391, 83.186087, 90.490435, 85.913043, 87.130435, 85.913043,
                ),
                VectorCommand::Line(40.869565, 85.913043),
                VectorCommand::Cubic(
                    37.509565, 85.913043, 34.782609, 83.186087, 34.782609, 79.826087,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 17))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_OFFICE_VIEWER: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0xf5dfe8),
                bottom: Color::from_hex(0xebd0dc),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Rect {
                x: 42.086957,
                y: 34.782609,
                width: 48.695652,
                height: 63.304348,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 32.347826),
                VectorCommand::Line(42.086957, 64.0),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Cubic(
                    86.18087, 95.652174, 88.347826, 93.480348, 88.347826, 90.782609,
                ),
                VectorCommand::Line(88.347826, 37.217391),
                VectorCommand::Cubic(
                    88.347826, 34.514783, 86.176, 32.347826, 83.478261, 32.347826,
                ),
                VectorCommand::Line(81.043478, 32.347826),
                VectorCommand::Line(76.173913, 34.782609),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Line(46.956522, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(246, 248, 250, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.521739, 32.347826),
                VectorCommand::Cubic(
                    41.81913, 32.347826, 39.652174, 34.519652, 39.652174, 37.217391,
                ),
                VectorCommand::Line(39.652174, 90.782609),
                VectorCommand::Cubic(
                    39.652174, 93.485217, 41.824, 95.652174, 44.521739, 95.652174,
                ),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(46.956522, 32.347826),
                VectorCommand::Line(44.521739, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(120, 147, 162, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(73.73913, 32.347826),
                VectorCommand::Line(83.478261, 32.347826),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Line(78.608696, 56.695652),
                VectorCommand::Line(73.73913, 61.565217),
                VectorCommand::Line(73.73913, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(71.304348, 32.347826),
                VectorCommand::Line(81.043478, 32.347826),
                VectorCommand::Line(81.043478, 59.130435),
                VectorCommand::Line(76.173913, 54.26087),
                VectorCommand::Line(71.304348, 59.130435),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(195, 92, 127, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(55.478261, 73.73913),
                VectorCommand::Cubic(
                    54.803826, 73.73913, 54.26087, 74.282087, 54.26087, 74.956522,
                ),
                VectorCommand::Cubic(
                    54.26087, 75.630957, 54.803826, 76.173913, 55.478261, 76.173913,
                ),
                VectorCommand::Line(79.826087, 76.173913),
                VectorCommand::Cubic(
                    80.500522, 76.173913, 81.043478, 75.630957, 81.043478, 74.956522,
                ),
                VectorCommand::Cubic(
                    81.043478, 74.282087, 80.500522, 73.73913, 79.826087, 73.73913,
                ),
                VectorCommand::Line(55.478261, 73.73913),
                VectorCommand::Close,
                VectorCommand::Move(55.478261, 78.608696),
                VectorCommand::Cubic(
                    54.803826, 78.608696, 54.26087, 79.151652, 54.26087, 79.826087,
                ),
                VectorCommand::Cubic(
                    54.26087, 80.500522, 54.803826, 81.043478, 55.478261, 81.043478,
                ),
                VectorCommand::Line(79.826087, 81.043478),
                VectorCommand::Cubic(
                    80.500522, 81.043478, 81.043478, 80.500522, 81.043478, 79.826087,
                ),
                VectorCommand::Cubic(
                    81.043478, 79.151652, 80.500522, 78.608696, 79.826087, 78.608696,
                ),
                VectorCommand::Line(55.478261, 78.608696),
                VectorCommand::Close,
                VectorCommand::Move(55.478261, 83.478261),
                VectorCommand::Cubic(
                    54.803826, 83.478261, 54.26087, 84.021217, 54.26087, 84.695652,
                ),
                VectorCommand::Cubic(
                    54.26087, 85.370087, 54.803826, 85.913043, 55.478261, 85.913043,
                ),
                VectorCommand::Line(79.826087, 85.913043),
                VectorCommand::Cubic(
                    80.500522, 85.913043, 81.043478, 85.370087, 81.043478, 84.695652,
                ),
                VectorCommand::Cubic(
                    81.043478, 84.021217, 80.500522, 83.478261, 79.826087, 83.478261,
                ),
                VectorCommand::Line(55.478261, 83.478261),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(73, 101, 121, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_SUB2API_MONITOR: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0xd9efe7),
                bottom: Color::from_hex(0xc4e3d8),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 34.782609),
                VectorCommand::Line(44.521739, 64.0),
                VectorCommand::Line(29.913043, 64.0),
                VectorCommand::Line(29.913043, 68.869565),
                VectorCommand::Line(48.173913, 68.869565),
                VectorCommand::Line(56.695652, 48.173913),
                VectorCommand::Line(76.173913, 98.086957),
                VectorCommand::Line(88.347826, 68.869565),
                VectorCommand::Line(102.956522, 68.869565),
                VectorCommand::Line(102.956522, 64.0),
                VectorCommand::Line(84.695652, 64.0),
                VectorCommand::Line(76.173913, 84.695652),
                VectorCommand::Line(56.695652, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(54.26087, 32.347826),
                VectorCommand::Line(42.086957, 61.565217),
                VectorCommand::Line(27.478261, 61.565217),
                VectorCommand::Line(27.478261, 66.434783),
                VectorCommand::Line(45.73913, 66.434783),
                VectorCommand::Line(54.26087, 45.73913),
                VectorCommand::Line(73.73913, 95.652174),
                VectorCommand::Line(85.913043, 66.434783),
                VectorCommand::Line(100.521739, 66.434783),
                VectorCommand::Line(100.521739, 61.565217),
                VectorCommand::Line(82.26087, 61.565217),
                VectorCommand::Line(73.73913, 82.26087),
                VectorCommand::Line(54.26087, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(54, 126, 105, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_WIFI: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xd2e4cb),
                bottom: Color::from_hex(0xe4efdf),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(88.808, 34.782609),
                VectorCommand::Line(82.944557, 40.522365),
                VectorCommand::Cubic(
                    92.0896, 49.463374, 92.164591, 63.91527, 82.996865, 72.873322,
                ),
                VectorCommand::Line(88.865178, 78.608452),
                VectorCommand::Cubic(
                    101.199787, 66.548974, 101.135509, 46.834539, 88.808112, 34.782365,
                ),
                VectorCommand::Line(88.808, 34.782609),
                VectorCommand::Close,
                VectorCommand::Move(82.944557, 40.522365),
                VectorCommand::Line(82.939801, 40.51761),
                VectorCommand::Line(82.939801, 40.52712),
                VectorCommand::Line(82.944557, 40.522365),
                VectorCommand::Close,
                VectorCommand::Move(44.063513, 34.830087),
                VectorCommand::Cubic(
                    31.719165, 46.889565, 31.679235, 66.506609, 44.006447, 78.551478,
                ),
                VectorCommand::Line(49.87476, 72.816348),
                VectorCommand::Cubic(
                    40.724847, 63.866087, 40.767455, 49.529843, 49.927069, 40.570087,
                ),
                VectorCommand::Line(44.063625, 34.83033),
                VectorCommand::Line(44.063513, 34.830087),
                VectorCommand::Close,
                VectorCommand::Move(78.703165, 44.7024),
                VectorCommand::Line(75.479026, 47.874191),
                VectorCommand::Cubic(
                    80.501983, 52.809496, 80.537774, 60.776348, 75.507557, 65.721391,
                ),
                VectorCommand::Line(78.7222, 68.893183),
                VectorCommand::Cubic(
                    85.483592, 62.246226, 85.457296, 51.36153, 78.703177, 44.7024,
                ),
                VectorCommand::Line(78.703165, 44.7024),
                VectorCommand::Close,
                VectorCommand::Move(54.165426, 44.764219),
                VectorCommand::Cubic(
                    47.404035, 51.403871, 47.375548, 62.246445, 54.136893, 68.883663,
                ),
                VectorCommand::Line(57.351536, 65.697506),
                VectorCommand::Cubic(
                    52.338319, 60.779245, 52.357067, 52.866932, 57.38007, 47.91215,
                ),
                VectorCommand::Line(54.165426, 44.763976),
                VectorCommand::Line(54.165426, 44.764219),
                VectorCommand::Close,
                VectorCommand::Move(66.434539, 51.835558),
                VectorCommand::Cubic(
                    63.744104, 51.835558, 61.564974, 54.012473, 61.564974, 56.695628,
                ),
                VectorCommand::Cubic(
                    61.564974, 58.498341, 62.544803, 60.065123, 63.999757, 60.90415,
                ),
                VectorCommand::Line(64.0378, 95.650932),
                VectorCommand::Line(46.956339, 95.650932),
                VectorCommand::Line(46.956339, 100.520497),
                VectorCommand::Line(85.912861, 100.520497),
                VectorCommand::Line(85.912861, 95.650932),
                VectorCommand::Line(68.859887, 95.650932),
                VectorCommand::Line(68.821843, 60.930932),
                VectorCommand::Cubic(
                    70.30319, 60.098967, 71.304104, 58.516261, 71.304104, 56.693923,
                ),
                VectorCommand::Cubic(
                    71.304104, 54.013228, 69.124974, 51.833854, 66.434539, 51.833854,
                ),
                VectorCommand::Line(66.434539, 51.835558),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.73113, 42.330435),
                VectorCommand::Cubic(
                    44.969739, 48.970087, 44.940522, 59.812174, 51.701913, 66.449391,
                ),
                VectorCommand::Line(54.915826, 63.262261),
                VectorCommand::Cubic(
                    49.902609, 58.344, 49.922087, 50.430957, 54.945043, 45.476174,
                ),
                VectorCommand::Move(73.045217, 45.437217),
                VectorCommand::Cubic(
                    78.068174, 50.372522, 78.104696, 58.344, 73.074435, 63.289043,
                ),
                VectorCommand::Line(76.288348, 66.456696),
                VectorCommand::Cubic(
                    83.049739, 59.809739, 83.022957, 48.928696, 76.26887, 42.269565,
                ),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(72, 124, 67, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(41.631652, 32.396522),
                VectorCommand::Cubic(
                    29.287304, 44.456, 29.248348, 64.073043, 41.575652, 76.117913,
                ),
                VectorCommand::Line(47.443478, 70.384),
                VectorCommand::Cubic(
                    38.293565, 61.433739, 38.332522, 47.09287, 47.492174, 38.13287,
                ),
                VectorCommand::Move(80.505391, 38.084174),
                VectorCommand::Cubic(89.655304, 47.024696, 89.730783, 61.48, 80.561391, 70.44),
                VectorCommand::Line(86.429217, 76.173913),
                VectorCommand::Cubic(98.763826, 64.114435, 98.700522, 44.4, 86.373217, 32.347826),
                VectorCommand::Line(80.505391, 38.091478),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(72, 124, 67, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 49.401043),
                VectorCommand::Cubic(
                    61.309565, 49.401043, 59.130435, 51.577958, 59.130435, 54.261113,
                ),
                VectorCommand::Cubic(
                    59.130435, 56.063826, 60.110264, 57.630609, 61.565217, 58.469635,
                ),
                VectorCommand::Line(61.603261, 93.216417),
                VectorCommand::Line(44.5218, 93.216417),
                VectorCommand::Line(44.5218, 98.085983),
                VectorCommand::Line(83.478322, 98.085983),
                VectorCommand::Line(83.478322, 93.216417),
                VectorCommand::Line(66.425348, 93.216417),
                VectorCommand::Line(66.387304, 58.496417),
                VectorCommand::Cubic(
                    67.86865, 57.664452, 68.869565, 56.081746, 68.869565, 54.259409,
                ),
                VectorCommand::Cubic(68.869565, 51.578713, 66.690435, 49.399339, 64.0, 49.399339),
                VectorCommand::Line(64.0, 49.401043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(52, 94, 57, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_BLUETOOTH: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xcbdfea),
                bottom: Color::from_hex(0xdfedf4),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.565217, 29.913043),
                VectorCommand::Line(60.347826, 66.434783),
                VectorCommand::Line(42.086957, 83.478261),
                VectorCommand::Line(46.956522, 88.347826),
                VectorCommand::Line(61.565217, 73.73913),
                VectorCommand::Line(61.565217, 102.956522),
                VectorCommand::Line(90.782609, 81.043478),
                VectorCommand::Line(72.05913, 66.434783),
                VectorCommand::Line(90.782609, 51.826087),
                VectorCommand::Move(68.869565, 44.521739),
                VectorCommand::Line(78.608696, 51.826087),
                VectorCommand::Line(68.869565, 59.130435),
                VectorCommand::Move(68.869565, 73.73913),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Line(68.869565, 88.347826),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.130435, 27.478261),
                VectorCommand::Line(59.130435, 56.695652),
                VectorCommand::Line(44.521739, 42.086957),
                VectorCommand::Line(39.652174, 46.956522),
                VectorCommand::Line(57.913043, 64.0),
                VectorCommand::Line(39.652174, 81.043478),
                VectorCommand::Line(44.521739, 85.913043),
                VectorCommand::Line(59.130435, 71.304348),
                VectorCommand::Line(59.130435, 100.521739),
                VectorCommand::Line(88.347826, 78.608696),
                VectorCommand::Line(69.624348, 64.0),
                VectorCommand::Line(88.347826, 49.391304),
                VectorCommand::Move(66.434783, 42.086957),
                VectorCommand::Line(76.173913, 49.391304),
                VectorCommand::Line(66.434783, 56.695652),
                VectorCommand::Move(66.434783, 71.304348),
                VectorCommand::Line(76.173913, 78.608696),
                VectorCommand::Line(66.434783, 85.913043),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(57, 109, 139, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_DISPLAY_SETTINGS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xc8e5dc),
                bottom: Color::from_hex(0xdef0e8),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.094609, 34.782609),
                VectorCommand::Cubic(
                    36.698783, 34.782609, 34.777739, 36.703652, 34.777739, 39.099478,
                ),
                VectorCommand::Line(34.782609, 83.478261),
                VectorCommand::Line(59.130435, 83.478261),
                VectorCommand::Cubic(
                    59.130435, 90.782609, 59.130435, 90.782609, 54.26087, 95.652174,
                ),
                VectorCommand::Line(49.391304, 95.652174),
                VectorCommand::Line(49.391304, 98.086957),
                VectorCommand::Line(83.478261, 98.086957),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(78.608696, 95.652174),
                VectorCommand::Cubic(
                    73.73913, 90.782609, 73.73913, 90.782609, 73.73913, 83.478261,
                ),
                VectorCommand::Line(98.086957, 83.478261),
                VectorCommand::Line(98.082087, 39.099478),
                VectorCommand::Cubic(
                    98.082087, 36.703652, 96.165913, 34.782609, 93.770087, 34.782609,
                ),
                VectorCommand::Line(39.101913, 34.782609),
                VectorCommand::Line(39.094609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.659826, 32.347826),
                VectorCommand::Cubic(34.264, 32.347826, 32.341983, 34.269941, 32.341983, 36.66567),
                VectorCommand::Line(32.341983, 73.740104),
                VectorCommand::Line(63.994157, 78.60967),
                VectorCommand::Line(95.64633, 73.740104),
                VectorCommand::Line(95.64633, 36.66567),
                VectorCommand::Cubic(
                    95.64633, 34.269843, 93.728988, 32.347826, 91.333113, 32.347826,
                ),
                VectorCommand::Line(36.662504, 32.347826),
                VectorCommand::Line(36.659826, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(77, 112, 122, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 81.043478),
                VectorCommand::Cubic(
                    56.695652, 88.347826, 56.695652, 88.347826, 51.826087, 93.217391,
                ),
                VectorCommand::Line(64.0, 95.652174),
                VectorCommand::Line(76.173913, 93.217391),
                VectorCommand::Cubic(
                    71.304348, 88.347826, 71.304348, 88.347826, 71.304348, 81.043478,
                ),
                VectorCommand::Line(64.0, 73.73913),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(163, 186, 192, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.956522, 93.217391),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Line(81.043478, 95.652174),
                VectorCommand::Line(81.043478, 93.217391),
                VectorCommand::Line(46.956522, 93.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(99, 129, 139, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 73.73913),
                VectorCommand::Line(32.347826, 81.043478),
                VectorCommand::Line(95.652174, 81.043478),
                VectorCommand::Line(95.652174, 73.73913),
                VectorCommand::Line(32.347826, 73.73913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(99, 129, 139, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.217391, 37.217391),
                VectorCommand::Line(90.782609, 37.217391),
                VectorCommand::Line(90.782609, 68.869565),
                VectorCommand::Line(37.217391, 68.869565),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(243, 248, 247, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(36.684174, 32.347826),
                VectorCommand::Cubic(
                    34.298087, 32.347826, 32.347339, 34.298696, 32.347339, 36.684661,
                ),
                VectorCommand::Line(32.347339, 73.739617),
                VectorCommand::Line(78.608209, 73.739617),
                VectorCommand::Line(49.390817, 32.348313),
                VectorCommand::Line(36.684174, 32.348313),
                VectorCommand::Line(36.684174, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(56.695652, 76.173913),
                VectorCommand::Line(71.304348, 76.173913),
                VectorCommand::Line(71.304348, 78.608696),
                VectorCommand::Line(56.695652, 78.608696),
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(77, 112, 122, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_DATE_TIME: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.682157, 46.956522),
                VectorCommand::Cubic(8.913092, 52.464974, 8.008278, 58.21447, 8.0, 64.0),
                VectorCommand::Cubic(8.0, 94.929043, 33.070957, 120.0, 64.0, 120.0),
                VectorCommand::Cubic(94.929043, 120.0, 120.0, 94.929043, 120.0, 64.0),
                VectorCommand::Cubic(
                    119.977934, 58.212278, 119.058737, 52.462783, 117.275235, 46.956522,
                ),
                VectorCommand::Cubic(
                    112.695652, 42.086957, 15.304104, 42.086957, 10.682887, 46.956522,
                ),
                VectorCommand::Line(10.682157, 46.956522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0xf0f3f3),
                bottom: Color::from_hex(0xdfe5e7),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.682157, 46.956522),
                VectorCommand::Cubic(
                    10.438783, 47.762922, 10.213631, 48.574727, 10.006894, 49.391304,
                ),
                VectorCommand::Line(118.045503, 49.391304),
                VectorCommand::Cubic(
                    117.807249, 48.573948, 117.55039, 47.762143, 117.275113, 46.956522,
                ),
                VectorCommand::Cubic(
                    112.69553, 42.086957, 15.303983, 42.086957, 10.682765, 46.956522,
                ),
                VectorCommand::Line(10.682157, 46.956522),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(
                    39.647842, 8.011544, 18.094655, 23.759603, 10.68313, 46.956522,
                ),
                VectorCommand::Line(117.275478, 46.956522),
                VectorCommand::Cubic(109.868612, 23.774129, 88.336889, 8.02953, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(184, 98, 91, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 29.913043),
                VectorCommand::Cubic(
                    36.962787, 29.913043, 34.782609, 32.093222, 34.782609, 34.782609,
                ),
                VectorCommand::Cubic(
                    34.782609, 37.471995, 36.962787, 39.652174, 39.652174, 39.652174,
                ),
                VectorCommand::Cubic(
                    42.341561, 39.652174, 44.521739, 37.471995, 44.521739, 34.782609,
                ),
                VectorCommand::Cubic(
                    44.521739, 32.093222, 42.341561, 29.913043, 39.652174, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(51.826087, 29.913043),
                VectorCommand::Cubic(
                    49.1367, 29.913043, 46.956522, 32.093222, 46.956522, 34.782609,
                ),
                VectorCommand::Cubic(
                    46.956522, 37.471995, 49.1367, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Cubic(
                    54.515474, 39.652174, 56.695652, 37.471995, 56.695652, 34.782609,
                ),
                VectorCommand::Cubic(
                    56.695652, 32.093222, 54.515474, 29.913043, 51.826087, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 29.913043),
                VectorCommand::Cubic(
                    73.484526, 29.913043, 71.304348, 32.093222, 71.304348, 34.782609,
                ),
                VectorCommand::Cubic(
                    71.304348, 37.471995, 73.484526, 39.652174, 76.173913, 39.652174,
                ),
                VectorCommand::Cubic(
                    78.8633, 39.652174, 81.043478, 37.471995, 81.043478, 34.782609,
                ),
                VectorCommand::Cubic(
                    81.043478, 32.093222, 78.8633, 29.913043, 76.173913, 29.913043,
                ),
                VectorCommand::Close,
                VectorCommand::Move(88.347826, 29.913043),
                VectorCommand::Cubic(
                    85.658439, 29.913043, 83.478261, 32.093222, 83.478261, 34.782609,
                ),
                VectorCommand::Cubic(
                    83.478261, 37.471995, 85.658439, 39.652174, 88.347826, 39.652174,
                ),
                VectorCommand::Cubic(
                    91.037213, 39.652174, 93.217391, 37.471995, 93.217391, 34.782609,
                ),
                VectorCommand::Cubic(
                    93.217391, 32.093222, 91.037213, 29.913043, 88.347826, 29.913043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Cubic(
                    38.303304, 32.347826, 37.217391, 33.433739, 37.217391, 34.782609,
                ),
                VectorCommand::Cubic(
                    37.217391, 36.131478, 38.303304, 37.217391, 39.652174, 37.217391,
                ),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Cubic(
                    53.174957, 37.217391, 54.26087, 36.131478, 54.26087, 34.782609,
                ),
                VectorCommand::Cubic(
                    54.26087, 33.433739, 53.174957, 32.347826, 51.826087, 32.347826,
                ),
                VectorCommand::Line(39.652174, 32.347826),
                VectorCommand::Close,
                VectorCommand::Move(76.173913, 32.347826),
                VectorCommand::Cubic(
                    74.825043, 32.347826, 73.73913, 33.433739, 73.73913, 34.782609,
                ),
                VectorCommand::Cubic(
                    73.73913, 36.131478, 74.825043, 37.217391, 76.173913, 37.217391,
                ),
                VectorCommand::Line(88.347826, 37.217391),
                VectorCommand::Cubic(
                    89.696696, 37.217391, 90.782609, 36.131478, 90.782609, 34.782609,
                ),
                VectorCommand::Cubic(
                    90.782609, 33.433739, 89.696696, 32.347826, 88.347826, 32.347826,
                ),
                VectorCommand::Line(76.173913, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(238, 222, 218, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.616068, 72.230978),
                VectorCommand::Quad(61.616068, 73.887739, 61.077058, 75.247194),
                VectorCommand::Quad(60.538048, 76.609047, 59.556457, 77.60646),
                VectorCommand::Quad(58.597119, 78.603873, 57.242177, 79.263221),
                VectorCommand::Quad(55.884762, 79.898592, 54.2257, 80.169524),
                VectorCommand::Line(54.2257, 80.306189),
                VectorCommand::Quad(58.364702, 80.804896, 60.515796, 82.869253),
                VectorCommand::Quad(62.66689, 84.909634, 62.66689, 88.19918),
                VectorCommand::Quad(62.66689, 90.376226, 61.895463, 92.19123),
                VectorCommand::Quad(61.146289, 94.006234, 59.603435, 95.320134),
                VectorCommand::Quad(58.060582, 96.636432, 55.696851, 97.360516),
                VectorCommand::Quad(53.335593, 98.086997, 50.106479, 98.086997),
                VectorCommand::Quad(47.557309, 98.086997, 45.287534, 97.700979),
                VectorCommand::Quad(43.042485, 97.314961, 41.054577, 96.295969),
                VectorCommand::Line(41.054577, 92.440583),
                VectorCommand::Quad(43.089462, 93.483552, 45.450721, 94.051789),
                VectorCommand::Quad(47.836704, 94.617629, 50.012523, 94.617629),
                VectorCommand::Quad(52.163617, 94.617629, 53.731196, 94.164478),
                VectorCommand::Quad(55.298775, 93.711326, 56.302619, 92.872156),
                VectorCommand::Quad(57.331188, 92.032987, 57.798494, 90.807799),
                VectorCommand::Quad(58.290526, 89.582611, 58.290526, 88.062515),
                VectorCommand::Quad(58.290526, 86.52084, 57.660033, 85.432317),
                VectorCommand::Quad(57.051793, 84.322216, 55.907015, 83.617313),
                VectorCommand::Quad(54.762238, 82.890832, 53.100703, 82.550369),
                VectorCommand::Quad(51.463894, 82.209905, 49.38203, 82.209905),
                VectorCommand::Line(46.271598, 82.209905),
                VectorCommand::Line(46.271598, 78.786093),
                VectorCommand::Line(49.38203, 78.786093),
                VectorCommand::Quad(51.275982, 78.786093, 52.749605, 78.332941),
                VectorCommand::Quad(54.223228, 77.87979, 55.204819, 77.04062),
                VectorCommand::Quad(56.211135, 76.20145, 56.72542, 75.045794),
                VectorCommand::Quad(57.239704, 73.890137, 57.239704, 72.506706),
                VectorCommand::Quad(57.239704, 71.327073, 56.819376, 70.396793),
                VectorCommand::Quad(56.399047, 69.466514, 55.62762, 68.831142),
                VectorCommand::Quad(54.856193, 68.174192, 53.780646, 67.833729),
                VectorCommand::Quad(52.705099, 67.493266, 51.394663, 67.493266),
                VectorCommand::Quad(48.892471, 67.493266, 46.973794, 68.265302),
                VectorCommand::Quad(45.079842, 69.013362, 43.349077, 70.23855),
                VectorCommand::Line(41.197983, 67.404554),
                VectorCommand::Quad(42.085619, 66.702049, 43.163638, 66.088256),
                VectorCommand::Quad(44.26391, 65.476861, 45.549622, 65.021312),
                VectorCommand::Quad(46.835333, 64.544184, 48.286703, 64.273252),
                VectorCommand::Quad(49.760326, 63.999922, 51.397135, 63.999922),
                VectorCommand::Quad(53.924053, 63.999922, 55.818004, 64.611317),
                VectorCommand::Quad(57.736681, 65.222712, 59.022393, 66.335212),
                VectorCommand::Quad(60.308104, 67.423735, 60.963322, 68.943831),
                VectorCommand::Quad(61.61854, 70.43995, 61.61854, 72.233376),
                VectorCommand::Line(61.616068, 72.230978),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(82.150361, 97.631448),
                VectorCommand::Line(78.033612, 97.631448),
                VectorCommand::Line(78.033612, 76.925534),
                VectorCommand::Quad(78.033612, 75.949699, 78.033612, 74.839598),
                VectorCommand::Quad(78.055865, 73.729496, 78.08059, 72.638575),
                VectorCommand::Quad(78.127568, 71.528474, 78.149821, 70.528663),
                VectorCommand::Quad(78.196799, 69.507273, 78.219051, 68.737635),
                VectorCommand::Quad(77.820975, 69.145231, 77.516855, 69.44014),
                VectorCommand::Quad(77.212735, 69.735048, 76.886362, 70.00598),
                VectorCommand::Quad(76.582242, 70.279309, 76.231144, 70.595796),
                VectorCommand::Quad(75.880046, 70.890705, 75.388014, 71.298301),
                VectorCommand::Line(71.926483, 74.043585),
                VectorCommand::Line(69.681434, 71.255144),
                VectorCommand::Line(78.63938, 64.474652),
                VectorCommand::Line(82.147888, 64.474652),
                VectorCommand::Line(82.147888, 97.631448),
                VectorCommand::Line(82.150361, 97.631448),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.181379, 69.794989),
                VectorCommand::Quad(59.181379, 71.451749, 58.642369, 72.811204),
                VectorCommand::Quad(58.103359, 74.173057, 57.121768, 75.17047),
                VectorCommand::Quad(56.16243, 76.167883, 54.807488, 76.827231),
                VectorCommand::Quad(53.450073, 77.462602, 51.791011, 77.733534),
                VectorCommand::Line(51.791011, 77.870199),
                VectorCommand::Quad(55.930013, 78.368906, 58.081107, 80.433263),
                VectorCommand::Quad(60.2322, 82.473645, 60.2322, 85.76319),
                VectorCommand::Quad(60.2322, 87.940236, 59.460774, 89.75524),
                VectorCommand::Quad(58.7116, 91.570245, 57.168746, 92.884145),
                VectorCommand::Quad(55.625892, 94.200442, 53.262162, 94.924526),
                VectorCommand::Quad(50.900903, 95.651007, 47.67179, 95.651007),
                VectorCommand::Quad(45.12262, 95.651007, 42.852845, 95.264989),
                VectorCommand::Quad(40.607795, 94.878971, 38.619888, 93.859979),
                VectorCommand::Line(38.619888, 90.004594),
                VectorCommand::Quad(40.654773, 91.047562, 43.016032, 91.615799),
                VectorCommand::Quad(45.402015, 92.18164, 47.577834, 92.18164),
                VectorCommand::Quad(49.728928, 92.18164, 51.296507, 91.728488),
                VectorCommand::Quad(52.864086, 91.275336, 53.867929, 90.436167),
                VectorCommand::Quad(54.896499, 89.596997, 55.363805, 88.371809),
                VectorCommand::Quad(55.855837, 87.146621, 55.855837, 85.626525),
                VectorCommand::Quad(55.855837, 84.08485, 55.225344, 82.996327),
                VectorCommand::Quad(54.617104, 81.886226, 53.472326, 81.181323),
                VectorCommand::Quad(52.327548, 80.454842, 50.666014, 80.114379),
                VectorCommand::Quad(49.029204, 79.773916, 46.947341, 79.773916),
                VectorCommand::Line(43.836909, 79.773916),
                VectorCommand::Line(43.836909, 76.350103),
                VectorCommand::Line(46.947341, 76.350103),
                VectorCommand::Quad(48.841293, 76.350103, 50.314916, 75.896951),
                VectorCommand::Quad(51.788539, 75.4438, 52.77013, 74.60463),
                VectorCommand::Quad(53.776446, 73.76546, 54.290731, 72.609804),
                VectorCommand::Quad(54.805015, 71.454147, 54.805015, 70.070716),
                VectorCommand::Quad(54.805015, 68.891083, 54.384687, 67.960803),
                VectorCommand::Quad(53.964358, 67.030524, 53.192931, 66.395152),
                VectorCommand::Quad(52.421504, 65.738202, 51.345957, 65.397739),
                VectorCommand::Quad(50.27041, 65.057276, 48.959974, 65.057276),
                VectorCommand::Quad(46.457782, 65.057276, 44.539105, 65.829312),
                VectorCommand::Quad(42.645153, 66.577372, 40.914388, 67.80256),
                VectorCommand::Line(38.763294, 64.968564),
                VectorCommand::Quad(39.650929, 64.266059, 40.728949, 63.652266),
                VectorCommand::Quad(41.829221, 63.040871, 43.114932, 62.585322),
                VectorCommand::Quad(44.400644, 62.108194, 45.852014, 61.837262),
                VectorCommand::Quad(47.325637, 61.563932, 48.962446, 61.563932),
                VectorCommand::Quad(51.489364, 61.563932, 53.383315, 62.175327),
                VectorCommand::Quad(55.301992, 62.786722, 56.587703, 63.899222),
                VectorCommand::Quad(57.873415, 64.987745, 58.528633, 66.507841),
                VectorCommand::Quad(59.183851, 68.003961, 59.183851, 69.797386),
                VectorCommand::Line(59.181379, 69.794989),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(80, 98, 116, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(79.715672, 95.185867),
                VectorCommand::Line(75.598923, 95.185867),
                VectorCommand::Line(75.598923, 74.479953),
                VectorCommand::Quad(75.598923, 73.504119, 75.598923, 72.394017),
                VectorCommand::Quad(75.621176, 71.283915, 75.645901, 70.192995),
                VectorCommand::Quad(75.692879, 69.082893, 75.715132, 68.083082),
                VectorCommand::Quad(75.762109, 67.061693, 75.784362, 66.292054),
                VectorCommand::Quad(75.386286, 66.699651, 75.082166, 66.994559),
                VectorCommand::Quad(74.778046, 67.289468, 74.451673, 67.560399),
                VectorCommand::Quad(74.147553, 67.833729, 73.796455, 68.150216),
                VectorCommand::Quad(73.445357, 68.445124, 72.953325, 68.852721),
                VectorCommand::Line(69.491794, 71.598005),
                VectorCommand::Line(67.246745, 68.809564),
                VectorCommand::Line(76.204691, 62.029072),
                VectorCommand::Line(79.713199, 62.029072),
                VectorCommand::Line(79.713199, 95.185867),
                VectorCommand::Line(79.715672, 95.185867),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(80, 98, 116, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_STORAGE: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.947826, 17.73913),
                VectorCommand::Cubic(
                    108.215652, 27.624348, 117.54087, 44.034783, 117.54087, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.54087, 92.973913, 92.949565, 117.565217, 62.758261, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.083478, 117.565217, 27.697391, 108.24, 17.714783, 93.972174,
                ),
                VectorCommand::Cubic(
                    20.246957, 98.403478, 23.33913, 102.493913, 26.918261, 106.073043,
                ),
                VectorCommand::Cubic(
                    36.852174, 115.106087, 50.073043, 120.608696, 64.657391, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.57913, 120.608696, 120.657391, 95.530435, 120.657391, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.657391, 50.121739, 115.154783, 36.852174, 106.121739, 26.869565,
                ),
                VectorCommand::Cubic(
                    102.542609, 23.266087, 98.452174, 20.198261, 94.02087, 17.666087,
                ),
                VectorCommand::Line(93.947826, 17.73913),
                VectorCommand::Close,
                VectorCommand::Move(106.048696, 26.942609),
                VectorCommand::Cubic(
                    115.422609, 36.949565, 121.193043, 50.413913, 121.193043, 65.168696,
                ),
                VectorCommand::Cubic(
                    121.193043, 96.090435, 96.114783, 121.168696, 65.193043, 121.168696,
                ),
                VectorCommand::Cubic(
                    50.389565, 121.168696, 36.949565, 115.422609, 26.966957, 106.024348,
                ),
                VectorCommand::Cubic(
                    37.095652, 116.104348, 51.022609, 122.386087, 66.410435, 122.386087,
                ),
                VectorCommand::Cubic(
                    97.332174, 122.386087, 122.410435, 97.307826, 122.410435, 66.386087,
                ),
                VectorCommand::Cubic(
                    122.410435, 50.949565, 116.128696, 36.925217, 106.048696, 26.942609,
                ),
                VectorCommand::Line(106.048696, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.121739, 26.942609),
                VectorCommand::Cubic(
                    115.154783, 36.876522, 120.657391, 50.097391, 120.657391, 64.681739,
                ),
                VectorCommand::Cubic(
                    120.657391, 95.603478, 95.57913, 120.681739, 64.657391, 120.681739,
                ),
                VectorCommand::Cubic(
                    50.170435, 120.681739, 36.90087, 115.17913, 26.918261, 106.146087,
                ),
                VectorCommand::Cubic(
                    36.925217, 115.52, 50.389565, 121.290435, 65.144348, 121.290435,
                ),
                VectorCommand::Cubic(
                    96.066087, 121.290435, 121.144348, 96.212174, 121.144348, 65.290435,
                ),
                VectorCommand::Cubic(
                    121.144348, 50.486957, 115.398261, 37.046957, 106.0, 27.064348,
                ),
                VectorCommand::Line(106.121739, 26.942609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.530435, 11.36),
                VectorCommand::Cubic(
                    102.518261, 19.029565, 117.565217, 39.116522, 117.565217, 62.733913,
                ),
                VectorCommand::Cubic(
                    117.565217, 92.925217, 92.973913, 117.516522, 62.782609, 117.516522,
                ),
                VectorCommand::Cubic(
                    39.14087, 117.516522, 18.956522, 102.566957, 11.408696, 81.481739,
                ),
                VectorCommand::Cubic(
                    19.833373, 107.734889, 46.114089, 123.949495, 73.356784, 119.702304,
                ),
                VectorCommand::Cubic(
                    100.59948, 115.455113, 120.697169, 92.010021, 120.730435, 64.438261,
                ),
                VectorCommand::Cubic(
                    120.721008, 40.084307, 104.972542, 18.528594, 81.773913, 11.116522,
                ),
                VectorCommand::Line(81.530435, 11.36),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 8.0,
                y2: 120.0,
                top: Color::from_hex(0xdfedf3),
                bottom: Color::from_hex(0xcadfe8),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(90.782609, 34.782609),
                VectorCommand::Cubic(
                    92.131478, 34.782609, 93.217391, 35.868522, 93.217391, 37.217391,
                ),
                VectorCommand::Line(93.217391, 95.652174),
                VectorCommand::Cubic(
                    93.217391, 97.001043, 92.131478, 98.086957, 90.782609, 98.086957,
                ),
                VectorCommand::Line(42.086957, 98.086957),
                VectorCommand::Cubic(
                    40.738087, 98.086957, 39.652174, 97.001043, 39.652174, 95.652174,
                ),
                VectorCommand::Line(90.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(102.956522, 23.89913),
                VectorCommand::Cubic(
                    111.989565, 33.833043, 117.492174, 47.053913, 117.492174, 61.638261,
                ),
                VectorCommand::Cubic(
                    117.492174, 92.56, 92.413913, 117.638261, 61.492174, 117.638261,
                ),
                VectorCommand::Cubic(
                    47.005217, 117.638261, 33.735652, 112.135652, 23.753043, 103.102609,
                ),
                VectorCommand::Cubic(
                    33.930435, 113.547826, 48.10087, 120.073043, 63.926957, 120.073043,
                ),
                VectorCommand::Cubic(
                    94.848696, 120.073043, 119.926957, 94.994783, 119.926957, 64.073043,
                ),
                VectorCommand::Cubic(
                    119.926957, 48.344348, 113.401739, 34.125217, 102.956522, 23.89913,
                ),
                VectorCommand::Line(102.956522, 23.89913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(90.782609, 85.913043),
                VectorCommand::Line(90.782609, 93.217391),
                VectorCommand::Cubic(
                    90.782609, 94.566261, 89.696696, 95.652174, 88.347826, 95.652174,
                ),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(64.0, 93.217391),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(39.652174, 95.652174),
                VectorCommand::Cubic(
                    38.303304, 95.652174, 37.217391, 94.566261, 37.217391, 93.217391,
                ),
                VectorCommand::Line(37.217391, 85.913043),
                VectorCommand::Line(90.782609, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(88, 113, 126, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.521739, 85.913043),
                VectorCommand::Line(83.478261, 85.913043),
                VectorCommand::Line(83.478261, 95.652174),
                VectorCommand::Line(44.521739, 95.652174),
                VectorCommand::Line(44.521739, 85.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(62, 86, 100, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(39.652174, 32.347826),
                VectorCommand::Line(88.347826, 32.347826),
                VectorCommand::Cubic(
                    89.696696, 32.347826, 90.782609, 33.433739, 90.782609, 34.782609,
                ),
                VectorCommand::Line(90.782609, 85.913043),
                VectorCommand::Cubic(
                    90.782609, 87.261913, 89.696696, 88.347826, 88.347826, 88.347826,
                ),
                VectorCommand::Line(39.652174, 88.347826),
                VectorCommand::Cubic(
                    38.303304, 88.347826, 37.217391, 87.261913, 37.217391, 85.913043,
                ),
                VectorCommand::Line(37.217391, 34.782609),
                VectorCommand::Cubic(
                    37.217391, 33.433739, 38.303304, 32.347826, 39.652174, 32.347826,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(139, 164, 176, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(50.608696, 90.782609),
                VectorCommand::Cubic(49.934261, 90.782609, 49.391304, 91.325565, 49.391304, 92.0),
                VectorCommand::Cubic(
                    49.391304, 92.674435, 49.934261, 93.217391, 50.608696, 93.217391,
                ),
                VectorCommand::Line(55.478261, 93.217391),
                VectorCommand::Cubic(56.152696, 93.217391, 56.695652, 92.674435, 56.695652, 92.0),
                VectorCommand::Cubic(
                    56.695652, 91.325565, 56.152696, 90.782609, 55.478261, 90.782609,
                ),
                VectorCommand::Line(50.608696, 90.782609),
                VectorCommand::Close,
                VectorCommand::Move(60.347826, 90.782609),
                VectorCommand::Cubic(59.673391, 90.782609, 59.130435, 91.325565, 59.130435, 92.0),
                VectorCommand::Cubic(
                    59.130435, 92.674435, 59.673391, 93.217391, 60.347826, 93.217391,
                ),
                VectorCommand::Line(77.391304, 93.217391),
                VectorCommand::Cubic(78.065739, 93.217391, 78.608696, 92.674435, 78.608696, 92.0),
                VectorCommand::Cubic(
                    78.608696, 91.325565, 78.065739, 90.782609, 77.391304, 90.782609,
                ),
                VectorCommand::Line(60.347826, 90.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(197, 153, 77, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(51.826087, 37.217391),
                VectorCommand::Cubic(
                    46.42087, 37.217391, 42.086957, 41.551304, 42.086957, 46.956522,
                ),
                VectorCommand::Line(42.086957, 57.353043),
                VectorCommand::Cubic(
                    43.542957, 58.193043, 44.521739, 59.763478, 44.521739, 61.565217,
                ),
                VectorCommand::Cubic(
                    44.521739, 63.366957, 43.542957, 64.949565, 42.086957, 65.777391,
                ),
                VectorCommand::Line(42.086957, 76.173913),
                VectorCommand::Line(49.391304, 83.478261),
                VectorCommand::Line(78.608696, 83.478261),
                VectorCommand::Line(85.913043, 76.173913),
                VectorCommand::Line(85.913043, 65.777391),
                VectorCommand::Cubic(
                    84.457043, 64.937391, 83.478261, 63.366957, 83.478261, 61.565217,
                ),
                VectorCommand::Cubic(
                    83.478261, 59.763478, 84.457043, 58.18087, 85.913043, 57.353043,
                ),
                VectorCommand::Line(85.913043, 46.956522),
                VectorCommand::Cubic(
                    85.913043, 41.551304, 81.57913, 37.217391, 76.173913, 37.217391,
                ),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(248, 251, 251, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(76.173913, 44.521739),
                VectorCommand::Cubic(
                    72.132174, 44.521739, 68.869565, 47.784348, 68.869565, 51.826087,
                ),
                VectorCommand::Line(56.695652, 64.0),
                VectorCommand::Cubic(52.653913, 64.0, 49.391304, 67.262609, 49.391304, 71.304348),
                VectorCommand::Cubic(
                    49.391334, 72.392696, 49.634783, 73.46887, 50.104696, 74.445217,
                ),
                VectorCommand::Line(54.974261, 69.575652),
                VectorCommand::Cubic(
                    55.926261, 68.626087, 57.457739, 68.626087, 58.407304, 69.575652,
                ),
                VectorCommand::Line(58.407304, 73.008696),
                VectorCommand::Line(53.537739, 77.878261),
                VectorCommand::Cubic(
                    54.521391, 78.348174, 55.597565, 78.591652, 56.678609, 78.591652,
                ),
                VectorCommand::Cubic(
                    60.720348, 78.591652, 63.982957, 75.329043, 63.982957, 71.287304,
                ),
                VectorCommand::Line(63.982957, 68.852522),
                VectorCommand::Line(73.722087, 59.113391),
                VectorCommand::Line(76.15687, 59.113391),
                VectorCommand::Cubic(
                    80.198609, 59.113391, 83.461217, 55.850783, 83.461217, 51.809043,
                ),
                VectorCommand::Cubic(
                    83.461188, 50.720696, 83.217739, 49.644522, 82.747826, 48.668174,
                ),
                VectorCommand::Line(77.878261, 53.537739),
                VectorCommand::Line(74.445217, 53.537739),
                VectorCommand::Cubic(
                    73.495652, 52.585739, 73.495652, 51.054261, 74.445217, 50.104696,
                ),
                VectorCommand::Line(79.314783, 45.23513),
                VectorCommand::Cubic(
                    78.33113, 44.765217, 77.254957, 44.521739, 76.173913, 44.521739,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(73.73913, 42.086957),
                VectorCommand::Cubic(
                    69.697391, 42.086957, 66.434783, 45.349565, 66.434783, 49.391304,
                ),
                VectorCommand::Line(66.434783, 51.826087),
                VectorCommand::Line(56.695652, 61.565217),
                VectorCommand::Line(54.26087, 61.565217),
                VectorCommand::Cubic(
                    50.21913, 61.565217, 46.956522, 64.827826, 46.956522, 68.869565,
                ),
                VectorCommand::Cubic(46.956551, 69.957913, 47.2, 71.034087, 47.669913, 72.010435),
                VectorCommand::Line(52.539478, 67.14087),
                VectorCommand::Cubic(
                    53.491478, 66.191304, 55.022957, 66.191304, 55.972522, 67.14087,
                ),
                VectorCommand::Cubic(
                    56.922087, 68.09287, 56.922087, 69.624348, 55.972522, 70.573913,
                ),
                VectorCommand::Line(51.102957, 75.443478),
                VectorCommand::Cubic(
                    52.086609, 75.913391, 53.162783, 76.15687, 54.243826, 76.15687,
                ),
                VectorCommand::Cubic(
                    58.285565, 76.15687, 61.548174, 72.894261, 61.548174, 68.852522,
                ),
                VectorCommand::Line(61.548174, 66.417739),
                VectorCommand::Line(71.287304, 56.678609),
                VectorCommand::Line(73.722087, 56.678609),
                VectorCommand::Cubic(
                    77.763826, 56.678609, 81.026435, 53.416, 81.026435, 49.374261,
                ),
                VectorCommand::Cubic(
                    81.026406, 48.285913, 80.782957, 47.209739, 80.313043, 46.233391,
                ),
                VectorCommand::Line(75.443478, 51.102957),
                VectorCommand::Cubic(74.491478, 52.052522, 72.96, 52.052522, 72.010435, 51.102957),
                VectorCommand::Cubic(
                    71.06087, 50.150957, 71.06087, 48.619478, 72.010435, 47.669913,
                ),
                VectorCommand::Line(76.88, 42.800348),
                VectorCommand::Cubic(
                    75.896348, 42.330435, 74.820174, 42.086957, 73.73913, 42.086957,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(98, 127, 144, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.678261, 54.504348),
                VectorCommand::Line(59.373913, 61.808696),
                VectorCommand::Cubic(
                    58.898665, 62.284087, 58.898665, 63.054696, 59.373913, 63.530087,
                ),
                VectorCommand::Cubic(
                    59.849304, 64.005335, 60.619913, 64.005335, 61.095304, 63.530087,
                ),
                VectorCommand::Line(68.399652, 56.225739),
                VectorCommand::Cubic(68.8749, 55.750348, 68.8749, 54.979739, 68.399652, 54.504348),
                VectorCommand::Cubic(67.924261, 54.0291, 67.153652, 54.0291, 66.678261, 54.504348),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(129, 157, 173, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_ABOUT: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xd2e8d1),
                bottom: Color::from_hex(0xe5f0de),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.473739, 29.917913),
                VectorCommand::Cubic(
                    46.30327, 29.917913, 29.954435, 46.266504, 29.954435, 66.437217,
                ),
                VectorCommand::Cubic(
                    29.954435, 86.607687, 46.303026, 102.956522, 66.473739, 102.956522,
                ),
                VectorCommand::Cubic(
                    86.644209, 102.956522, 102.993043, 86.60793, 102.993043, 66.437217,
                ),
                VectorCommand::Cubic(
                    102.993043, 46.266748, 86.644452, 29.917913, 66.473739, 29.917913,
                ),
                VectorCommand::Line(66.473739, 29.917913),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 36.521739,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 255, 249, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 29.217391,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(64, 132, 139, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(66.434783, 46.956522),
                VectorCommand::Cubic(
                    63.745322, 46.956522, 61.565217, 49.136699, 61.565217, 51.826087,
                ),
                VectorCommand::Cubic(
                    61.565217, 54.515475, 63.745395, 56.695652, 66.434783, 56.695652,
                ),
                VectorCommand::Cubic(
                    69.12417, 56.695652, 71.304348, 54.515475, 71.304348, 51.826087,
                ),
                VectorCommand::Cubic(
                    71.304348, 49.136699, 69.12417, 46.956522, 66.434783, 46.956522,
                ),
                VectorCommand::Close,
                VectorCommand::Move(64.0, 61.565217),
                VectorCommand::Cubic(62.65113, 61.565217, 61.565217, 62.65113, 61.565217, 64.0),
                VectorCommand::Line(61.565217, 83.478261),
                VectorCommand::Cubic(61.565217, 84.82713, 62.65113, 85.913043, 64.0, 85.913043),
                VectorCommand::Line(68.869565, 85.913043),
                VectorCommand::Cubic(
                    70.218435, 85.913043, 71.304348, 84.82713, 71.304348, 83.478261,
                ),
                VectorCommand::Line(71.304348, 64.0),
                VectorCommand::Cubic(
                    71.304348, 62.65113, 70.218435, 61.565217, 68.869565, 61.565217,
                ),
                VectorCommand::Line(64.0, 61.565217),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 44.521739),
                VectorCommand::Cubic(
                    61.310539, 44.521739, 59.130435, 46.701917, 59.130435, 49.391304,
                ),
                VectorCommand::Cubic(59.130435, 52.080692, 61.310612, 54.26087, 64.0, 54.26087),
                VectorCommand::Cubic(
                    66.689388, 54.26087, 68.869565, 52.080692, 68.869565, 49.391304,
                ),
                VectorCommand::Cubic(68.869565, 46.701917, 66.689388, 44.521739, 64.0, 44.521739),
                VectorCommand::Close,
                VectorCommand::Move(61.565217, 59.130435),
                VectorCommand::Cubic(
                    60.216348, 59.130435, 59.130435, 60.216348, 59.130435, 61.565217,
                ),
                VectorCommand::Line(59.130435, 81.043478),
                VectorCommand::Cubic(
                    59.130435, 82.392348, 60.216348, 83.478261, 61.565217, 83.478261,
                ),
                VectorCommand::Line(66.434783, 83.478261),
                VectorCommand::Cubic(
                    67.783652, 83.478261, 68.869565, 82.392348, 68.869565, 81.043478,
                ),
                VectorCommand::Line(68.869565, 61.565217),
                VectorCommand::Cubic(
                    68.869565, 60.216348, 67.783652, 59.130435, 66.434783, 59.130435,
                ),
                VectorCommand::Line(61.565217, 59.130435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 255, 250, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 34.782609),
                VectorCommand::Cubic(47.86368, 34.782609, 34.782609, 47.86368, 34.782609, 64.0),
                VectorCommand::Cubic(
                    34.788539, 72.523188, 38.515921, 80.618864, 44.987757, 86.165043,
                ),
                VectorCommand::Cubic(
                    40.018196, 80.793446, 37.245284, 73.752556, 37.217391, 66.434783,
                ),
                VectorCommand::Cubic(
                    37.217391, 50.298463, 50.298463, 37.217391, 66.434783, 37.217391,
                ),
                VectorCommand::Cubic(
                    73.590449, 37.221143, 80.495944, 39.850719, 85.841704, 44.607443,
                ),
                VectorCommand::Cubic(80.299534, 38.361902, 72.349995, 34.786041, 64.0, 34.782609),
                VectorCommand::Line(64.0, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_USB: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xcfdfef),
                bottom: Color::from_hex(0xe1edf8),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(32.347826, 66.434783),
                VectorCommand::Cubic(
                    32.347826, 70.469217, 35.617739, 73.73913, 39.652174, 73.73913,
                ),
                VectorCommand::Cubic(42.744348, 73.73913, 45.498087, 71.784, 46.528, 68.869565),
                VectorCommand::Line(55.478261, 68.869565),
                VectorCommand::Line(64.77913, 80.050087),
                VectorCommand::Cubic(
                    65.149217, 80.593043, 65.79687, 81.043478, 66.434783, 81.043478,
                ),
                VectorCommand::Line(71.28487, 81.043478),
                VectorCommand::Line(71.304348, 83.478261),
                VectorCommand::Line(81.043478, 83.478261),
                VectorCommand::Line(81.043478, 73.73913),
                VectorCommand::Line(71.304348, 73.73913),
                VectorCommand::Line(71.304348, 76.173913),
                VectorCommand::Line(67.652174, 76.173913),
                VectorCommand::Line(61.565217, 68.869565),
                VectorCommand::Line(85.913043, 68.869565),
                VectorCommand::Line(85.913043, 73.73913),
                VectorCommand::Line(98.086957, 66.434783),
                VectorCommand::Line(85.913043, 59.130435),
                VectorCommand::Line(85.913043, 64.0),
                VectorCommand::Line(55.478261, 64.0),
                VectorCommand::Line(61.565217, 56.695652),
                VectorCommand::Line(67.094609, 56.695652),
                VectorCommand::Cubic(
                    67.963826, 58.200348, 69.565913, 59.130435, 71.304348, 59.130435,
                ),
                VectorCommand::Cubic(
                    73.994783, 59.13287, 76.173913, 56.951304, 76.173913, 54.26087,
                ),
                VectorCommand::Cubic(
                    76.173913, 51.570435, 73.994783, 49.391304, 71.304348, 49.391304,
                ),
                VectorCommand::Cubic(
                    69.565913, 49.391304, 67.961391, 50.321391, 67.092174, 51.826087,
                ),
                VectorCommand::Line(60.347826, 51.826087),
                VectorCommand::Cubic(
                    59.74887, 51.826087, 59.084174, 52.176696, 58.692174, 52.653913,
                ),
                VectorCommand::Line(49.391304, 64.0),
                VectorCommand::Line(46.53287, 64.0),
                VectorCommand::Cubic(
                    45.500522, 61.08313, 42.744348, 59.130435, 39.652174, 59.130435,
                ),
                VectorCommand::Cubic(
                    35.617739, 59.128, 32.347826, 62.400348, 32.347826, 66.434783,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(29.913043, 64.0),
                VectorCommand::Cubic(
                    29.913043, 68.034435, 33.182957, 71.304348, 37.217391, 71.304348,
                ),
                VectorCommand::Cubic(
                    40.309565, 71.304348, 43.063304, 69.349217, 44.093217, 66.434783,
                ),
                VectorCommand::Line(53.043478, 66.434783),
                VectorCommand::Line(62.344348, 77.615304),
                VectorCommand::Cubic(62.714435, 78.158261, 63.362087, 78.608696, 64.0, 78.608696),
                VectorCommand::Line(68.850087, 78.608696),
                VectorCommand::Line(68.869565, 81.043478),
                VectorCommand::Line(78.608696, 81.043478),
                VectorCommand::Line(78.608696, 71.304348),
                VectorCommand::Line(68.869565, 71.304348),
                VectorCommand::Line(68.869565, 73.73913),
                VectorCommand::Line(65.217391, 73.73913),
                VectorCommand::Line(59.130435, 66.434783),
                VectorCommand::Line(83.478261, 66.434783),
                VectorCommand::Line(83.478261, 71.304348),
                VectorCommand::Line(95.652174, 64.0),
                VectorCommand::Line(83.478261, 56.695652),
                VectorCommand::Line(83.478261, 61.565217),
                VectorCommand::Line(53.043478, 61.565217),
                VectorCommand::Line(59.130435, 54.26087),
                VectorCommand::Line(64.659826, 54.26087),
                VectorCommand::Cubic(
                    65.529043, 55.765565, 67.13113, 56.695652, 68.869565, 56.695652,
                ),
                VectorCommand::Cubic(71.56, 56.698087, 73.73913, 54.516522, 73.73913, 51.826087),
                VectorCommand::Cubic(73.73913, 49.135652, 71.56, 46.956522, 68.869565, 46.956522),
                VectorCommand::Cubic(
                    67.13113, 46.956522, 65.526609, 47.886609, 64.657391, 49.391304,
                ),
                VectorCommand::Line(57.913043, 49.391304),
                VectorCommand::Cubic(
                    57.314087, 49.391304, 56.649391, 49.741913, 56.257391, 50.21913,
                ),
                VectorCommand::Line(46.956522, 61.565217),
                VectorCommand::Line(44.098087, 61.565217),
                VectorCommand::Cubic(
                    43.065739, 58.648348, 40.309565, 56.695652, 37.217391, 56.695652,
                ),
                VectorCommand::Cubic(33.182957, 56.693217, 29.913043, 59.965565, 29.913043, 64.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(60, 112, 157, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_BATTERY: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xc6e2d9),
                bottom: Color::from_hex(0xddefe6),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(61.565217, 32.347826),
                VectorCommand::Cubic(
                    60.216348, 32.347826, 59.130435, 33.433739, 59.130435, 34.782609,
                ),
                VectorCommand::Line(59.130435, 37.217391),
                VectorCommand::Line(51.826087, 37.217391),
                VectorCommand::Cubic(
                    49.128348, 37.217391, 46.956522, 39.389217, 46.956522, 42.086957,
                ),
                VectorCommand::Line(46.956522, 95.652174),
                VectorCommand::Cubic(
                    46.956522, 98.349913, 49.128348, 100.521739, 51.826087, 100.521739,
                ),
                VectorCommand::Line(81.043478, 100.521739),
                VectorCommand::Cubic(
                    83.741217, 100.521739, 85.913043, 98.349913, 85.913043, 95.652174,
                ),
                VectorCommand::Line(85.913043, 42.086957),
                VectorCommand::Cubic(
                    85.913043, 39.389217, 83.741217, 37.217391, 81.043478, 37.217391,
                ),
                VectorCommand::Line(73.73913, 37.217391),
                VectorCommand::Line(73.73913, 34.782609),
                VectorCommand::Cubic(
                    73.73913, 33.433739, 72.653217, 32.347826, 71.304348, 32.347826,
                ),
                VectorCommand::Line(61.565217, 32.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(59.130435, 29.913043),
                VectorCommand::Cubic(
                    57.781565, 29.913043, 56.695652, 30.998957, 56.695652, 32.347826,
                ),
                VectorCommand::Line(56.695652, 34.782609),
                VectorCommand::Line(49.391304, 34.782609),
                VectorCommand::Cubic(
                    46.693565, 34.782609, 44.521739, 36.954435, 44.521739, 39.652174,
                ),
                VectorCommand::Line(44.521739, 93.217391),
                VectorCommand::Cubic(
                    44.521739, 95.91513, 46.693565, 98.086957, 49.391304, 98.086957,
                ),
                VectorCommand::Line(78.608696, 98.086957),
                VectorCommand::Cubic(
                    81.306435, 98.086957, 83.478261, 95.91513, 83.478261, 93.217391,
                ),
                VectorCommand::Line(83.478261, 39.652174),
                VectorCommand::Cubic(
                    83.478261, 36.954435, 81.306435, 34.782609, 78.608696, 34.782609,
                ),
                VectorCommand::Line(71.304348, 34.782609),
                VectorCommand::Line(71.304348, 32.347826),
                VectorCommand::Cubic(
                    71.304348, 30.998957, 70.218435, 29.913043, 68.869565, 29.913043,
                ),
                VectorCommand::Line(59.130435, 29.913043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(73, 111, 96, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(49.391304, 78.608696),
                VectorCommand::Line(49.391304, 42.086957),
                VectorCommand::Cubic(
                    49.391304, 40.738087, 50.477217, 39.652174, 51.826087, 39.652174,
                ),
                VectorCommand::Line(76.173913, 39.652174),
                VectorCommand::Cubic(
                    77.522783, 39.652174, 78.608696, 40.738087, 78.608696, 42.086957,
                ),
                VectorCommand::Line(78.608696, 49.391304),
                VectorCommand::Line(64.0, 68.869565),
                VectorCommand::Line(49.391304, 78.608696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(229, 237, 223, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(78.608696, 49.391304),
                VectorCommand::Line(78.608696, 90.782609),
                VectorCommand::Cubic(
                    78.608696, 92.131478, 77.522783, 93.217391, 76.173913, 93.217391,
                ),
                VectorCommand::Line(51.826087, 93.217391),
                VectorCommand::Cubic(
                    50.477217, 93.217391, 49.391304, 92.131478, 49.391304, 90.782609,
                ),
                VectorCommand::Line(49.391304, 78.608696),
                VectorCommand::Line(78.608696, 49.391304),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(152, 179, 137, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_BRIGHTNESS: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 53.565217,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 117.565169,
                y2: 10.434929,
                top: Color::from_hex(0xf5f5ec),
                bottom: Color::from_hex(0xfffff5),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(44.940522, 86.122435),
                VectorCommand::Cubic(
                    40.051235, 80.79367, 37.2208, 73.805357, 37.217635, 66.435026,
                ),
                VectorCommand::Cubic(
                    37.216386, 63.369148, 37.710849, 60.323235, 38.663287, 57.409287,
                ),
                VectorCommand::Cubic(
                    42.571843, 45.375617, 53.782557, 37.216174, 66.434417, 37.217635,
                ),
                VectorCommand::Cubic(
                    73.790139, 37.218565, 80.644052, 40.003026, 85.869826, 44.679026,
                ),
                VectorCommand::Cubic(
                    74.93473, 27.557148, 41.724783, 19.863722, 29.913652, 32.34807,
                ),
                VectorCommand::Cubic(
                    18.101791, 44.832904, 27.688115, 77.496243, 44.940887, 86.122678,
                ),
                VectorCommand::Line(44.940522, 86.122435),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Cubic(39.114087, 8.0, 18.031304, 24.236348, 10.73913, 46.69113),
                VectorCommand::Cubic(
                    12.869565, 54.26087, 27.478261, 56.695652, 36.22887, 54.974991,
                ),
                VectorCommand::Cubic(40.137446, 42.941302, 51.347467, 34.790794, 64.0, 34.783339),
                VectorCommand::Cubic(68.869565, 29.913043, 68.869565, 12.869565, 64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(86, 104, 122, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(64.0, 8.0),
                VectorCommand::Line(64.0, 34.782609),
                VectorCommand::Cubic(
                    76.647895, 34.799896, 87.848625, 42.953133, 91.751652, 54.983757,
                ),
                VectorCommand::Cubic(
                    98.086957, 59.130435, 112.695652, 54.26087, 117.26087, 46.695026,
                ),
                VectorCommand::Cubic(109.969426, 24.238783, 88.888348, 7.999026, 64.0, 7.999026),
                VectorCommand::Line(64.0, 8.0),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(129, 147, 162, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(117.26087, 46.696),
                VectorCommand::Line(91.751652, 54.98473),
                VectorCommand::Cubic(
                    92.709569, 57.894783, 93.20383, 60.937287, 93.21632, 64.000974,
                ),
                VectorCommand::Cubic(
                    93.202929, 73.347861, 88.718546, 82.124522, 81.151729, 87.611791,
                ),
                VectorCommand::Cubic(
                    78.608598, 95.652174, 88.347729, 110.26087, 96.911346, 109.301322,
                ),
                VectorCommand::Cubic(
                    110.899659, 99.120522, 119.998929, 82.625843, 119.998929, 64.002191,
                ),
                VectorCommand::Cubic(
                    119.998929, 57.960765, 119.029788, 52.148452, 117.259798, 46.697217,
                ),
                VectorCommand::Line(117.26087, 46.696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(170, 183, 193, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(46.842087, 87.610087),
                VectorCommand::Cubic(
                    37.217391, 88.347826, 29.913043, 102.956522, 31.087339, 109.294748,
                ),
                VectorCommand::Cubic(
                    40.327583, 116.020591, 51.695583, 119.99927, 64.00073, 119.99927,
                ),
                VectorCommand::Cubic(
                    76.30393, 119.99927, 87.673148, 116.024487, 96.914122, 109.299617,
                ),
                VectorCommand::Line(81.154504, 87.610087),
                VectorCommand::Cubic(
                    76.171722, 91.242052, 70.167548, 93.204487, 64.001704, 93.216661,
                ),
                VectorCommand::Cubic(
                    57.834157, 93.205704, 51.828035, 91.24305, 46.844035, 87.610087,
                ),
                VectorCommand::Line(46.842087, 87.610087),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(203, 212, 218, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(10.73913, 46.69113),
                VectorCommand::Cubic(8.96824, 52.143583, 8.0, 57.957843, 8.0, 64.000974),
                VectorCommand::Cubic(8.0, 82.625843, 17.100487, 99.115409, 31.087583, 109.29767),
                VectorCommand::Line(46.84233, 87.613009),
                VectorCommand::Cubic(
                    39.277456, 82.124578, 34.794795, 73.348316, 34.782609, 64.002191,
                ),
                VectorCommand::Cubic(
                    34.78797, 60.936398, 35.275823, 57.890552, 36.228261, 54.976452,
                ),
                VectorCommand::Line(10.73913, 46.69113),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(225, 231, 235, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static NUMIX_LIGHT_AUDIO: VectorIcon = VectorIcon {
    width: 128.0,
    height: 128.0,
    layers: &[
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(93.972174, 17.73913),
                VectorCommand::Cubic(
                    108.237565, 27.629217, 117.560348, 44.110261, 117.560348, 62.782609,
                ),
                VectorCommand::Cubic(
                    117.560348, 93.037217, 93.042087, 117.565217, 62.777739, 117.565217,
                ),
                VectorCommand::Cubic(
                    44.105391, 117.565217, 27.624348, 108.24487, 17.734261, 93.977043,
                ),
                VectorCommand::Cubic(
                    20.25913, 98.413217, 23.34887, 102.496348, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.886261, 115.113391, 50.104696, 120.608696, 64.603826, 120.608696,
                ),
                VectorCommand::Cubic(
                    95.53287, 120.608696, 120.603826, 95.535304, 120.603826, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.603826, 50.112, 115.110957, 36.89113, 106.070609, 26.945043,
                ),
                VectorCommand::Cubic(
                    102.493913, 23.353739, 98.408348, 20.264, 93.972174, 17.73913,
                ),
                VectorCommand::Close,
                VectorCommand::Move(106.070609, 26.945043),
                VectorCommand::Cubic(
                    115.454261, 36.959304, 121.212522, 50.409043, 121.212522, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.212522, 96.146435, 96.141565, 121.217391, 65.212522, 121.217391,
                ),
                VectorCommand::Cubic(
                    50.406609, 121.217391, 36.954435, 115.461565, 26.940174, 106.075478,
                ),
                VectorCommand::Cubic(
                    37.06887, 116.165217, 51.00313, 122.434783, 66.429913, 122.434783,
                ),
                VectorCommand::Cubic(
                    97.358957, 122.434783, 122.429913, 97.361391, 122.429913, 66.434783,
                ),
                VectorCommand::Cubic(
                    122.429913, 51.010435, 116.160348, 37.071304, 106.070609, 26.945043,
                ),
                VectorCommand::Line(106.070609, 26.945043),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 6))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(106.073043, 26.945043),
                VectorCommand::Cubic(
                    115.110957, 36.89113, 120.606261, 50.109565, 120.606261, 64.608696,
                ),
                VectorCommand::Cubic(
                    120.606261, 95.537739, 95.535304, 120.608696, 64.606261, 120.608696,
                ),
                VectorCommand::Cubic(
                    50.109565, 120.608696, 36.888696, 115.115826, 26.942609, 106.075478,
                ),
                VectorCommand::Cubic(
                    36.95687, 115.45913, 50.406609, 121.217391, 65.214957, 121.217391,
                ),
                VectorCommand::Cubic(
                    96.144, 121.217391, 121.214957, 96.144, 121.214957, 65.217391,
                ),
                VectorCommand::Cubic(
                    121.214957, 50.411478, 115.45913, 36.959304, 106.073043, 26.945043,
                ),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(81.652174, 11.347826),
                VectorCommand::Cubic(
                    102.627826, 19.02713, 117.565217, 39.145739, 117.565217, 62.794783,
                ),
                VectorCommand::Cubic(
                    117.565217, 93.049391, 93.046957, 117.577391, 62.782609, 117.577391,
                ),
                VectorCommand::Cubic(39.145739, 117.577391, 19.02713, 102.64, 11.36, 81.664348),
                VectorCommand::Cubic(
                    19.854275, 107.818778, 46.08181, 123.935616, 73.252448, 119.697299,
                ),
                VectorCommand::Cubic(
                    100.423086, 115.458982, 120.495922, 92.119802, 120.62087, 64.62087,
                ),
                VectorCommand::Cubic(
                    120.584996, 40.289371, 104.841617, 18.76522, 81.664348, 11.36,
                ),
                VectorCommand::Line(81.652174, 11.347826),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 23))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 56.0,
            },
            fill: Some(VectorPaint::VerticalGradient {
                y1: 120.0,
                y2: 8.0,
                top: Color::from_hex(0xebd2b7),
                bottom: Color::from_hex(0xf6e5ce),
            }),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(103.029565, 23.901565),
                VectorCommand::Cubic(
                    112.067478, 33.845217, 117.562783, 47.066087, 117.562783, 61.565217,
                ),
                VectorCommand::Cubic(
                    117.562783, 92.494261, 92.491826, 117.565217, 61.562783, 117.565217,
                ),
                VectorCommand::Cubic(
                    47.066087, 117.565217, 33.845217, 112.072348, 23.89913, 103.032,
                ),
                VectorCommand::Cubic(34.071652, 113.479652, 48.271304, 120.0, 63.997565, 120.0),
                VectorCommand::Cubic(94.926609, 120.0, 119.997565, 94.926609, 119.997565, 64.0),
                VectorCommand::Cubic(
                    119.997565, 48.266435, 113.479652, 34.074087, 103.029565, 23.901565,
                ),
                VectorCommand::Line(103.029565, 23.901565),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782334, 93.216534),
                VectorCommand::Cubic(
                    34.782334, 95.905968, 36.962344, 98.086051, 39.651851, 98.086051,
                ),
                VectorCommand::Cubic(
                    40.351625, 98.086051, 41.009692, 97.928571, 41.611077, 97.662817,
                ),
                VectorCommand::Cubic(
                    58.159888, 104.692939, 74.709185, 104.696104, 91.25337, 97.667687,
                ),
                VectorCommand::Cubic(
                    91.854195, 97.932856, 92.518373, 98.086173, 93.217343, 98.086173,
                ),
                VectorCommand::Cubic(
                    95.906777, 98.086173, 98.08686, 95.906139, 98.08686, 93.216656,
                ),
                VectorCommand::Cubic(
                    98.08686, 92.447686, 97.890667, 91.728117, 97.573272, 91.081494,
                ),
                VectorCommand::Cubic(
                    104.735115, 73.120524, 104.735115, 59.748586, 97.573272, 41.787372,
                ),
                VectorCommand::Cubic(
                    97.891812, 41.139799, 98.08686, 40.422666, 98.08686, 39.652211,
                ),
                VectorCommand::Cubic(
                    98.08686, 36.962776, 95.906851, 34.782694, 93.217343, 34.782694,
                ),
                VectorCommand::Cubic(
                    93.214178, 34.783112, 34.783138, 93.216899, 34.783138, 93.216899,
                ),
                VectorCommand::Line(34.782334, 93.216534),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(0, 0, 0, 11))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(34.782609, 34.782609),
                VectorCommand::Cubic(
                    25.042261, 56.696139, 25.042261, 71.301913, 34.782609, 93.214957,
                ),
                VectorCommand::Cubic(
                    54.263061, 102.955304, 73.744, 102.955304, 93.214957, 93.214957,
                ),
                VectorCommand::Cubic(
                    102.955304, 71.301426, 102.955304, 56.695652, 93.214957, 34.782609,
                ),
                VectorCommand::Cubic(
                    73.734504, 25.042261, 54.253565, 25.042261, 34.782609, 34.782609,
                ),
                VectorCommand::Line(34.782609, 34.782609),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(249, 247, 239, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 31.652174,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(107, 120, 121, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 24.347826,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(212, 168, 102, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 37.214956,
                y: 37.217391,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(249, 247, 239, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 37.214956,
                y: 90.782609,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(249, 247, 239, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 90.780174,
                y: 90.782609,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(249, 247, 239, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 90.780174,
                y: 37.217391,
                radius: 4.869565,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(249, 247, 239, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 63.997565,
                y: 64.0,
                radius: 17.043478,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(70, 89, 102, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 61.562782,
                y: 59.130435,
                radius: 4.869566,
            },
            fill: Some(VectorPaint::Solid(Color::from_rgba(255, 251, 240, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Path(&[
                VectorCommand::Move(37.032348, 34.787478),
                VectorCommand::Cubic(
                    35.760893, 34.882144, 34.777753, 35.942032, 34.778275, 37.217513,
                ),
                VectorCommand::Cubic(
                    34.778275, 38.562744, 35.868364, 39.65327, 37.213057, 39.65327,
                ),
                VectorCommand::Cubic(
                    38.557751, 39.65327, 39.64784, 38.562744, 39.64784, 37.217513,
                ),
                VectorCommand::Cubic(
                    39.648116, 36.539936, 39.366252, 35.892865, 38.869968, 35.431763,
                ),
                VectorCommand::Cubic(
                    38.373684, 34.97066, 37.707836, 34.737202, 37.032397, 34.787478,
                ),
                VectorCommand::Line(37.032348, 34.787478),
                VectorCommand::Close,
                VectorCommand::Move(90.597565, 34.787478),
                VectorCommand::Cubic(
                    89.32611, 34.882144, 88.342971, 35.942032, 88.343492, 37.217513,
                ),
                VectorCommand::Cubic(
                    88.343492, 38.562744, 89.433581, 39.65327, 90.778275, 39.65327,
                ),
                VectorCommand::Cubic(
                    92.122968, 39.65327, 93.213057, 38.562744, 93.213057, 37.217513,
                ),
                VectorCommand::Cubic(
                    93.213333, 36.539936, 92.93147, 35.892865, 92.435186, 35.431763,
                ),
                VectorCommand::Cubic(
                    91.938901, 34.97066, 91.273053, 34.737202, 90.597614, 34.787478,
                ),
                VectorCommand::Line(90.597565, 34.787478),
                VectorCommand::Close,
                VectorCommand::Move(37.032348, 88.352696),
                VectorCommand::Cubic(
                    35.760893, 88.447362, 34.777753, 89.507249, 34.778275, 90.78273,
                ),
                VectorCommand::Cubic(
                    34.778275, 92.127962, 35.868364, 93.218487, 37.213057, 93.218487,
                ),
                VectorCommand::Cubic(
                    38.557751, 93.218487, 39.64784, 92.127962, 39.64784, 90.78273,
                ),
                VectorCommand::Cubic(
                    39.648116, 90.105153, 39.366252, 89.458083, 38.869968, 88.99698,
                ),
                VectorCommand::Cubic(
                    38.373684, 88.535877, 37.707836, 88.302419, 37.032397, 88.352696,
                ),
                VectorCommand::Line(37.032348, 88.352696),
                VectorCommand::Close,
                VectorCommand::Move(90.597565, 88.352696),
                VectorCommand::Cubic(
                    89.32611, 88.447362, 88.342971, 89.507249, 88.343492, 90.78273,
                ),
                VectorCommand::Cubic(
                    88.343492, 92.127962, 89.433581, 93.218487, 90.778275, 93.218487,
                ),
                VectorCommand::Cubic(
                    92.122968, 93.218487, 93.213057, 92.127962, 93.213057, 90.78273,
                ),
                VectorCommand::Cubic(
                    93.213333, 90.105153, 92.93147, 89.458083, 92.435186, 88.99698,
                ),
                VectorCommand::Cubic(
                    91.938901, 88.535877, 91.273053, 88.302419, 90.597614, 88.352696,
                ),
                VectorCommand::Line(90.597565, 88.352696),
                VectorCommand::Close,
            ]),
            fill: Some(VectorPaint::Solid(Color::from_rgba(153, 168, 174, 255))),
            stroke: None,
            stroke_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
        VectorLayer {
            shape: VectorShape::Circle {
                x: 64.0,
                y: 64.0,
                radius: 55.513043,
            },
            fill: None,
            stroke: Some(VectorPaint::Solid(Color::from_rgba(66, 85, 106, 41))),
            stroke_width: 0.973913,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            fill_rule: FillRule::Winding,
        },
    ],
};
pub static ALL_NUMIX_ICONS: &[(&str, &VectorIcon)] = &[
    ("NUMIX_DARK_CLOCK", &NUMIX_DARK_CLOCK),
    ("NUMIX_DARK_TIMER", &NUMIX_DARK_TIMER),
    ("NUMIX_DARK_NOTES", &NUMIX_DARK_NOTES),
    ("NUMIX_DARK_CALCULATOR", &NUMIX_DARK_CALCULATOR),
    ("NUMIX_DARK_MAC", &NUMIX_DARK_MAC),
    ("NUMIX_DARK_SETTINGS", &NUMIX_DARK_SETTINGS),
    ("NUMIX_DARK_DISPLAY", &NUMIX_DARK_DISPLAY),
    ("NUMIX_DARK_SCREEN", &NUMIX_DARK_SCREEN),
    ("NUMIX_DARK_FILE_MANAGER", &NUMIX_DARK_FILE_MANAGER),
    ("NUMIX_DARK_OFFICE_VIEWER", &NUMIX_DARK_OFFICE_VIEWER),
    ("NUMIX_DARK_SUB2API_MONITOR", &NUMIX_DARK_SUB2API_MONITOR),
    ("NUMIX_DARK_WIFI", &NUMIX_DARK_WIFI),
    ("NUMIX_DARK_BLUETOOTH", &NUMIX_DARK_BLUETOOTH),
    ("NUMIX_DARK_DISPLAY_SETTINGS", &NUMIX_DARK_DISPLAY_SETTINGS),
    ("NUMIX_DARK_DATE_TIME", &NUMIX_DARK_DATE_TIME),
    ("NUMIX_DARK_STORAGE", &NUMIX_DARK_STORAGE),
    ("NUMIX_DARK_ABOUT", &NUMIX_DARK_ABOUT),
    ("NUMIX_DARK_USB", &NUMIX_DARK_USB),
    ("NUMIX_DARK_BATTERY", &NUMIX_DARK_BATTERY),
    ("NUMIX_DARK_BRIGHTNESS", &NUMIX_DARK_BRIGHTNESS),
    ("NUMIX_DARK_AUDIO", &NUMIX_DARK_AUDIO),
    ("NUMIX_LIGHT_CLOCK", &NUMIX_LIGHT_CLOCK),
    ("NUMIX_LIGHT_TIMER", &NUMIX_LIGHT_TIMER),
    ("NUMIX_LIGHT_NOTES", &NUMIX_LIGHT_NOTES),
    ("NUMIX_LIGHT_CALCULATOR", &NUMIX_LIGHT_CALCULATOR),
    ("NUMIX_LIGHT_MAC", &NUMIX_LIGHT_MAC),
    ("NUMIX_LIGHT_SETTINGS", &NUMIX_LIGHT_SETTINGS),
    ("NUMIX_LIGHT_DISPLAY", &NUMIX_LIGHT_DISPLAY),
    ("NUMIX_LIGHT_SCREEN", &NUMIX_LIGHT_SCREEN),
    ("NUMIX_LIGHT_FILE_MANAGER", &NUMIX_LIGHT_FILE_MANAGER),
    ("NUMIX_LIGHT_OFFICE_VIEWER", &NUMIX_LIGHT_OFFICE_VIEWER),
    ("NUMIX_LIGHT_SUB2API_MONITOR", &NUMIX_LIGHT_SUB2API_MONITOR),
    ("NUMIX_LIGHT_WIFI", &NUMIX_LIGHT_WIFI),
    ("NUMIX_LIGHT_BLUETOOTH", &NUMIX_LIGHT_BLUETOOTH),
    (
        "NUMIX_LIGHT_DISPLAY_SETTINGS",
        &NUMIX_LIGHT_DISPLAY_SETTINGS,
    ),
    ("NUMIX_LIGHT_DATE_TIME", &NUMIX_LIGHT_DATE_TIME),
    ("NUMIX_LIGHT_STORAGE", &NUMIX_LIGHT_STORAGE),
    ("NUMIX_LIGHT_ABOUT", &NUMIX_LIGHT_ABOUT),
    ("NUMIX_LIGHT_USB", &NUMIX_LIGHT_USB),
    ("NUMIX_LIGHT_BATTERY", &NUMIX_LIGHT_BATTERY),
    ("NUMIX_LIGHT_BRIGHTNESS", &NUMIX_LIGHT_BRIGHTNESS),
    ("NUMIX_LIGHT_AUDIO", &NUMIX_LIGHT_AUDIO),
];
