use crate::rendering::flex::{
    Axis, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, RenderFlex,
};
use crate::rendering::render_box::RenderBox;
use crate::widgets::widget::Widget;

pub struct Column {
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
    pub children: Vec<Box<dyn Widget>>,
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Column {
    pub fn new() -> Self {
        Self {
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            main_axis_size: MainAxisSize::Max,
            children: Vec::new(),
        }
    }

    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }

    pub fn push(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }
}

impl Widget for Column {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let render_children = self
            .children
            .iter()
            .map(|c| c.create_render_object())
            .collect();
        Box::new(RenderFlex::new(
            Axis::Vertical,
            self.main_axis_alignment,
            self.cross_axis_alignment,
            self.main_axis_size,
            render_children,
        ))
    }
}

pub struct Row {
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
    pub children: Vec<Box<dyn Widget>>,
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Row {
    pub fn new() -> Self {
        Self {
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            main_axis_size: MainAxisSize::Max,
            children: Vec::new(),
        }
    }

    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }

    pub fn push(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }
}

impl Widget for Row {
    fn create_render_object(&self) -> Box<dyn RenderBox> {
        let render_children = self
            .children
            .iter()
            .map(|c| c.create_render_object())
            .collect();
        Box::new(RenderFlex::new(
            Axis::Horizontal,
            self.main_axis_alignment,
            self.cross_axis_alignment,
            self.main_axis_size,
            render_children,
        ))
    }
}

/// Declarative helper macro to build a Column with a list of child widgets.
#[macro_export]
macro_rules! col {
    ($($child:expr),* $(,)?) => {
        $crate::widgets::Column::new()
            $(.push($child))*
    };
}

/// Declarative helper macro to build a Row with a list of child widgets.
#[macro_export]
macro_rules! row {
    ($($child:expr),* $(,)?) => {
        $crate::widgets::Row::new()
            $(.push($child))*
    };
}
