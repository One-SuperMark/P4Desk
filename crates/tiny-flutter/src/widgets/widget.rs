use crate::graphics::geometry::EdgeInsets;
use crate::rendering::render_box::RenderBox;
use crate::widgets::basic::{Center, Expanded, Padding, SizedBox};

/// Core declarative Widget trait. Every UI element in tiny-flutter implements Widget.
pub trait Widget: Send + Sync {
    /// Instantiate or update the underlying RenderBox for this widget.
    fn create_render_object(&self) -> Box<dyn RenderBox>;
}

impl Widget for Box<dyn Widget> {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        (**self).create_render_object()
    }
}

impl<T: Widget + Clone + 'static> From<T> for Box<dyn Widget> {
    fn from(w: T) -> Self {
        Box::new(w)
    }
}

/// Ergonomic extension trait providing fluent modifier methods for all widgets.
pub trait WidgetExt: Widget + Sized + 'static {
    /// Wrap this widget in a Center widget.
    fn centered(self) -> Center {
        Center::new(self)
    }

    /// Wrap this widget with custom EdgeInsets padding.
    fn padding(self, insets: EdgeInsets) -> Padding {
        Padding::new(insets, self)
    }

    /// Wrap this widget with uniform padding on all sides.
    fn padding_all(self, val: f32) -> Padding {
        Padding::new(EdgeInsets::all(val), self)
    }

    /// Wrap this widget with symmetric vertical and horizontal padding.
    fn padding_symmetric(self, vertical: f32, horizontal: f32) -> Padding {
        Padding::new(EdgeInsets::symmetric(vertical, horizontal), self)
    }

    /// Wrap this widget in an Expanded widget with a flex factor.
    fn expanded(self, flex: u32) -> Expanded {
        Expanded::new(self).flex(flex)
    }

    /// Wrap this widget in a SizedBox with explicit dimensions.
    fn sized(self, width: f32, height: f32) -> SizedBox {
        SizedBox::with_child(Some(width), Some(height), self)
    }

    /// Wrap this widget in a square SizedBox.
    fn sized_square(self, dim: f32) -> SizedBox {
        SizedBox::with_child(Some(dim), Some(dim), self)
    }
}

impl<W: Widget + Sized + 'static> WidgetExt for W {}
