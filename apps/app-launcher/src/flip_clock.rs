//! Three mechanical flip cards, drawn by the original tiny-flutter renderer.
//! Wall time selects the face; monotonic time drives a bounded, local animation.

use crate::launcher_state::LauncherState;
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

pub const CLOCK_CONTENT_ORIGIN: Offset = Offset::new(24.0, 78.0);
// Leave enough visible travel on the LCD before the next wall second. The
// final 100 ms lifts only the moving lower leaf; the card and hinge stay fixed.
pub const FLIP_DURATION_MS: u64 = 640;
const FALL_MS: f32 = 280.0;
const LAND_MS: f32 = 260.0;
const FRAME_INTERVAL_MS: u64 = 20;
const DAMAGE_MARGIN: f32 = 16.0;
const DIGIT_WIDTH: usize = 144;
const DIGIT_HEIGHT: usize = 208;
const DASH: u8 = 10;
const BLANK: u8 = 11;
const DIGITS: &[u8; 11 * DIGIT_WIDTH * DIGIT_HEIGHT] =
    include_bytes!("../../../assets/clock/flip-digits.alpha");
// Lift the plates away from the black background, including when the moving
// leaf is shaded. These grays quantize to matching RGB565 channel intensities.
const TOP: Color = Color::from_hex(0x424242);
const BOTTOM: Color = Color::from_hex(0x313131);
const CARD_BASE: Color = Color::from_hex(0x181818);
const HINGE_HIGHLIGHT: Color = Color::from_hex(0x525252);

#[derive(Clone, Copy)]
pub enum ClockControl {
    Home,
    Close,
}

/// Original outline icons, with geometry independent of text/font resources.
pub struct ClockControlPainter {
    path: tiny_gfx::Path,
}

impl ClockControlPainter {
    pub fn new(control: ClockControl) -> Self {
        let mut path = tiny_gfx::PathBuilder::new();
        match control {
            ClockControl::Home => {
                path.move_to(3.0, 14.0);
                path.line_to(16.0, 3.0);
                path.line_to(29.0, 14.0);
                path.move_to(7.0, 12.0);
                for (x, y) in [
                    (7.0, 28.0),
                    (12.0, 28.0),
                    (12.0, 20.0),
                    (20.0, 20.0),
                    (20.0, 28.0),
                    (25.0, 28.0),
                    (25.0, 12.0),
                ] {
                    path.line_to(x, y);
                }
            }
            ClockControl::Close => {
                path.move_to(7.0, 7.0);
                path.line_to(25.0, 25.0);
                path.move_to(25.0, 7.0);
                path.line_to(7.0, 25.0);
            }
        }
        Self {
            path: path.finish().expect("nonempty clock control"),
        }
    }
}

impl CustomPainter for ClockControlPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        canvas.save();
        canvas.translate((size.width - 32.0) * 0.5, (size.height - 32.0) * 0.5);
        canvas.stroke_path(
            &self.path,
            &tiny_gfx::Paint::new(Color::WHITE.to_gfx()),
            &tiny_gfx::Stroke {
                width: 2.4,
                line_cap: tiny_gfx::LineCap::Round,
                line_join: tiny_gfx::LineJoin::Round,
                ..Default::default()
            },
        );
        canvas.restore();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockFace {
    pub cards: [[u8; 2]; 3],
    pub period: Option<&'static str>,
}

impl Default for ClockFace {
    fn default() -> Self {
        Self {
            cards: [[DASH; 2]; 3],
            period: None,
        }
    }
}

