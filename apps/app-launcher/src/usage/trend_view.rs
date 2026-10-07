//! Exact, bounded trend replay and in-place refresh of the whole chart card.
use super::{chart_snapshot, edit, trend_path, Data, Shared};
use crate::usage::{Page, Period};
use std::sync::Arc;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Scope {
    revision: u64,
    page: Page,
    period: Period,
    detail: Option<i64>,
}
impl Scope {
    fn read(state: &crate::usage::State) -> Self {
        Self {
            revision: state.revision,
            page: state.page,
            period: state.period,
            detail: state.detail,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TrendAxis {
    Samples,
    Day,
}
impl TrendAxis {
    pub(super) fn read(state: &crate::usage::State) -> Self {
        if state.page == Page::Overview && state.period == Period::Day && state.detail.is_none() {
            Self::Day
        } else {
            Self::Samples
        }
    }
    fn fraction(self, point: &crate::usage::Point, index: usize, count: usize) -> Option<f32> {
        match self {
            Self::Day => minute_of_day(&point.date).map(|minute| minute as f32 / 1440.),
            Self::Samples => Some(if count <= 1 {
                0.5
            } else {
                index as f32 / (count - 1) as f32
            }),
        }
    }
}
pub(super) fn minute_of_day(date: &str) -> Option<u16> {
    let time = if date.len() == 5 {
        date
    } else if matches!(date.as_bytes().get(10), Some(b' ') | Some(b'T')) {
        date.get(11..16)?
    } else {
        return None;
    };
    let b = time.as_bytes();
    if b.len() != 5 || b[2] != b':' || ![b[0], b[1], b[3], b[4]].iter().all(u8::is_ascii_digit) {
        return None;
    }
    let hour = (b[0] - b'0') as u16 * 10 + (b[1] - b'0') as u16;
    let minute = (b[3] - b'0') as u16 * 10 + (b[4] - b'0') as u16;
    (hour < 24 && minute < 60).then_some(hour * 60 + minute)
}

fn same_points(a: &[crate::usage::Point], b: &[crate::usage::Point]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            a.date == b.date
                && a.totals.tokens == b.totals.tokens
                && a.totals.cost.to_bits() == b.totals.cost.to_bits()
                && a.totals.requests == b.totals.requests
        })
}
fn same_chart(a: &Data, b: &Data) -> bool {
    a.trend_label == b.trend_label
        && same_points(&a.trend, &b.trend)
        && same_points(&a.yesterday_trend, &b.yesterday_trend)
}

pub(super) struct LiveTrendCard {
    state: Shared,
    data: Arc<Data>,
    size: Size,
}
impl LiveTrendCard {
    pub(super) fn new(state: Shared, data: Arc<Data>, size: Size) -> Self {
        Self { state, data, size }
    }
}
impl Widget for LiveTrendCard {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let state = self.state.lock().unwrap();
        Box::new(LiveTrendBox {
            state: self.state.clone(),
            data: self.data.clone(),
            selected: state.usage.chart_selected,
            scope: Scope::read(&state.usage),
            size: self.size,
            offset: Offset::ZERO,
            appearance: (Folio::is_light(), Folio::glass_amount()),
            child: None,
        })
    }
}
struct LiveTrendBox {
    state: Shared,
    data: Arc<Data>,
    selected: Option<usize>,
    scope: Scope,
    size: Size,
    offset: Offset,
    appearance: (bool, u8),
    child: Option<Box<dyn RenderBox>>,
}
impl RenderBox for LiveTrendBox {
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
        let previous_size = self.size;
        self.size = constraints.constrain(self.size);
        let state = self.state.lock().unwrap();
        let mut changed = self.child.is_none() || previous_size != self.size;
        if Scope::read(&state.usage) == self.scope {
            if let Some(data) = &state.usage.data {
                changed |= (!Arc::ptr_eq(data, &self.data) && !same_chart(data, &self.data))
                    || self.selected != state.usage.chart_selected;
                self.data = data.clone();
                self.selected = state.usage.chart_selected;
            }
        }
        drop(state);
        let appearance = (Folio::is_light(), Folio::glass_amount());
        changed |= self.appearance != appearance;
        self.appearance = appearance;
        if changed {
            // Release the old subtree before allocating points, labels and
            // render objects for the next snapshot. The plot's native cache
            // independently releases its previous version before capture.
            self.child = None;
            self.child = Some(
                chart_snapshot(
                    self.state.clone(),
                    &self.data,
                    self.size.width,
                    self.size.height,
                )
                .create_render_object(),
            );
        }
        if let Some(child) = &mut self.child {
            child.layout(&BoxConstraints::tight(self.size));
        }
        self.size
    }
    fn animation_dirty(&self) -> Option<Rect> {
        let state = self.state.lock().unwrap();
        if !state.usage_headline_visible() || Scope::read(&state.usage) != self.scope {
            return None;
        }
        let changed = state.usage.data.as_ref().is_some_and(|data| {
            (!Arc::ptr_eq(data, &self.data) && !same_chart(data, &self.data))
                || self.selected != state.usage.chart_selected
        });
        changed.then(|| Rect::from_ltwh(0., 0., self.size.width, self.size.height))
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        if let Some(child) = &self.child {
            child.paint(canvas, offset);
        }
    }
    fn paint_opaque_region(&self, canvas: &mut Canvas, offset: Offset, dirty: Rect) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.paint_opaque_region(canvas, offset, dirty))
    }
    fn hit_rect(&self, point: tiny_flutter::Point) -> Option<Rect> {
        self.child.as_ref().and_then(|child| child.hit_rect(point))
    }
    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        self.child
            .as_mut()
            .is_some_and(|child| child.dispatch_touch(event))
    }
    fn set_pressed_at(&mut self, point: tiny_flutter::Point, pressed: bool) {
        if let Some(child) = &mut self.child {
            child.set_pressed_at(point, pressed);
        }
    }
    fn captures_touch(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.captures_touch())
    }
}

