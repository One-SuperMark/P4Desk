use crate::graphics::baked_icons::BakedIcon;
use crate::graphics::canvas::Canvas;
use crate::graphics::color::Color;
use crate::graphics::geometry::{Offset, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::RenderBox;
use crate::widgets::widget::Widget;

/// A scalable SVG icon, rasterized at the widget's actual size.
pub struct Icon {
    pub icon: BakedIcon,
    pub color: Color,
    pub size: Option<Size>,
}

impl Icon {
    pub fn new(icon: BakedIcon) -> Self {
        Self {
            icon,
            color: Color::WHITE,
            size: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }
}

impl Widget for Icon {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let size = self
            .size
            .unwrap_or_else(|| Size::new(self.icon.width as f32, self.icon.height as f32));
        Box::new(RenderIcon {
            icon: self.icon,
            color: self.color,
            size,
            offset: Offset::ZERO,
        })
    }
}

pub struct RenderIcon {
    icon: BakedIcon,
    color: Color,
    size: Size,
    offset: Offset,
}

impl RenderBox for RenderIcon {
    fn size(&self) -> Size {
        self.size
    }

    fn offset(&self) -> Offset {
        self.offset
    }

    fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    fn layout(&mut self, constraints: &BoxConstraints) -> Size {
        self.size = constraints.constrain(self.size);
        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        self.icon.vector.paint(
            canvas,
            Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height),
            self.color,
        );
    }
}