impl ClockFace {
    fn from_clock(clock: &str) -> Self {
        let b = clock.as_bytes();
        if b.len() != 8
            || b[2] != b':'
            || b[5] != b':'
            || [0, 1, 3, 4, 6, 7].iter().any(|&i| !b[i].is_ascii_digit())
        {
            return Self::default();
        }
        let hour = (b[0] - b'0') * 10 + b[1] - b'0';
        let minute = (b[3] - b'0') * 10 + b[4] - b'0';
        let second = (b[6] - b'0') * 10 + b[7] - b'0';
        if hour > 23 || minute > 59 || second > 59 {
            return Self::default();
        }
        let twelve = if hour % 12 == 0 { 12 } else { hour % 12 };
        Self {
            cards: [
                [if twelve < 10 { BLANK } else { twelve / 10 }, twelve % 10],
                [minute / 10, minute % 10],
                [second / 10, second % 10],
            ],
            period: Some(if hour < 12 { "AM" } else { "PM" }),
        }
    }
}

#[derive(Clone, Copy)]
pub struct FlipSample {
    pub from: ClockFace,
    pub to: ClockFace,
    pub progress: f32,
}

#[derive(Default)]
pub struct FlipClockState {
    from: ClockFace,
    to: ClockFace,
    started_ms: Option<u64>,
    last_frame: u64,
}

impl FlipClockState {
    pub fn snap(&mut self, clock: &str) {
        self.to = ClockFace::from_clock(clock);
        self.from = self.to;
        self.started_ms = None;
        self.last_frame = 0;
    }

    pub fn update(&mut self, clock: &str, now_ms: u64, animate: bool) {
        let next = ClockFace::from_clock(clock);
        if next == self.to {
            return;
        }
        if animate && self.to.period.is_some() && next.period.is_some() {
            self.from = self.to;
            self.to = next;
            self.started_ms = Some(now_ms);
            self.last_frame = 0;
        } else {
            self.snap(clock);
        }
    }

    pub fn sample(&self, now_ms: u64) -> FlipSample {
        let progress = self.started_ms.map_or(1.0, |start| {
            now_ms.saturating_sub(start).min(FLIP_DURATION_MS) as f32 / FLIP_DURATION_MS as f32
        });
        FlipSample {
            from: self.from,
            to: self.to,
            progress,
        }
    }

    /// Consume at most one local repaint per gate. The settled frame is always
    /// returned, even if the loop was blocked beyond the complete animation.
    pub fn take_dirty(&mut self, now_ms: u64, content_size: Size) -> Option<Rect> {
        let start = self.started_ms?;
        let elapsed = now_ms.saturating_sub(start);
        if elapsed >= FLIP_DURATION_MS {
            self.started_ms = None;
        } else {
            let frame = elapsed / FRAME_INTERVAL_MS;
            if frame == self.last_frame {
                return None;
            }
            self.last_frame = frame;
        }
        let layout = ClockLayout::new(content_size);
        let mut dirty: Option<Rect> = None;
        for i in 0..3 {
            if self.from.cards[i] != self.to.cards[i]
                || (i == 0 && self.from.period != self.to.period)
            {
                let card = layout.cards[i].inflate(DAMAGE_MARGIN * layout.scale);
                dirty = Some(dirty.map_or(card, |r| r.union(&card)));
            }
        }
        dirty.map(|r| r.shift(CLOCK_CONTENT_ORIGIN))
    }
}

struct ClockLayout {
    cards: [Rect; 3],
    scale: f32,
}

impl ClockLayout {
    fn new(size: Size) -> Self {
        let scale = (size.width / 976.0)
            .min(size.height / 506.0)
            .clamp(0.1, 1.0);
        let gap = 24.0 * scale;
        let width = ((size.width - 2.0 * gap) / 3.0).min(309.0).floor().max(1.0);
        let height = width.min((size.height - 184.0 * scale).max(1.0));
        let left = ((size.width - width * 3.0 - gap * 2.0) / 2.0).round();
        let top = ((size.height - height - 64.0 * scale) / 2.0)
            .round()
            .max(0.0);
        Self {
            cards: std::array::from_fn(|i| {
                Rect::from_ltwh(left + i as f32 * (width + gap), top, width, height)
            }),
            scale: width / 309.0,
        }
    }
}

pub struct FlipClockPainter {
    state: Arc<Mutex<LauncherState>>,
}

