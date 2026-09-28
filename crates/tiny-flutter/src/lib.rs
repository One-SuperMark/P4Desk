pub mod app;
pub mod graphics;
pub mod platform;
pub mod rendering;
pub mod theme;
pub mod widgets;

pub use app::App;
pub use graphics::*;
#[cfg(feature = "simulator")]
pub use platform::{run_simulator, run_simulator_animated, SimulatorBackend};
pub use platform::{EspWasmBackend, PlatformBackend, RawTouchFrame, RawTouchPoint};
pub use rendering::{
    Axis, BoxConstraints, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, RenderBox,
    TouchEvent,
};
pub use theme::{ColorScheme, ThemeData};
pub use tiny_gfx;
pub use tiny_gfx as tiny_skia;
pub use widgets::{
    BackListener, ButtonStyle, Center, Column, Container, CustomPaint, CustomPainter,
    ElevatedButton, Expanded, FractionallySizedBox, GestureDetector, Icon, Padding, PageController,
    PageTransition, PageView, Positioned, Row, ScrollController, SingleChildScrollView, SizedBox,
    Stack, Text, TextStyle, Widget, WidgetExt,
};

pub mod prelude {
    pub use crate::app::App;
    pub use crate::graphics::*;
    #[cfg(feature = "simulator")]
    pub use crate::platform::{run_simulator, run_simulator_animated, SimulatorBackend};
    pub use crate::platform::{EspWasmBackend, PlatformBackend, RawTouchFrame, RawTouchPoint};
    pub use crate::rendering::{
        Axis, BoxConstraints, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, RenderBox,
        TouchEvent,
    };
    pub use crate::theme::{ColorScheme, ThemeData};
    pub use crate::tiny_gfx;
    pub use crate::tiny_skia;
    pub use crate::widgets::{
        BackListener, ButtonStyle, Center, Column, Container, CustomPaint, CustomPainter,
        ElevatedButton, Expanded, FractionallySizedBox, GestureDetector, Icon, Padding,
        PageController, PageTransition, PageView, Positioned, Row, ScrollController,
        SingleChildScrollView, SizedBox, Stack, Text, TextStyle, Widget, WidgetExt,
    };
    pub use crate::{col, row};
}
