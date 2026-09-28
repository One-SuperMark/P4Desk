use crate::graphics::font::TextLayout;
use crate::graphics::{Canvas, Color, Font, Offset, Point, Size};
use crate::rendering::{BoxConstraints, RenderBox};
use crate::widgets::Widget;

#[derive(Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub color: Color,
    pub font: Option<Font>,
    pub wrap: bool,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            color: Color::WHITE,
            font: None,
            wrap: false,
        }
    }
}
pub struct Text {
    pub content: String,
    pub style: TextStyle,
}
impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
        }
    }
    pub fn font_size(mut self, v: f32) -> Self {
        self.style.font_size = v;
        self
    }
    pub fn color(mut self, v: Color) -> Self {
        self.style.color = v;
        self
    }
    pub fn font(mut self, v: Font) -> Self {
        self.style.font = Some(v);
        self
    }
    pub fn wrap(mut self) -> Self {
        self.style.wrap = true;
        self
    }
}
impl Widget for Text {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        Box::new(RenderText {
            content: self.content.clone(),
            style: self.style.clone(),
            size: Size::ZERO,
            offset: Offset::ZERO,
            layout: None,
        })
    }
}
pub struct RenderText {
    pub content: String,
    pub style: TextStyle,
    size: Size,
    offset: Offset,
    layout: Option<TextLayout>,
}
impl RenderBox for RenderText {
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, v: Offset) {
        self.offset = v;
    }
    fn layout(&mut self, c: &BoxConstraints) -> Size {
        let f = self
            .style
            .font
            .as_ref()
            .unwrap_or_else(|| Font::default_font());
        let l = f.layout_text(
            &self.content,
            self.style.font_size,
            c.max_width,
            self.style.wrap,
        );
        self.size = c.constrain(l.size);
        self.layout = Some(l);
        self.size
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let f = self
            .style
            .font
            .as_ref()
            .unwrap_or_else(|| Font::default_font());
        if let Some(l) = &self.layout {
            canvas.save();
            canvas.clip_rect(crate::Rect::from_ltwh(
                offset.dx,
                offset.dy,
                self.size.width,
                self.size.height,
            ));
            for (i, line) in l.lines.iter().enumerate() {
                canvas.draw_text(
                    line,
                    f,
                    self.style.font_size,
                    Point::new(offset.dx, offset.dy + i as f32 * l.line_height),
                    self.style.color,
                );
            }
            canvas.restore();
        }
    }
}