impl FlipClockPainter {
    pub fn new(state: Arc<Mutex<LauncherState>>) -> Self {
        Self { state }
    }
}

impl CustomPainter for FlipClockPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let sample = {
            let state = self.state.lock().unwrap();
            state.flip_clock.sample(state.monotonic_ms)
        };
        let layout = ClockLayout::new(size);
        canvas.draw_rect(
            Rect::from_ltwh(0.0, 0.0, size.width, size.height),
            Color::BLACK,
        );
        for (i, card) in layout.cards.iter().copied().enumerate() {
            if !visible(canvas, card.inflate(DAMAGE_MARGIN * layout.scale)) {
                continue;
            }
            canvas.draw_rrect(
                RRect::from_rect_circular(
                    card.shift(Offset::new(0.0, 6.0 * layout.scale)),
                    20.0 * layout.scale,
                ),
                CARD_BASE,
            );
            let turning = sample.progress < 1.0 && sample.from.cards[i] != sample.to.cards[i];
            paint_half(canvas, card, sample.to.cards[i], true);
            paint_half(
                canvas,
                card,
                if turning {
                    sample.from.cards[i]
                } else {
                    sample.to.cards[i]
                },
                false,
            );
            if turning {
                let leaf = FlipLeaf::at(sample.progress);
                paint_cast_shadow(canvas, card, leaf);
                paint_leaf(
                    canvas,
                    card,
                    if leaf.upper {
                        sample.from.cards[i]
                    } else {
                        sample.to.cards[i]
                    },
                    leaf,
                );
            }
            let hinge = (card.y + card.height * 0.5).round();
            canvas.draw_rect(
                Rect::from_ltwh(card.x, hinge - 2.0, card.width, 4.0),
                Color::BLACK,
            );
            canvas.draw_rect(
                Rect::from_ltwh(card.x + 2.0, hinge + 2.0, card.width - 4.0, 1.0),
                HINGE_HIGHLIGHT,
            );
            if i == 0 {
                if let Some(period) = sample.to.period {
                    canvas.draw_text(
                        period,
                        Font::default_font(),
                        28.0 * layout.scale,
                        Point::new(
                            card.x + 18.0 * layout.scale,
                            card.bottom() - 57.0 * layout.scale,
                        ),
                        Color::WHITE,
                    );
                }
            }
        }
    }
}

// tiny_gfx's current clip is in screen coordinates. Empty intersections must be
// rejected before clip_rect: that API currently represents an empty clip as None.
fn visible(canvas: &Canvas, rect: Rect) -> bool {
    canvas
        .current_clip()
        .is_none_or(|clip| clip.intersect(&rect.shift(CLOCK_CONTENT_ORIGIN)).is_some())
}

fn paint_half(canvas: &mut Canvas, card: Rect, digits: [u8; 2], upper: bool) {
    let hinge = card.y + card.height * 0.5;
    let height = card.height * 0.5;
    let half = if upper {
        Rect::from_ltwh(card.x, hinge - height, card.width, height)
    } else {
        Rect::from_ltwh(card.x, hinge, card.width, height)
    };
    if !visible(canvas, half) {
        return;
    }
    canvas.save();
    canvas.clip_rect(half);
    canvas.draw_rrect(
        RRect::from_rect_circular(
            Rect::from_ltwh(card.x, hinge - height, card.width, height * 2.0),
            (20.0 * card.width / 309.0).min(height),
        ),
        if upper { TOP } else { BOTTOM },
    );
    paint_digits(
        canvas,
        card,
        digits,
        if upper {
            Color::WHITE
        } else {
            Color::from_hex(0xf4f4f4)
        },
    );
    canvas.restore();
}