#[derive(Clone, Copy)]
struct PlotStyle {
    background: Color,
    grid: Color,
    fill: Color,
    previous: Color,
    current: Color,
    marker: Color,
}
impl PlotStyle {
    fn current() -> Self {
        Self {
            background: Folio::surface(),
            grid: Folio::line().with_opacity(0.35),
            fill: Folio::blue().with_opacity(0.07),
            previous: Folio::muted().with_opacity(0.65),
            current: Folio::blue(),
            marker: Folio::blue().with_opacity(0.22),
        }
    }
    fn words(self) -> [u64; 6] {
        [
            self.background,
            self.grid,
            self.fill,
            self.previous,
            self.current,
            self.marker,
        ]
        .map(|c| u32::from_le_bytes([c.r, c.g, c.b, c.a]) as u64)
    }
}
fn plot_signature(
    state: &Shared,
    points: &[crate::usage::Point],
    previous: &[crate::usage::Point],
    selected: usize,
    style: PlotStyle,
) -> Option<Vec<u64>> {
    let state = state.lock().unwrap();
    let scope = Scope::read(&state.usage);
    let axis = TrendAxis::read(&state.usage);
    drop(state);
    let point_words = if axis == TrendAxis::Day { 2 } else { 1 };
    let len = 15usize.checked_add(
        points
            .len()
            .checked_add(previous.len())?
            .checked_mul(point_words)?,
    )?;
    if len > 512 {
        return None;
    }
    let mut key = Vec::new();
    key.try_reserve_exact(len).ok()?;
    key.extend_from_slice(&[
        scope.revision,
        scope.page as u64,
        scope.period as u64,
        scope.detail.is_some() as u64,
        scope.detail.unwrap_or(0) as u64,
        selected as u64,
        points.len() as u64,
        previous.len() as u64,
        axis as u64,
    ]);
    key.extend_from_slice(&style.words());
    for point in points.iter().chain(previous) {
        key.push(point.totals.tokens);
        if axis == TrendAxis::Day {
            // Moving a bucket with unchanged tokens changes its x coordinate.
            key.push(
                minute_of_day(&point.date)
                    .map(u64::from)
                    .unwrap_or(u64::MAX),
            );
        }
    }
    Some(key)
}

