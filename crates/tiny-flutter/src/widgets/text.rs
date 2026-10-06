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
            layout_input: None,
        })
    }
}
pub struct RenderText {
    pub content: String,
    pub style: TextStyle,
    size: Size,
    offset: Offset,
    layout: Option<TextLayout>,
    layout_input: Option<LayoutInput>,
}
struct LayoutInput {
    content: String,
    // Keeping the previous Font alive prevents pointer reuse from making a
    // newly assigned font look like the source of the cached layout.
    font: Font,
    font_size: f32,
    max_width: f32,
    wrap: bool,
    provider_revision: usize,
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
        let provider_revision = f.layout_revision();
        if let (Some(input), Some(layout)) = (&self.layout_input, &self.layout) {
            if input.content == self.content
                && input.font.same_layout_source(f)
                && input.font_size == self.style.font_size
                && input.max_width == c.max_width
                && input.wrap == self.style.wrap
                && input.provider_revision == provider_revision
            {
                // Minimum constraints and height can still change without
                // changing line breaking. Always constrain the measured size.
                self.size = c.constrain(layout.size);
                return self.size;
            }
        }
        let l = f.layout_text(
            &self.content,
            self.style.font_size,
            c.max_width,
            self.style.wrap,
        );
        self.size = c.constrain(l.size);
        self.layout = Some(l);
        self.layout_input = Some(LayoutInput {
            content: self.content.clone(),
            font: f.clone(),
            font_size: self.style.font_size,
            max_width: c.max_width,
            wrap: self.style.wrap,
            provider_revision,
        });
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

#[cfg(test)]
mod layout_cache_tests {
    use super::*;
    use crate::graphics::font::{install_fontpack, FONT_PACK_TEST_LOCK};
    use crate::graphics::fontpack::{encode_fontpack, FontPack, PackGlyph};
    use std::sync::Arc;

