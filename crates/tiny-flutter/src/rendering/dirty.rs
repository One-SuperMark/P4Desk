use crate::graphics::geometry::{Rect, Size};

/// Manages and tracks damaged dirty rectangles to avoid full-screen software rasterization.
#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyRegion {
    bounds: Option<Rect>,
}

impl DirtyRegion {
    pub fn new() -> Self {
        Self { bounds: None }
    }

    /// Mark an arbitrary rectangular region as dirty.
    pub fn mark_dirty(&mut self, rect: Rect) {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        self.bounds = match self.bounds {
            Some(existing) => Some(existing.union(&rect)),
            None => Some(rect),
        };
    }

    /// Mark the entire screen/viewport as dirty.
    pub fn mark_all_dirty(&mut self, size: Size) {
        self.mark_dirty(Rect::from_ltwh(0.0, 0.0, size.width, size.height));
    }

    /// Extract and clear current dirty bounding box for rendering.
    pub fn take(&mut self) -> Option<Rect> {
        self.bounds.take()
    }

    pub fn is_empty(&self) -> bool {
        self.bounds.is_none()
    }

    pub fn current(&self) -> Option<Rect> {
        self.bounds
    }
}