pub(super) struct InteractiveTrend {
    pub(super) state: Shared,
    pub(super) points: Arc<Vec<crate::usage::Point>>,
    pub(super) previous: Arc<Vec<crate::usage::Point>>,
    pub(super) selected: usize,
    pub(super) size: Size,
}
impl Widget for InteractiveTrend {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let style = PlotStyle::current();
        let axis = TrendAxis::read(&self.state.lock().unwrap().usage);
        Box::new(TrendBox {
            state: self.state.clone(),
            points: self.points.clone(),
            previous: self.previous.clone(),
            selected: self.selected,
            size: self.size,
            offset: Offset::ZERO,
            origin: None,
            axis,
            signature: plot_signature(
                &self.state,
                &self.points,
                &self.previous,
                self.selected,
                style,
            ),
            style,
        })
    }
}
struct TrendBox {
    state: Shared,
    points: Arc<Vec<crate::usage::Point>>,
    previous: Arc<Vec<crate::usage::Point>>,
    selected: usize,
    size: Size,
    offset: Offset,
    origin: Option<tiny_flutter::Point>,
    axis: TrendAxis,
    signature: Option<Vec<u64>>,
    style: PlotStyle,
}
impl RenderBox for TrendBox {
    fn size(&self) -> Size {
        self.size
    }
    fn offset(&self) -> Offset {
        self.offset
    }
    fn set_offset(&mut self, o: Offset) {
        self.offset = o;
    }
    fn layout(&mut self, c: &BoxConstraints) -> Size {
        self.size = c.constrain(self.size);
        self.size
    }
    fn paint(&self, c: &mut Canvas, o: Offset) {
        if !c.is_rect_visible(Rect::from_ltwh(
            o.dx,
            o.dy,
            self.size.width,
            self.size.height,
        )) {
            return;
        }
        c.save();
        c.translate(o.dx, o.dy);
        c.clip_rect(Rect::from_ltwh(0., 0., self.size.width, self.size.height));
        if let Some(signature) = &self.signature {
            c.cache_opaque_region(
                Rect::from_ltwh(0., 0., self.size.width, self.size.height),
                0x5452454e,
                signature,
                |c| self.paint_plot(c),
            );
        } else {
            self.paint_plot(c);
        }
        c.restore();
    }
    fn hit_rect(&self, p: tiny_flutter::Point) -> Option<Rect> {
        self.hit_test(p)
            .then(|| Rect::from_ltwh(0., 0., self.size.width, self.size.height))
    }
    fn dispatch_touch(&mut self, e: &TouchEvent) -> bool {
        match e {
            TouchEvent::Down(p) if self.hit_test(*p) => {
                self.origin = Some(*p);
                true
            }
            TouchEvent::Move(p) => {
                if self
                    .origin
                    .is_some_and(|o| (p.x - o.x).abs() > 12. || (p.y - o.y).abs() > 12.)
                {
                    self.origin = None;
                }
                false
            }
            TouchEvent::Up(p) => {
                if self.origin.take().is_some() && self.hit_test(*p) && !self.points.is_empty() {
                    if let Some(index) = self.selection_at(p.x) {
                        edit(&self.state, |u| u.chart_selected = Some(index));
                        return true;
                    }
                }
                false
            }
            TouchEvent::Cancel => {
                self.origin = None;
                false
            }
            _ => false,
        }
    }
}
impl TrendBox {
    fn x(&self, points: &[crate::usage::Point], index: usize) -> Option<f32> {
        self.axis
            .fraction(points.get(index)?, index, self.points.len())
            .map(|f| 2. + f * (self.size.width - 4.))
    }
    fn selection_at(&self, x: f32) -> Option<usize> {
        let mut nearest = None;
        let mut distance = f32::INFINITY;
        let mut latest = f32::NEG_INFINITY;
        for index in 0..self.points.len() {
            if let Some(px) = self.x(&self.points, index) {
                latest = latest.max(px);
                let d = (px - x).abs();
                if d < distance {
                    distance = d;
                    nearest = Some(index);
                }
            }
        }
        // The future has no selectable bucket; allow a small hit margin at
        // the last real marker without manufacturing a future reading.
        if self.axis == TrendAxis::Day && x > latest + 12. {
            None
        } else {
            nearest
        }
    }
    fn paint_plot(&self, c: &mut Canvas) {
        let w = self.size.width;
        let h = self.size.height;
        c.draw_rect(Rect::from_ltwh(0., 0., w, h), self.style.background);
        for i in 0..3 {
            c.draw_rect(
                Rect::from_ltwh(0., 2. + i as f32 * (h - 4.) / 2., w, 1.),
                self.style.grid,
            );
        }
        let max = self
            .points
            .iter()
            .chain(self.previous.iter())
            .map(|v| v.totals.tokens)
            .max()
            .unwrap_or(0)
            .max(1) as f64;
        let n = self.points.len();
        let y = |v: u64| 2. + (1. - v as f64 / max) as f32 * (h - 4.);
        if n > 1 {
            let coords = self
                .points
                .iter()
                .enumerate()
                .filter_map(|(i, v)| self.x(&self.points, i).map(|x| (x, y(v.totals.tokens))))
                .collect::<Vec<_>>();
            if let (Some(first), Some(last)) = (coords.first(), coords.last()) {
                let mut area = trend_path(&coords);
                area.line_to(last.0, h);
                area.line_to(first.0, h);
                area.close();
                if let Some(p) = area.finish() {
                    c.fill_path(
                        &p,
                        &tiny_gfx::Paint::new(self.style.fill.to_gfx()),
                        tiny_gfx::FillRule::Winding,
                    );
                }
            }
        }
        for (points, color, width) in [
            (&*self.previous, self.style.previous, 1.3),
            (&*self.points, self.style.current, 2.0),
        ] {
            let coords = points
                .iter()
                .enumerate()
                .take(if self.axis == TrendAxis::Day {
                    points.len()
                } else {
                    n
                })
                .filter_map(|(i, v)| self.x(points, i).map(|x| (x, y(v.totals.tokens))))
                .collect::<Vec<_>>();
            if let Some(path) = trend_path(&coords).finish() {
                c.stroke_path(
                    &path,
                    &tiny_gfx::Paint::new(color.to_gfx()),
                    &tiny_gfx::Stroke {
                        width,
                        line_cap: tiny_gfx::LineCap::Round,
                        line_join: tiny_gfx::LineJoin::Round,
                        ..Default::default()
                    },
                );
            }
        }
        if let (Some(p), Some(px)) = (
            self.points.get(self.selected),
            self.x(&self.points, self.selected),
        ) {
            let py = y(p.totals.tokens);
            c.draw_rect(Rect::from_ltwh(px, 0., 1., h), self.style.marker);
            c.paint_circle(
                tiny_flutter::Point::new(px, py),
                4.2,
                &tiny_gfx::Paint::new(self.style.background.to_gfx()),
                None,
            );
            c.paint_circle(
                tiny_flutter::Point::new(px, py),
                2.6,
                &tiny_gfx::Paint::new(self.style.current.to_gfx()),
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        usage::{Point, Totals},
        LauncherState,
    };
    use std::sync::Mutex;
    use tiny_flutter::tiny_gfx::Pixmap565;

    fn points(n: usize, seed: u64) -> Vec<Point> {
        (0..n)
            .map(|i| Point {
                date: format!("2026-10-06 {i:02}:00"),
                totals: Totals {
                    tokens: (i as u64 * seed + 1423) % 3_050_000,
                    cost: i as f64 / 13.,
                    requests: Some(i as u64),
                },
            })
            .collect()
    }
    fn state() -> Shared {
        let mut state = LauncherState::new();
        state.open_app("sub2api-monitor");
        let mut data = Data::default();
        data.trend = points(24, 923471);
        data.yesterday_trend = points(24, 723411);
        data.trend_label = "真实趋势".into();
        state.usage.data = Some(Arc::new(data));
        Arc::new(Mutex::new(state))
    }
    fn plot(state: &Shared, n: usize, selected: usize, previous: bool, size: Size) -> TrendBox {
        let current = Arc::new(points(n, 923471));
        let previous = Arc::new(if previous {
            points(n, 723411)
        } else {
            Vec::new()
        });
        let style = PlotStyle::current();
        TrendBox {
            state: state.clone(),
            signature: plot_signature(state, &current, &previous, selected, style),
            style,
            points: current,
            previous,
            selected,
            size,
            offset: Offset::ZERO,
            origin: None,
            axis: TrendAxis::read(&state.lock().unwrap().usage),
        }
    }
    fn paint_plot(
        plot: &TrendBox,
        cached: bool,
        background: Color,
        clip: Option<Rect>,
    ) -> Pixmap565 {
        let mut pixels = Pixmap565::new(600, 180).unwrap();
        let mut c = Canvas::new(pixels.as_mut());
        c.clear(background);
        if let Some(clip) = clip {
            c.clip_rect(clip);
        }
        if cached {
            plot.paint(&mut c, Offset::new(24., 20.));
        } else {
            c.save();
            c.translate(24., 20.);
            c.clip_rect(Rect::from_ltwh(0., 0., plot.size.width, plot.size.height));
            plot.paint_plot(&mut c);
            c.restore();
        }
        pixels
    }
    #[test]
    fn native_plot_replay_matches_original_aa_for_themes_data_selection_and_clips() {
        let state = state();
        state.lock().unwrap().usage.period = Period::Month;
        for light in [false, true] {
            Folio::configure(light, 65);
            for n in [0usize, 1, 14, 24, 31, 240] {
                for selected in [0, n / 2, n.saturating_sub(1)] {
                    for size in [
                        Size::new(540., 71.),
                        Size::new(320., 104.),
                        Size::new(540.5, 71.),
                    ] {
                        let plot = plot(&state, n, selected, true, size);
                        for (background, clip) in [
                            (Color::BLACK, None),
                            (Color::GREEN, None),
                            (Color::RED, Some(Rect::from_ltwh(36., 25., 170., 30.))),
                        ] {
                            assert_eq!(
                                paint_plot(&plot, true, background, clip),
                                paint_plot(&plot, false, background, clip)
                            );
                        }
                        assert!(tiny_flutter::graphics::vector_cache_stats().1 <= 6 * 1024 * 1024);
                    }
                }
            }
        }
    }
    fn paint_card(card: &dyn RenderBox) -> Pixmap565 {
        let mut pixels = Pixmap565::new(620, 220).unwrap();
        let mut c = Canvas::new(pixels.as_mut());
        c.clear(Folio::bg());
        card.paint(&mut c, Offset::new(20., 18.));
        pixels
    }
    #[test]
    fn day_axis_keeps_real_hours_fixed_as_samples_arrive_and_future_is_not_selectable() {
        let state = state();
        let early = plot(&state, 13, 12, false, Size::new(484., 71.));
        let later = plot(&state, 24, 23, false, Size::new(484., 71.));
        assert_eq!(early.x(&early.points, 0), Some(2.));
        assert_eq!(early.x(&early.points, 6), Some(122.));
        assert_eq!(early.x(&early.points, 12), Some(242.));
        assert_eq!(early.x(&early.points, 12), later.x(&later.points, 12));
        assert_eq!(later.x(&later.points, 23), Some(462.));
        assert_eq!(early.selection_at(122.), Some(6));
        assert_eq!(early.selection_at(242.), Some(12));
        assert_eq!(early.selection_at(362.), None);
        let midnight = plot(&state, 1, 0, false, Size::new(484., 71.));
        assert_eq!(midnight.x(&midnight.points, 0), Some(2.));
        assert_eq!(midnight.selection_at(242.), None);
        state.lock().unwrap().usage.period = Period::Month;
        let month = plot(&state, 13, 12, false, Size::new(484., 71.));
        assert_eq!(month.x(&month.points, 6), Some(242.));
        assert_eq!(month.x(&month.points, 12), Some(482.));
    }
    #[test]
    fn todays_line_and_fill_stop_at_observed_hour_while_yesterday_remains_full_day() {
        let state = state();
        for light in [false, true] {
            Folio::configure(light, 65);
            let mut today = plot(&state, 13, 12, true, Size::new(484., 71.));
            today.points = Arc::new(
                points(13, 20)
                    .into_iter()
                    .map(|mut p| {
                        p.totals.tokens = 100;
                        p
                    })
                    .collect(),
            );
            today.previous = Arc::new(
                points(24, 20)
                    .into_iter()
                    .map(|mut p| {
                        p.totals.tokens = 600;
                        p
                    })
                    .collect(),
            );
            today.signature = plot_signature(
                &state,
                &today.points,
                &today.previous,
                today.selected,
                today.style,
            );
            let actual = paint_plot(&today, true, Color::BLACK, None);
            assert_eq!(actual, paint_plot(&today, false, Color::BLACK, None));
            let mut yesterday_only = plot(&state, 0, 0, false, today.size);
            yesterday_only.previous = today.previous.clone();
            yesterday_only.signature = plot_signature(
                &state,
                &yesterday_only.points,
                &yesterday_only.previous,
                0,
                yesterday_only.style,
            );
            let previous = paint_plot(&yesterday_only, false, Color::BLACK, None);
            // The current marker is at screen x=24+242. Beyond its AA radius,
            // every future pixel must match the complete yesterday-only plot.
            for y in 20..91 {
                for x in 274..508 {
                    assert_eq!(
                        actual.data()[y * 600 + x],
                        previous.data()[y * 600 + x],
                        "future pixel {x},{y}"
                    );
                }
            }
            let mut empty = plot(&state, 0, 0, false, today.size);
            empty.signature =
                plot_signature(&state, &empty.points, &empty.previous, 0, empty.style);
            let empty_pixels = paint_plot(&empty, false, Color::BLACK, None);
            assert!(
                (274..490)
                    .any(|x| previous.data()[22 * 600 + x] != empty_pixels.data()[22 * 600 + x]),
                "yesterday continues after noon"
            );
        }
    }
    #[test]
    fn unchanged_tokens_at_a_new_hour_invalidate_cached_coordinates() {
        let state = state();
        Folio::configure(false, 65);
        let initial = plot(&state, 2, 1, false, Size::new(484., 71.));
        let before = paint_plot(&initial, true, Color::BLACK, None);
        let mut changed = plot(&state, 2, 1, false, initial.size);
        let mut points = changed.points.as_ref().clone();
        points[1].date = "2026-10-07 12:00".into();
        changed.points = Arc::new(points);
        changed.signature = plot_signature(
            &state,
            &changed.points,
            &changed.previous,
            changed.selected,
            changed.style,
        );
        assert_ne!(initial.signature, changed.signature);
        let after = paint_plot(&changed, true, Color::BLACK, None);
        assert_ne!(before, after);
        assert_eq!(after, paint_plot(&changed, false, Color::BLACK, None));
    }
    #[test]
    fn live_card_refreshes_all_labels_data_and_selection_without_replacing_root() {
        Folio::configure(false, 65);
        let state = state();
        let size = Size::new(576., 184.);
        let initial = state.lock().unwrap().usage.data.clone().unwrap();
        let mut card = LiveTrendCard::new(state.clone(), initial, size).create_render_object();
        card.layout(&BoxConstraints::tight(size));
        assert!(card.animation_dirty().is_none());
        for change in 0..10 {
            {
                let mut state = state.lock().unwrap();
                let mut next = state.usage.data.as_ref().unwrap().as_ref().clone();
                match change {
                    0 => next.trend[7].totals.tokens += 135_711,
                    1 => next.yesterday_trend[7].totals.tokens += 513_711,
                    2 => next.trend.last_mut().unwrap().totals.cost = 71.21,
                    3 => next.trend.last_mut().unwrap().date = "2026-10-06 23:59".into(),
                    4 => state.usage.chart_selected = Some(7),
                    5 => next.trend[7].totals.requests = Some(98),
                    6 => next.yesterday_trend.clear(),
                    7 => next.trend_label = "最近时段".into(),
                    8 => next.trend.push(points(1, 91).remove(0)),
                    _ => next.trend.clear(),
                }
                state.usage.data = Some(Arc::new(next));
            }
            assert_eq!(
                card.animation_dirty(),
                Some(Rect::from_ltwh(0., 0., size.width, size.height))
            );
            card.layout(&BoxConstraints::tight(size));
            assert!(card.animation_dirty().is_none());
            let data = state.lock().unwrap().usage.data.clone().unwrap();
            let mut reference = chart_snapshot(state.clone(), &data, size.width, size.height)
                .create_render_object();
            reference.layout(&BoxConstraints::tight(size));
            assert_eq!(paint_card(&*card), paint_card(&*reference));
        }
    }
    #[test]
    fn live_card_does_not_schedule_background_overlay_or_previous_scope() {
        let state = state();
        let size = Size::new(576., 184.);
        let data = state.lock().unwrap().usage.data.clone().unwrap();
        let mut card = LiveTrendCard::new(state.clone(), data, size).create_render_object();
        card.layout(&BoxConstraints::tight(size));
        {
            let mut state = state.lock().unwrap();
            let mut next = state.usage.data.as_ref().unwrap().as_ref().clone();
            next.trend[0].totals.tokens += 1;
            state.usage.data = Some(Arc::new(next));
            state.status_panel_open = true;
        }
        assert!(card.animation_dirty().is_none());
        state.lock().unwrap().status_panel_open = false;
        assert!(card.animation_dirty().is_some());
        state.lock().unwrap().background_active_app();
        assert!(card.animation_dirty().is_none());
        state.lock().unwrap().open_app("sub2api-monitor");
        assert!(card.animation_dirty().is_some());
        state
            .lock()
            .unwrap()
            .usage
            .navigate(Page::Overview, Period::Month, None);
        assert!(card.animation_dirty().is_none());
    }
    #[test]
    fn theme_layout_and_scope_changes_never_leave_a_repeating_dirty_or_old_plot() {
        let state = state();
        let size = Size::new(576., 184.);
        let data = state.lock().unwrap().usage.data.clone().unwrap();
        let mut card = LiveTrendCard::new(state.clone(), data, size).create_render_object();
        for light in [false, true, false, true] {
            Folio::configure(light, 65);
            card.layout(&BoxConstraints::tight(size));
            assert!(card.animation_dirty().is_none());
            let data = state.lock().unwrap().usage.data.clone().unwrap();
            let mut reference = chart_snapshot(state.clone(), &data, size.width, size.height)
                .create_render_object();
            reference.layout(&BoxConstraints::tight(size));
            assert_eq!(paint_card(&*card), paint_card(&*reference));
        }
        let style = PlotStyle::current();
        let initial = {
            let data = state.lock().unwrap().usage.data.clone().unwrap();
            plot_signature(&state, &data.trend, &data.yesterday_trend, 0, style).unwrap()
        };
        for (page, period, detail) in [
            (Page::Overview, Period::Month, None),
            (Page::Users, Period::Day, Some(-1)),
            (Page::Accounts, Period::Day, Some(0)),
        ] {
            let data = state.lock().unwrap().usage.data.clone().unwrap();
            {
                let mut state = state.lock().unwrap();
                state.usage.page = page;
                state.usage.period = period;
                state.usage.detail = detail;
                state.usage.revision += 1;
            }
            assert_ne!(
                initial,
                plot_signature(&state, &data.trend, &data.yesterday_trend, 0, style).unwrap()
            );
            assert!(card.animation_dirty().is_none());
            card.layout(&BoxConstraints::tight(size));
            assert!(card.animation_dirty().is_none());
        }
    }
}