fn paint_digits(canvas: &mut Canvas, card: Rect, digits: [u8; 2], color: Color) {
    let scale = card.width / 309.0;
    let width = (DIGIT_WIDTH as f32 * scale)
        .round()
        .clamp(1.0, DIGIT_WIDTH as f32) as usize;
    let height = (DIGIT_HEIGHT as f32 * scale).round().max(1.0) as usize;
    let count = if digits[0] == BLANK { 1 } else { 2 };
    let left = (card.x + (card.width - width as f32 * count as f32) * 0.5).round() as i32;
    let hinge = card.y + card.height * 0.5;
    // Both upright and projected digits rotate around the plate's actual
    // midline. The atlas ink is centered vertically in its 208-pixel cell.
    let top = (hinge - DIGIT_HEIGHT as f32 * scale * 0.5).round() as i32;
    for slot in 0..count {
        let digit = digits[if count == 1 { 1 } else { slot }];
        if digit > DASH {
            continue;
        }
        let offset = digit as usize * DIGIT_WIDTH * DIGIT_HEIGHT;
        let mask = &DIGITS[offset..offset + DIGIT_WIDTH * DIGIT_HEIGHT];
        let x = left + (slot * width) as i32;
        if width == DIGIT_WIDTH && height == DIGIT_HEIGHT {
            canvas.blit_mask(x, top, DIGIT_WIDTH as u32, DIGIT_HEIGHT as u32, mask, color);
        } else {
            // Pure row scaling, with no icon corner mask, per-frame atlas or
            // full-screen scratch allocation. Upright digits use the native mask.
            let mut row_mask = [0u8; DIGIT_WIDTH];
            for row in 0..height {
                let source = (row * DIGIT_HEIGHT / height).min(DIGIT_HEIGHT - 1) * DIGIT_WIDTH;
                let row_data = if width == DIGIT_WIDTH {
                    &mask[source..source + DIGIT_WIDTH]
                } else {
                    for (col, alpha) in row_mask[..width].iter_mut().enumerate() {
                        *alpha = mask[source + col * DIGIT_WIDTH / width];
                    }
                    &row_mask[..width]
                };
                canvas.blit_mask(x, top + row as i32, width as u32, 1, row_data, color);
            }
        }
    }
}

/// Two faces of one rigid leaf, projected about the fixed horizontal hinge.
/// See docs/flip-clock-animation.md for the geometry and reference projects.
#[derive(Clone, Copy)]
struct FlipLeaf {
    upper: bool,
    cosine: f32,
    sine: f32,
}

impl FlipLeaf {
    fn at(progress: f32) -> Self {
        use std::f32::consts::{FRAC_PI_2, PI};
        let ms = progress.clamp(0.0, 1.0) * FLIP_DURATION_MS as f32;
        let angle = if ms < FALL_MS {
            let t = ms / FALL_MS;
            FRAC_PI_2 * t * t // gravity accelerates the departing upper face
        } else if ms < FALL_MS + LAND_MS {
            let t = (ms - FALL_MS) / LAND_MS;
            FRAC_PI_2 + FRAC_PI_2 * (1.0 - (1.0 - t).powi(2))
        } else {
            let t = (ms - FALL_MS - LAND_MS) / (FLIP_DURATION_MS as f32 - FALL_MS - LAND_MS);
            // One small, damped lift after contact, never a shake of the whole
            // numeral or a swap back to the old front face.
            PI - 12.0f32.to_radians() * (PI * t).sin() * (1.0 - t)
        };
        Self {
            upper: ms < FALL_MS,
            cosine: angle.cos().abs(),
            sine: angle.sin().max(0.0),
        }
    }

    fn depth(self, card: Rect) -> f32 {
        // This distance limits widening to < 12 px per side at native size;
        // even simultaneous flips cannot overlap the adjacent card.
        card.height * 7.0
    }

    fn extent(self, card: Rect) -> f32 {
        let half = card.height * 0.5;
        half * self.cosine / (1.0 - half * self.sine / self.depth(card))
    }
}

