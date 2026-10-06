//! Live metadata paired with HeadlineView. Network samples update these opaque
//! text strips without replacing the surrounding navigation or dashboard.
use super::{short, Shared};
use crate::LauncherState;
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

#[derive(Clone, Copy)]
enum Kind {
    Cost,
    Status,
}
enum Observation {
    Cost(Option<u64>),
    Status {
        raw: String,
        stale: bool,
        partial: bool,
    },
}
impl Observation {
    fn cost(state: &LauncherState) -> Option<u64> {
        state
            .usage
            .data
            .as_ref()
            .and_then(|d| d.totals.as_ref())
            .map(|t| t.cost.to_bits())
    }
    fn partial(state: &LauncherState) -> bool {
        state
            .usage
            .data
            .as_ref()
            .is_some_and(|d| d.warning.as_deref() == Some(state.usage.status.as_str()))
    }
    fn matches(&self, state: &LauncherState) -> bool {
        match self {
            Self::Cost(cost) => *cost == Self::cost(state),
            Self::Status {
                raw,
                stale,
                partial,
            } => {
                raw == &state.usage.status
                    && *stale == state.usage.stale
                    && *partial == Self::partial(state)
            }
        }
    }
    fn capture(kind: Kind, state: &LauncherState) -> Self {
        match kind {
            Kind::Cost => Self::Cost(Self::cost(state)),
            Kind::Status => Self::Status {
                raw: state.usage.status.clone(),
                stale: state.usage.stale,
                partial: Self::partial(state),
            },
        }
    }
    fn text(&self) -> String {
        match self {
            Self::Cost(cost) => cost
                .map(|bits| format!("${:.2}", f64::from_bits(bits)))
                .unwrap_or_else(|| "—".into()),
            Self::Status {
                raw,
                stale,
                partial,
            } => {
                if *partial {
                    format!("部分未更新 · {}", short(raw, 44))
                } else if *stale {
                    format!("上次数据 · {}", short(raw, 45))
                } else {
                    short(raw, 49)
                }
            }
        }
    }
    fn color(&self) -> Color {
        if matches!(self, Self::Status { stale: true, .. }) {
            Folio::orange()
        } else {
            Folio::muted()
        }
    }
}

pub(super) struct LiveText {
    state: Shared,
    width: f32,
    kind: Kind,
}
impl LiveText {
    pub(super) fn cost(state: Shared, width: f32) -> Self {
        Self {
            state,
            width,
            kind: Kind::Cost,
        }
    }
    pub(super) fn status(state: Shared, width: f32) -> Self {
        Self {
            state,
            width,
            kind: Kind::Status,
        }
    }
}
impl Widget for LiveText {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let revision = self.state.lock().unwrap().usage.revision;
        Box::new(LiveTextBox {
            state: self.state.clone(),
            size: Size::new(self.width, 24.),
            kind: self.kind,
            revision,
            offset: Offset::ZERO,
            observation: None,
            text: String::new(),
        })
    }
}
struct LiveTextBox {
    state: Shared,
    size: Size,
    kind: Kind,
    revision: u64,
    offset: Offset,
    observation: Option<Observation>,
    text: String,
}
impl RenderBox for LiveTextBox {
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
        let state = self.state.lock().unwrap();
        if !self.observation.as_ref().is_some_and(|o| o.matches(&state)) {
            let observation = Observation::capture(self.kind, &state);
            drop(state);
            self.text = observation.text();
            self.observation = Some(observation);
        }
        self.size
    }
    fn animation_dirty(&self) -> Option<Rect> {
        let state = self.state.lock().unwrap();
        if !state.usage_headline_visible() || state.usage.revision != self.revision {
            return None;
        }
        (!self.observation.as_ref().is_some_and(|o| o.matches(&state)))
            .then(|| Rect::from_ltwh(0., 0., self.size.width, self.size.height))
    }
    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let bounds = Rect::from_ltwh(offset.dx, offset.dy, self.size.width, self.size.height);
        if !canvas.is_rect_visible(bounds) {
            return;
        }
        canvas.save();
        canvas.clip_rect(bounds);
        canvas.draw_rect(
            bounds,
            match self.kind {
                Kind::Cost => Folio::surface(),
                Kind::Status => Folio::bg(),
            },
        );
        let size = match self.kind {
            Kind::Cost => 16.,
            Kind::Status => 14.,
        };
        let color = self
            .observation
            .as_ref()
            .map_or(Folio::muted(), Observation::color);
        canvas.draw_text(
            &self.text,
            Font::default_font(),
            size,
            Point::new(offset.dx, offset.dy),
            color,
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
    use crate::usage::{Data, Totals};
    use std::sync::{Arc, Mutex};
    fn fixture() -> Shared {
        let mut state = LauncherState::new();
        state.open_app("sub2api-monitor");
        state.tick(10_000, 1_791_287_400_000);
        state.usage.data = Some(Arc::new(Data {
            totals: Some(Totals {
                tokens: 10,
                cost: 1.23,
                requests: None,
            }),
            ..Data::default()
        }));
        Arc::new(Mutex::new(state))
    }
    #[test]
    fn cost_change_and_absence_damage_only_the_fee_strip() {
        let state = fixture();
        let mut view = LiveText::cost(state.clone(), 200.).create_render_object();
        view.layout(&BoxConstraints::tight(Size::new(200., 24.)));
        assert_eq!(view.animation_dirty(), None);
        state.lock().unwrap().usage.data = Some(Arc::new(Data::default()));
        assert_eq!(
            view.animation_dirty(),
            Some(Rect::from_ltwh(0., 0., 200., 24.))
        );
        view.layout(&BoxConstraints::tight(Size::new(200., 24.)));
        assert_eq!(view.animation_dirty(), None);
    }
    #[test]
    fn status_updates_error_partial_stale_and_recovers_without_a_new_tree() {
        let state = fixture();
        let mut view = LiveText::status(state.clone(), 700.).create_render_object();
        view.layout(&BoxConstraints::tight(Size::new(700., 24.)));
        for (status, stale, warning) in [
            ("网络连接失败", true, None),
            ("趋势未更新", true, Some("趋势未更新")),
            ("更新于 14:30:05", false, None),
        ] {
            let mut s = state.lock().unwrap();
            s.usage.status = status.into();
            s.usage.stale = stale;
            let mut data = (**s.usage.data.as_ref().unwrap()).clone();
            data.warning = warning.map(str::to_owned);
            s.usage.data = Some(Arc::new(data));
            drop(s);
            assert_eq!(
                view.animation_dirty(),
                Some(Rect::from_ltwh(0., 0., 700., 24.))
            );
            view.layout(&BoxConstraints::tight(Size::new(700., 24.)));
            assert_eq!(view.animation_dirty(), None);
        }
    }
    #[test]
    fn hidden_overlay_and_navigated_tree_emit_no_local_damage() {
        let state = fixture();
        let mut view = LiveText::status(state.clone(), 700.).create_render_object();
        view.layout(&BoxConstraints::tight(Size::new(700., 24.)));
        let mut s = state.lock().unwrap();
        s.usage.status = "new".into();
        s.status_panel_open = true;
        drop(s);
        assert_eq!(view.animation_dirty(), None);
        let mut s = state.lock().unwrap();
        s.status_panel_open = false;
        s.usage.navigate(
            crate::usage::Page::Overview,
            crate::usage::Period::Month,
            None,
        );
        drop(s);
        assert_eq!(view.animation_dirty(), None);
    }
}
