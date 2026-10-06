//! An in-place headline counter. Only its opaque number strip is damaged.
use super::{grouped, Shared};
use crate::usage::headline::{HeadlineSample, FRAME_INTERVAL_MS};
use std::sync::OnceLock;
use tiny_flutter::graphics::font::Glyph;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

pub(super) const NUMBER_HEIGHT: f32 = 90.;

fn numeric_font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(include_bytes!(
            "../../../../assets/fonts/source/DINish-Heavy.ttf"
        ))
        .expect("bundled licensed DINish")
    })
}

fn number_width(chars: usize, size: f32) -> f32 {
    let commas = chars / 4;
    let digits = chars.saturating_sub(commas);
    numeric_font().metrics('0', size).advance_width * digits as f32
        + numeric_font().metrics(',', size).advance_width * commas as f32
}

fn approximate(value: u64) -> String {
    for (scale, suffix) in [
        (1_000_000_000_000_000_000u64, "E"),
        (1_000_000_000_000_000, "P"),
        (1_000_000_000_000, "T"),
        (1_000_000_000, "B"),
        (1_000_000, "M"),
        (1_000, "K"),
    ] {
        if value >= scale {
            return format!("≈ {:.1}{suffix}", value as f64 / scale as f64);
        }
    }
    format!("≈ {value}")
}

/// Warm the eleven numeric glyphs once for a layout. Animation painting uses
/// their masks directly, without reparsing text or locking the font per digit.
struct NumberLayout {
    chars: usize,
    target: Option<u64>,
    width: f32,
    glyphs: Vec<Glyph>,
    slot: f32,
    comma: f32,
    full_width: f32,
    abbreviation: String,
    baseline: f32,
}
impl NumberLayout {
    fn new(sample: HeadlineSample, width: f32) -> Self {
        // Desktop shows the true target's approximation, not the changing
        // interpolated value. Its baseline and reservation stay steady too.
        let abbreviation = sample.target.map(approximate).unwrap_or_default();
        let approximate_width = Font::default_font().measure_text(&abbreviation, 14.0).width;
        let available = (width - approximate_width - 14.0).max(1.0);
        let chars = sample.width_chars.max(1);
        let size = [
            64.0, 60.0, 56.0, 52.0, 48.0, 44.0, 40.0, 36.0, 32.0, 28.0, 24.0, 20.0,
        ]
        .into_iter()
        .find(|size| number_width(chars, *size) <= available)
        .unwrap_or(20.0);
        let glyphs = "0123456789,—"
            .chars()
            .map(|ch| numeric_font().glyph(ch, size))
            .collect::<Vec<_>>();
        let slot = glyphs[0].advance;
        let comma = glyphs[10].advance;
        // Centre the actual numeral outlines, not the font's em box. This
        // remains centred when a very large value requires a smaller size.
        let top = glyphs[..10]
            .iter()
            .map(|g| g.ymin + g.height as i32)
            .max()
            .unwrap_or(1);
        let bottom = glyphs[..10].iter().map(|g| g.ymin).min().unwrap_or(0);
        let baseline = NUMBER_HEIGHT * 0.5 + (top + bottom) as f32 * 0.5;
        Self {
            chars,
            target: sample.target,
            width,
            glyphs,
            slot,
            comma,
            full_width: number_width(chars, size),
            abbreviation,
            baseline,
        }
    }
    fn matches(&self, sample: HeadlineSample, width: f32) -> bool {
        self.chars == sample.width_chars.max(1)
            && self.target == sample.target
            && self.width == width
    }
    fn advance(&self, ch: u8) -> f32 {
        if ch == b',' {
            self.comma
        } else {
            self.slot
        }
    }
    fn paint_number(&self, canvas: &mut Canvas, label: &str, origin: Offset) {
        let mut x = origin.dx;
        let baseline = origin.dy + self.baseline;
        for ch in label.chars() {
            let index = match ch {
                '0'..='9' => (ch as u8 - b'0') as usize,
                ',' => 10,
                _ => 11,
            };
            let glyph = &self.glyphs[index];
            let advance = if ch.is_ascii_digit() {
                self.slot
            } else {
                glyph.advance
            };
            canvas.blit_mask(
                (x + (advance - glyph.advance) * 0.5 + glyph.xmin as f32).round() as i32,
                (baseline - glyph.ymin as f32 - glyph.height as f32).round() as i32,
                glyph.width as u32,
                glyph.height as u32,
                glyph.bitmap(),
                Folio::ink(),
            );
            x += advance;
        }
    }
    fn dirty(&self, previous: HeadlineSample, next: HeadlineSample, height: f32) -> Option<Rect> {
        if previous.value == next.value && self.matches(next, self.width) {
            return None;
        }
        if !self.matches(next, self.width) || previous.value.is_none() || next.value.is_none() {
            return Some(Rect::from_ltwh(0., 0., self.width, height));
        }
        let before = grouped(previous.value.unwrap());
        let after = grouped(next.value.unwrap());
        if before.len() != after.len() {
            return Some(Rect::from_ltwh(0., 0., self.width, height));
        }
        let first = before
            .bytes()
            .zip(after.bytes())
            .position(|(a, b)| a != b)?;
        let left = before
            .bytes()
            .take(first)
            .map(|ch| self.advance(ch))
            .sum::<f32>();
        let right = before.bytes().map(|ch| self.advance(ch)).sum::<f32>();
        // Two pixels cover antialiased glyph bearings. Unchanged leading
        // digits and the fixed approximation need no framebuffer upload.
        // Integer-aligned clipping clears a complete pixel before blending
        // its glyph mask. A fractional clip edge would accumulate coverage
        // when an unchanged leading digit touches the damage boundary.
        let left = (left - 2.).floor().max(0.);
        Some(Rect::from_ltwh(
            left,
            0.,
            (right + 2.).ceil().min(self.width) - left,
            height,
        ))
    }
}