fn paint_cast_shadow(canvas: &mut Canvas, card: Rect, leaf: FlipLeaf) {
    let hinge = card.y + card.height * 0.5;
    let scale = card.width / 309.0;
    let edge = if leaf.upper {
        hinge - leaf.extent(card)
    } else {
        hinge + leaf.extent(card)
    };
    let blur = (6.0 + 30.0 * leaf.sine) * scale;
    // The moving free edge casts a soft shadow on the exposed stationary leaf.
    // A second hinge shadow shows the old bottom behind the departing top.
    for (upper, edge, strength) in if leaf.upper {
        [(true, edge, 0.48), (false, hinge, 0.32)]
    } else {
        [(false, edge, 0.48), (false, hinge, 0.0)]
    } {
        let half = Rect::from_ltwh(
            card.x,
            if upper { card.y } else { hinge },
            card.width,
            card.height * 0.5,
        );
        if !visible(canvas, half) || strength == 0.0 {
            continue;
        }
        canvas.save();
        canvas.clip_rect(half);
        for row in 0..blur.ceil() as i32 {
            let fade = 1.0 - row as f32 / blur;
            canvas.draw_rect(
                Rect::from_ltwh(
                    card.x,
                    if upper {
                        edge - row as f32 - 1.0
                    } else {
                        edge + row as f32
                    },
                    card.width,
                    1.0,
                ),
                Color::BLACK.with_opacity(strength * leaf.sine * fade * fade),
            );
        }
        canvas.restore();
    }
}

// A row is enough for the projected page. No per-frame texture, matrix/image
// allocation, icon mask, or full-screen scratch buffer is needed on the P4.
const LEAF_ROW_CAPACITY: usize = 352;

