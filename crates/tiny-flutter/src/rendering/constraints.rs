use crate::graphics::geometry::{EdgeInsets, Size};

/// Immutable layout constraints passed from parent to child in the render tree.
/// Direct implementation of Flutter's BoxConstraints protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl BoxConstraints {
    pub const ZERO: BoxConstraints = BoxConstraints {
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
    };

    pub const UNCONSTRAINED: BoxConstraints = BoxConstraints {
        min_width: 0.0,
        max_width: f32::INFINITY,
        min_height: 0.0,
        max_height: f32::INFINITY,
    };

    pub const fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    /// Creates box constraints that require the child to have exactly the specified size.
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Creates box constraints with specified tight width or height if provided.
    pub fn tight_for(width: Option<f32>, height: Option<f32>) -> Self {
        Self {
            min_width: width.unwrap_or(0.0),
            max_width: width.unwrap_or(f32::INFINITY),
            min_height: height.unwrap_or(0.0),
            max_height: height.unwrap_or(f32::INFINITY),
        }
    }

    /// Creates box constraints that forbid sizes larger than the given size, but permit smaller ones.
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    /// Returns new box constraints that remove minimum width and height constraints.
    pub fn loosen(&self) -> Self {
        Self {
            min_width: 0.0,
            max_width: self.max_width,
            min_height: 0.0,
            max_height: self.max_height,
        }
    }

    /// Creates box constraints that expand to fill the available space.
    pub fn expand(width: Option<f32>, height: Option<f32>) -> Self {
        Self {
            min_width: width.unwrap_or(f32::INFINITY),
            max_width: width.unwrap_or(f32::INFINITY),
            min_height: height.unwrap_or(f32::INFINITY),
            max_height: height.unwrap_or(f32::INFINITY),
        }
    }

    /// Returns new box constraints that respect the given constraints while being as close as
    /// possible to the original constraints (exact Flutter implementation).
    pub fn enforce(&self, constraints: &BoxConstraints) -> Self {
        let max_w = constraints.max_width.max(constraints.min_width);
        let max_h = constraints.max_height.max(constraints.min_height);
        Self {
            min_width: self.min_width.clamp(constraints.min_width, max_w),
            max_width: self.max_width.clamp(constraints.min_width, max_w),
            min_height: self.min_height.clamp(constraints.min_height, max_h),
            max_height: self.max_height.clamp(constraints.min_height, max_h),
        }
    }

    /// Returns the size that is closest to the given size while satisfying constraints.
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            self.constrain_width(size.width),
            self.constrain_height(size.height),
        )
    }

    pub fn constrain_width(&self, width: f32) -> f32 {
        let max_w = self.max_width.max(self.min_width);
        width.clamp(self.min_width, max_w)
    }

    pub fn constrain_height(&self, height: f32) -> f32 {
        let max_h = self.max_height.max(self.min_height);
        height.clamp(self.min_height, max_h)
    }

    pub fn is_tight(&self) -> bool {
        self.min_width >= self.max_width && self.min_height >= self.max_height
    }

    pub fn has_bounded_width(&self) -> bool {
        self.max_width.is_finite()
    }

    pub fn has_bounded_height(&self) -> bool {
        self.max_height.is_finite()
    }

    pub fn deflate(&self, edges: EdgeInsets) -> Self {
        let horizontal = edges.horizontal();
        let vertical = edges.vertical();

        let deflated_min_width = (self.min_width - horizontal).max(0.0);
        let deflated_min_height = (self.min_height - vertical).max(0.0);
        let deflated_max_width = (self.max_width - horizontal).max(deflated_min_width);
        let deflated_max_height = (self.max_height - vertical).max(deflated_min_height);

        Self {
            min_width: deflated_min_width,
            max_width: deflated_max_width,
            min_height: deflated_min_height,
            max_height: deflated_max_height,
        }
    }
}