    fn text(content: &str) -> RenderText {
        RenderText {
            content: content.into(),
            style: TextStyle::default(),
            size: Size::ZERO,
            offset: Offset::ZERO,
            layout: None,
            layout_input: None,
        }
    }
    fn assert_matches_fresh_measurement(root: &mut RenderText, c: &BoxConstraints) {
        let actual_size = root.layout(c);
        let font = root
            .style
            .font
            .as_ref()
            .unwrap_or_else(|| Font::default_font());
        let expected = font.layout_text(
            &root.content,
            root.style.font_size,
            c.max_width,
            root.style.wrap,
        );
        assert_eq!(actual_size, c.constrain(expected.size));
        let actual = root.layout.as_ref().unwrap();
        assert_eq!(actual.lines, expected.lines);
        assert_eq!(actual.size, expected.size);
        assert_eq!(actual.line_height, expected.line_height);

        // Paint the cached layout and the original, freshly measured lines onto
        // equal patterned backgrounds. Include own-size clipping and offsets.
        let mut actual_pixels = crate::tiny_gfx::Pixmap565::new(240, 140).unwrap();
        actual_pixels.fill(0x3186);
        let mut expected_pixels = actual_pixels.clone();
        let offset = Offset::new(11.25, 6.5);
        let clip = crate::Rect::from_ltwh(21.0, 3.0, 160.0, 110.0);
        {
            let mut canvas = Canvas::new(actual_pixels.as_mut());
            canvas.clip_rect(clip);
            root.paint(&mut canvas, offset);
        }
        {
            let mut canvas = Canvas::new(expected_pixels.as_mut());
            canvas.clip_rect(clip);
            canvas.clip_rect(crate::Rect::from_ltwh(
                offset.dx,
                offset.dy,
                root.size.width,
                root.size.height,
            ));
            for (i, line) in expected.lines.iter().enumerate() {
                canvas.draw_text(
                    line,
                    font,
                    root.style.font_size,
                    Point::new(offset.dx, offset.dy + i as f32 * expected.line_height),
                    root.style.color,
                );
            }
        }
        assert_eq!(actual_pixels, expected_pixels);
    }
    #[test]
    fn identical_inputs_reuse_lines_but_new_constraints_still_constrain_size() {
        let mut root = text("中文 labels\r\n\nlast line\n");
        root.style.wrap = true;
        let first = BoxConstraints::loose(Size::new(100.0, 110.0));
        assert_matches_fresh_measurement(&mut root, &first);
        let lines = root.layout.as_ref().unwrap().lines.as_ptr();
        let input = root.layout_input.as_ref().unwrap().content.as_ptr();
        for constraints in [
            first,
            BoxConstraints::new(90.0, 100.0, 30.0, 40.0),
            BoxConstraints::new(0.0, 100.0, 120.0, 130.0),
        ] {
            root.style.color = Color::from_rgba(60, 180, 220, 137);
            root.set_offset(Offset::new(17.0, 9.0));
            assert_matches_fresh_measurement(&mut root, &constraints);
            assert_eq!(root.layout.as_ref().unwrap().lines.as_ptr(), lines);
            assert_eq!(root.layout_input.as_ref().unwrap().content.as_ptr(), input);
        }
    }
    #[test]
    fn public_content_style_and_font_changes_invalidate_cached_line_breaking() {
        let mut root = text("A long label for repeated wrapping");
        root.style.wrap = true;
        let wide = BoxConstraints::loose(Size::new(170.0, 130.0));
        let narrow = BoxConstraints::loose(Size::new(65.0, 130.0));
        assert_matches_fresh_measurement(&mut root, &wide);
        assert_matches_fresh_measurement(&mut root, &narrow);
        root.content = "中文漢\nchanged content".into();
        assert_matches_fresh_measurement(&mut root, &narrow);
        root.style.font_size = 22.0;
        assert_matches_fresh_measurement(&mut root, &narrow);
        root.style.wrap = false;
        assert_matches_fresh_measurement(&mut root, &narrow);
        root.style.font = Some(
            Font::from_bytes(include_bytes!(
                "../../../../assets/fonts/source/HarmonyOS_Sans_Regular.ttf"
            ))
            .unwrap(),
        );
        assert_matches_fresh_measurement(&mut root, &narrow);
        root.style.font = None;
        assert_matches_fresh_measurement(&mut root, &narrow);
    }
    fn install_glyph(advance: f32) {
        let bytes = encode_fontpack(vec![PackGlyph {
            character: '钟',
            size: 22,
            width: 2,
            height: 3,
            xmin: 0,
            ymin: 0,
            advance,
            bitmap: vec![180; 6],
        }])
        .unwrap();
        install_fontpack(Some(Arc::new(FontPack::from_bytes(bytes).unwrap())));
    }
    #[test]
    fn synced_fontpack_revisions_remeasure_content_without_invalidating_system_labels() {
        let _guard = FONT_PACK_TEST_LOCK.lock().unwrap();
        let mut content = text("钟钟钟");
        content.style.font_size = 22.0;
        content.style.wrap = true;
        content.style.font = Some(Font::content_font().clone());
        let mut system = text("时钟");
        let constraints = BoxConstraints::loose(Size::new(100.0, 130.0));
        install_glyph(41.0);
        assert_matches_fresh_measurement(&mut content, &constraints);
        let first_size = content.layout.as_ref().unwrap().size;
        let first_revision = content.layout_input.as_ref().unwrap().provider_revision;
        assert_matches_fresh_measurement(&mut system, &constraints);
        let system_lines = system.layout.as_ref().unwrap().lines.as_ptr();
        install_glyph(17.0);
        assert_matches_fresh_measurement(&mut content, &constraints);
        assert_ne!(content.layout.as_ref().unwrap().size, first_size);
        assert_ne!(
            content.layout_input.as_ref().unwrap().provider_revision,
            first_revision
        );
        assert_matches_fresh_measurement(&mut system, &constraints);
        assert_eq!(system.layout.as_ref().unwrap().lines.as_ptr(), system_lines);
        // These fonts share the same TTF Arc but have different providers.
        content.style.font = Some(Font::default_font().clone());
        assert_matches_fresh_measurement(&mut content, &constraints);
        content.style.font = Some(Font::content_font().clone());
        install_fontpack(None);
        assert_matches_fresh_measurement(&mut content, &constraints);
    }
}
