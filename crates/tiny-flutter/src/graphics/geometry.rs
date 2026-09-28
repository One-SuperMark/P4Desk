use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add<Offset> for Point {
    type Output = Point;
    fn add(self, rhs: Offset) -> Self::Output {
        Point::new(self.x + rhs.dx, self.y + rhs.dy)
    }
}

impl Sub<Point> for Point {
    type Output = Offset;
    fn sub(self, rhs: Point) -> Self::Output {
        Offset::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Offset> for Point {
    type Output = Point;
    fn sub(self, rhs: Offset) -> Self::Output {
        Point::new(self.x - rhs.dx, self.y - rhs.dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };
    pub const INFINITY: Size = Size {
        width: f32::INFINITY,
        height: f32::INFINITY,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    pub dx: f32,
    pub dy: f32,
}

impl Offset {
    pub const ZERO: Offset = Offset { dx: 0.0, dy: 0.0 };

    pub const fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
}

impl Add for Offset {
    type Output = Offset;
    fn add(self, rhs: Self) -> Self::Output {
        Offset::new(self.dx + rhs.dx, self.dy + rhs.dy)
    }
}

impl Sub for Offset {
    type Output = Offset;
    fn sub(self, rhs: Self) -> Self::Output {
        Offset::new(self.dx - rhs.dx, self.dy - rhs.dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            x: left,
            y: top,
            width,
            height,
        }
    }

    pub const fn from_origin_size(origin: Point, size: Size) -> Self {
        Self {
            x: origin.x,
            y: origin.y,
            width: size.width,
            height: size.height,
        }
    }

    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: left,
            y: top,
            width: (right - left).max(0.0),
            height: (bottom - top).max(0.0),
        }
    }

    pub fn left(&self) -> f32 {
        self.x
    }
    pub fn top(&self) -> f32 {
        self.y
    }
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn shift(&self, offset: Offset) -> Rect {
        Rect::from_ltwh(
            self.x + offset.dx,
            self.y + offset.dy,
            self.width,
            self.height,
        )
    }

    pub fn inflate(&self, delta: f32) -> Rect {
        Rect::from_ltrb(
            self.left() - delta,
            self.top() - delta,
            self.right() + delta,
            self.bottom() + delta,
        )
    }

    pub fn deflate(&self, delta: f32) -> Rect {
        self.inflate(-delta)
    }

    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > left && bottom > top {
            Some(Rect::from_ltrb(left, top, right, bottom))
        } else {
            None
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        if self.width <= 0.0 || self.height <= 0.0 {
            return *other;
        }
        if other.width <= 0.0 || other.height <= 0.0 {
            return *self;
        }
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::from_ltrb(left, top, right, bottom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
    pub x: f32,
    pub y: f32,
}

impl Radius {
    pub const ZERO: Radius = Radius { x: 0.0, y: 0.0 };

    pub const fn circular(radius: f32) -> Self {
        Self {
            x: radius,
            y: radius,
        }
    }

    pub const fn ellipitical(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RRect {
    pub rect: Rect,
    pub radius: Radius,
}

impl RRect {
    pub const fn from_rect_and_radius(rect: Rect, radius: Radius) -> Self {
        Self { rect, radius }
    }

    pub const fn from_rect_circular(rect: Rect, radius: f32) -> Self {
        Self {
            rect,
            radius: Radius::circular(radius),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    pub const ZERO: EdgeInsets = EdgeInsets {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    pub const fn ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub const fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub fn deflate_size(&self, size: Size) -> Size {
        Size::new(
            (size.width - self.horizontal()).max(0.0),
            (size.height - self.vertical()).max(0.0),
        )
    }

    pub fn inflate_size(&self, size: Size) -> Size {
        Size::new(
            size.width + self.horizontal(),
            size.height + self.vertical(),
        )
    }
}