fn paint_leaf(canvas: &mut Canvas, card: Rect, digits: [u8; 2], leaf: FlipLeaf) {
    // At rest use the exact native glyph blit, including its alpha blending.
    // This makes release/contact pixel-identical to the accepted clock face.
    if leaf.sine < 0.00001 {
        paint_half(canvas, card, digits, leaf.upper);
        return;
    }
    let hinge = card.y + card.height * 0.5;
    let center = card.x + card.width * 0.5;
    let half = card.height * 0.5;
    let extent = leaf.extent(card);
    let scale = card.width / 309.0;
    let max_width = card.width / (1.0 - half * leaf.sine / leaf.depth(card));
    let bounds = Rect::from_ltwh(
        center - max_width * 0.5,
        if leaf.upper { hinge - extent } else { hinge },
        max_width,
        extent.max(2.0),
    );
    if !visible(canvas, bounds) {
        return;
    }
    if extent < 0.8 {
        // At 90 degrees only the physical edge remains visible.
        canvas.draw_rect(
            Rect::from_ltwh(center - max_width * 0.5, hinge - 1.0, max_width, 2.0),
            Color::from_hex(0x363636),
        );
        return;
    }
    let top = if leaf.upper { hinge - extent } else { hinge };
    let bottom = if leaf.upper { hinge } else { hinge + extent };
    let count = if digits[0] == BLANK { 1 } else { 2 };
    let slot_width = (DIGIT_WIDTH as f32 * scale).round().max(1.0);
    let glyph_left = (card.x + (card.width - slot_width * count as f32) * 0.5).round();
    let glyph_top = (hinge - DIGIT_HEIGHT as f32 * scale * 0.5).round();
    let texture_scale = slot_width / DIGIT_WIDTH as f32;
    let radius = 20.0 * scale;
    let shade = 1.0 - if leaf.upper { 0.50 } else { 0.32 } * leaf.sine;
    let background = ((if leaf.upper { TOP.r } else { BOTTOM.r }) as f32 * shade).round() as u16;
    let ink = ((if leaf.upper { 255.0 } else { 244.0 }) * shade).round() as u16;
    let palette: [u16; 256] = std::array::from_fn(|a| {
        let gray = background + ((ink - background) * a as u16 + 127) / 255;
        // Match 5- and 6-bit channel intensities so changing shadows stay gray.
        let r = gray >> 3;
        (r << 11) | (((r << 1) | (r >> 4)) << 5) | r
    });
    let mut pixels = [0u16; LEAF_ROW_CAPACITY];
    for y in top.floor() as i32..bottom.ceil() as i32 {
        let distance = ((y as f32 + 0.5) - hinge).abs();
        // Invert perspective once per scanline: v=s*cos/(1-s*sin/d).
        let source_distance = distance / (leaf.cosine + distance * leaf.sine / leaf.depth(card));
        if source_distance > half {
            continue;
        }
        let perspective = 1.0 / (1.0 - source_distance * leaf.sine / leaf.depth(card));
        let corner_y = (source_distance - (half - radius)).max(0.0);
        let inset = if corner_y > 0.0 {
            radius - (radius * radius - corner_y * corner_y).max(0.0).sqrt()
        } else {
            0.0
        };
        let width = (card.width - 2.0 * inset) * perspective;
        let left = center - width * 0.5;
        let right = center + width * 0.5;
        let x1 = left.ceil() as i32;
        let x2 = right.floor() as i32;
        let len = (x2 - x1).max(0) as usize;
        if len > LEAF_ROW_CAPACITY {
            continue;
        }
        let source_y = (hinge
            + if leaf.upper {
                -source_distance
            } else {
                source_distance
            }
            - glyph_top)
            / scale
            - 0.5;
        let source_x =
            (center + (x1 as f32 + 0.5 - center) / perspective - glyph_left) / texture_scale - 0.5;
        // Q16 stepping avoids cumulative subpixel drift across a two-digit
        // row; the bilinear sampler only needs the high Q8 portion.
        let mut u = (source_x * 65536.0).round() as i32;
        let step = (65536.0 / (texture_scale * perspective)).round() as i32;
        let v = (source_y * 256.0).round() as i32;
        for pixel in &mut pixels[..len] {
            *pixel = palette[sample_digit_row(digits, count, u >> 8, v) as usize];
            u += step;
        }
        canvas.blit_image_565(x1, y, len as u32, 1, &pixels[..len]);
        // Fractional trapezoid edges are antialiased separately. The atlas ink
        // has margins here; opaque interior rows avoid an extra alpha pass.
        for (x, coverage) in [(x1 - 1, x1 as f32 - left), (x2, right - x2 as f32)] {
            canvas.blit_mask(
                x,
                y,
                1,
                1,
                &[(coverage * 255.0).clamp(0.0, 255.0) as u8],
                Color::from_rgb(background as u8, background as u8, background as u8),
            );
        }
    }
}

/// Bilinear alpha sampling uses Q8 coordinates. Interpolation stays integer in
/// the inner loop, preventing vibrating nearest-neighbor edges during the turn.
#[inline(always)]
fn sample_digit_row(digits: [u8; 2], count: usize, u: i32, v: i32) -> u8 {
    if u < 0 || v < 0 || u >= (count * DIGIT_WIDTH * 256) as i32 || v >= (DIGIT_HEIGHT * 256) as i32
    {
        return 0;
    }
    let x = (u >> 8) as usize;
    let y = (v >> 8) as usize;
    let alpha = |x: usize, y: usize| -> u32 {
        if x >= count * DIGIT_WIDTH || y >= DIGIT_HEIGHT {
            return 0;
        }
        let digit = digits[if count == 1 { 1 } else { x / DIGIT_WIDTH }];
        if digit > DASH {
            return 0;
        }
        DIGITS[digit as usize * DIGIT_WIDTH * DIGIT_HEIGHT + y * DIGIT_WIDTH + x % DIGIT_WIDTH]
            as u32
    };
    let fx = (u & 255) as u32;
    let fy = (v & 255) as u32;
    let a = alpha(x, y) * (256 - fx) + alpha(x + 1, y) * fx;
    let b = alpha(x, y + 1) * (256 - fx) + alpha(x + 1, y + 1) * fx;
    ((a * (256 - fy) + b * fy + 32768) >> 16) as u8
}
