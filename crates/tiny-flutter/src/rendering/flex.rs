use crate::graphics::canvas::Canvas;
use crate::graphics::geometry::{Offset, Point, Rect, Size};
use crate::rendering::constraints::BoxConstraints;
use crate::rendering::render_box::{RenderBox, TouchEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainAxisAlignment {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossAxisAlignment {
    #[default]
    Center,
    Start,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainAxisSize {
    #[default]
    Max,
    Min,
}

pub struct RenderFlex {
    pub direction: Axis,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
    pub children: Vec<Box<dyn RenderBox>>,
    size: Size,
    offset: Offset,
}

impl RenderFlex {
    pub fn new(
        direction: Axis,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        main_axis_size: MainAxisSize,
        children: Vec<Box<dyn RenderBox>>,
    ) -> Self {
        Self {
            direction,
            main_axis_alignment,
            cross_axis_alignment,
            main_axis_size,
            children,
            size: Size::ZERO,
            offset: Offset::ZERO,
        }
    }
}

impl RenderBox for RenderFlex {
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
        let is_horiz = self.direction == Axis::Horizontal;

        let max_main = if is_horiz {
            constraints.max_width
        } else {
            constraints.max_height
        };
        let max_cross = if is_horiz {
            constraints.max_height
        } else {
            constraints.max_width
        };

        let total_flex: u32 = self.children.iter().map(|c| c.flex()).sum();

        let mut total_allocated_main = 0.0f32;
        let mut max_child_cross = 0.0f32;

        let non_flex_constraints = if is_horiz {
            BoxConstraints::new(0.0, f32::INFINITY, 0.0, max_cross)
        } else {
            BoxConstraints::new(0.0, max_cross, 0.0, f32::INFINITY)
        };

        // Pass 1: Measure non-flexible children
        for child in self.children.iter_mut() {
            if child.flex() == 0 {
                let child_size = child.layout(&non_flex_constraints);
                let (child_main, child_cross) = if is_horiz {
                    (child_size.width, child_size.height)
                } else {
                    (child_size.height, child_size.width)
                };

                total_allocated_main += child_main;
                if child_cross > max_child_cross {
                    max_child_cross = child_cross;
                }
            }
        }

        // Pass 2: Distribute remaining main space to flexible children (Expanded)
        if total_flex > 0 {
            let available_main = if max_main.is_finite() {
                max_main
            } else {
                total_allocated_main
            };
            let remaining_main = (available_main - total_allocated_main).max(0.0);
            let space_per_flex = remaining_main / total_flex as f32;

            for child in self.children.iter_mut() {
                let flex = child.flex();
                if flex > 0 {
                    let child_flex_main = (space_per_flex * flex as f32).max(0.0);
                    let flex_constraints = if is_horiz {
                        BoxConstraints::tight_for(Some(child_flex_main), Some(max_cross))
                    } else {
                        BoxConstraints::tight_for(Some(max_cross), Some(child_flex_main))
                    };
                    let child_size = child.layout(&flex_constraints);
                    let (child_main, child_cross) = if is_horiz {
                        (child_size.width, child_size.height)
                    } else {
                        (child_size.height, child_size.width)
                    };

                    total_allocated_main += child_main;
                    if child_cross > max_child_cross {
                        max_child_cross = child_cross;
                    }
                }
            }
        }

        let actual_main = match self.main_axis_size {
            MainAxisSize::Max if max_main.is_finite() => max_main,
            _ => total_allocated_main,
        };

        let resolved_cross = if is_horiz {
            constraints.constrain_height(max_child_cross)
        } else {
            constraints.constrain_width(max_child_cross)
        };

        let free_space = (actual_main - total_allocated_main).max(0.0);
        let child_count = self.children.len();

        let (leading_space, between_space) = if child_count == 0 {
            (0.0, 0.0)
        } else {
            match self.main_axis_alignment {
                MainAxisAlignment::Start => (0.0, 0.0),
                MainAxisAlignment::End => (free_space, 0.0),
                MainAxisAlignment::Center => (free_space / 2.0, 0.0),
                MainAxisAlignment::SpaceBetween => {
                    if child_count > 1 {
                        (0.0, free_space / (child_count - 1) as f32)
                    } else {
                        (0.0, 0.0)
                    }
                }
                MainAxisAlignment::SpaceAround => {
                    let unit = free_space / child_count as f32;
                    (unit / 2.0, unit)
                }
                MainAxisAlignment::SpaceEvenly => {
                    let unit = free_space / (child_count + 1) as f32;
                    (unit, unit)
                }
            }
        };

        let mut current_main = leading_space;
        for child in self.children.iter_mut() {
            let child_size = child.size();
            let (child_main, child_cross) = if is_horiz {
                (child_size.width, child_size.height)
            } else {
                (child_size.height, child_size.width)
            };

            let cross_offset = match self.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::End => resolved_cross - child_cross,
                CrossAxisAlignment::Center => (resolved_cross - child_cross) / 2.0,
                CrossAxisAlignment::Stretch => 0.0,
            };

            let child_offset = if is_horiz {
                Offset::new(current_main, cross_offset)
            } else {
                Offset::new(cross_offset, current_main)
            };
            child.set_offset(child_offset);

            current_main += child_main + between_space;
        }

        self.size = if is_horiz {
            constraints.constrain(Size::new(actual_main, resolved_cross))
        } else {
            constraints.constrain(Size::new(resolved_cross, actual_main))
        };

        self.size
    }

    fn paint(&self, canvas: &mut Canvas, offset: Offset) {
        let clip = canvas.current_clip();
        for child in &self.children {
            let child_offset = offset + child.offset();
            if let Some(clip_rect) = clip {
                let s = child.size();
                let child_rect =
                    Rect::from_ltwh(child_offset.dx, child_offset.dy, s.width, s.height);
                if child_rect.bottom() < clip_rect.y || child_rect.y > clip_rect.bottom() {
                    continue;
                }
            }
            child.paint(canvas, child_offset);
        }
    }

    fn dispatch_touch(&mut self, event: &TouchEvent) -> bool {
        let local_point = event.point();
        let is_up = matches!(event, TouchEvent::Up(_));

        // Dispatch backwards (front-to-back in render order)
        for child in self.children.iter_mut().rev() {
            let child_offset = child.offset();
            let child_local_p = local_point - child_offset;

            if child.hit_test(child_local_p) || is_up || matches!(event, TouchEvent::Cancel) {
                let child_event = event.transform(child_local_p);
                if child.dispatch_touch(&child_event) {
                    return true;
                }
            }
        }
        false
    }

    fn set_pressed_at(&mut self, point: Point, pressed: bool) {
        for child in self.children.iter_mut().rev() {
            let child_offset = child.offset();
            let child_local_p = point - child_offset;
            if child.hit_test(child_local_p) {
                child.set_pressed_at(child_local_p, pressed);
            }
        }
    }

    fn hit_rect(&self, point: Point) -> Option<Rect> {
        for child in self.children.iter().rev() {
            let child_offset = child.offset();
            let child_local_p = point - child_offset;
            if child.hit_test(child_local_p) {
                if let Some(r) = child.hit_rect(child_local_p) {
                    return Some(Rect::from_ltwh(
                        r.x + child_offset.dx,
                        r.y + child_offset.dy,
                        r.width,
                        r.height,
                    ));
                }
            }
        }
        None
    }
}