pub(super) struct HeadlineView {
    state: Shared,
    size: Size,
}
impl HeadlineView {
    pub(super) fn new(state: Shared, width: f32, height: f32) -> Self {
        Self {
            state,
            size: Size::new(width, height),
        }
    }
}
impl Widget for HeadlineView {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let revision = self.state.lock().unwrap().usage.revision;
        Box::new(HeadlineBox {
            state: self.state.clone(),
            size: self.size,
            offset: Offset::ZERO,
            sample: HeadlineSample::default(),
            scope_revision: revision,
            last_layout_ms: 0,
            number: None,
            label: String::new(),
        })
    }
}
struct HeadlineBox {
    state: Shared,
    size: Size,
    offset: Offset,
    sample: HeadlineSample,
    scope_revision: u64,
    last_layout_ms: u64,
    number: Option<NumberLayout>,
    label: String,
}
impl RenderBox for HeadlineBox {
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
        let mut state = self.state.lock().unwrap();
        state.sync_usage_headline();
        self.last_layout_ms = state.monotonic_ms;
        self.sample = state.usage.headline.sample(state.monotonic_ms);
        drop(state);
        self.label = self.sample.value.map(grouped).unwrap_or_else(|| "—".into());
        if !self
            .number
            .as_ref()
            .is_some_and(|n| n.matches(self.sample, self.size.width))
        {
            self.number = Some(NumberLayout::new(self.sample, self.size.width));
        }
        self.size
    }
    fn animation_dirty(&self) -> Option<Rect> {
        let mut state = self.state.lock().unwrap();
        state.sync_usage_headline();
        if !state.usage_headline_visible() || state.usage.revision != self.scope_revision {
            return None;
        }
        let now = state.monotonic_ms;
        let next = state.usage.headline.sample(now);
        if next.active && now.saturating_sub(self.last_layout_ms) < FRAME_INTERVAL_MS {
            return None;
        }
        self.number
            .as_ref()?
            .dirty(self.sample, next, self.size.height)
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let bounds = Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height);
        if !canvas.is_rect_visible(bounds) {
            return;
        }
        canvas.save();
        canvas.clip_rect(bounds);
        canvas.draw_rect(bounds, Folio::surface());
        let Some(number) = &self.number else {
            canvas.restore();
            return;
        };
        let small_size = 14.0;
        number.paint_number(canvas, &self.label, offset);
        canvas.draw_text(
            &number.abbreviation,
            Font::default_font(),
            small_size,
            Point::new(
                offset.dx + number.full_width + 14.0,
                offset.dy + number.baseline - 5.0 - Font::default_font().cap_height(small_size),
            ),
            Folio::muted(),
        );
        canvas.restore();
    }
    fn paint_opaque_region(&self, canvas: &mut Canvas, offset: Offset, dirty: Rect) -> bool {
        let bounds = Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height);
        if dirty.x < bounds.x
            || dirty.y < bounds.y
            || dirty.right() > bounds.right()
            || dirty.bottom() > bounds.bottom()
        {
            return false;
        }
        self.paint(canvas, offset);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::headline::HeadlineState;
    #[test]
    fn approximation_matches_desktop_one_decimal_and_preserves_small_values() {
        assert_eq!(approximate(270_879_573), "≈ 270.9M");
        assert_eq!(approximate(9), "≈ 9");
        assert_eq!(approximate(u64::MAX), "≈ 18.4E");
    }
    #[test]
    fn grouped_width_uses_fixed_digit_slots_and_natural_separator_spacing() {
        let font = numeric_font();
        assert_eq!(
            number_width(11, 52.0),
            font.glyph('0', 52.0).advance * 9.0 + font.glyph(',', 52.0).advance * 2.0
        );
        assert_eq!(
            number_width(26, 20.0),
            font.glyph('0', 20.0).advance * 20.0 + font.glyph(',', 20.0).advance * 6.0
        );
    }
    #[test]
    fn target_approximation_and_layout_are_stable_during_animation() {
        let mut state = HeadlineState::default();
        state.snap(Some(270_879_573));
        state.update(Some(280_000_000), 100, 1000);
        let layout = NumberLayout::new(state.sample(100), 532.);
        assert_eq!(layout.abbreviation, "≈ 280.0M");
        for ms in [100, 133, 400, 900, 1100] {
            assert!(layout.matches(state.sample(ms), 532.));
        }
        assert!(
            layout.full_width
                + 14.
                + Font::default_font()
                    .measure_text(&layout.abbreviation, 14.)
                    .width
                <= 532.
        );
        state.snap(Some(u64::MAX));
        let largest = NumberLayout::new(state.sample(1100), 532.);
        assert!(
            largest.full_width
                + 14.
                + Font::default_font()
                    .measure_text(&largest.abbreviation, 14.)
                    .width
                <= 532.
        );
    }
    #[test]
    fn small_update_damages_only_changed_suffix_and_width_change_clears_whole_strip() {
        let mut state = HeadlineState::default();
        state.snap(Some(270_879_573));
        state.update(Some(270_879_673), 100, 1000);
        let previous = state.sample(100);
        let layout = NumberLayout::new(previous, 532.);
        let dirty = layout.dirty(previous, state.sample(1100), 90.).unwrap();
        assert!(
            dirty.x > layout.full_width * 0.6 && dirty.width <= layout.slot * 3. + 6.,
            "suffix damage: {dirty:?}"
        );
        state.snap(Some(999));
        let previous = state.sample(1200);
        let layout = NumberLayout::new(previous, 532.);
        state.update(Some(1000), 1200, 1000);
        assert_eq!(
            layout.dirty(previous, state.sample(1233), 90.),
            Some(Rect::from_ltwh(0., 0., 532., 90.))
        );
    }
}
