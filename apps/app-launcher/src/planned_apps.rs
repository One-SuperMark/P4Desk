//! Three requested launch entries. Their feature implementations are not enabled yet.
use serde::{Deserialize, Serialize};
use tiny_flutter::prelude::*;
use tiny_flutter::theme::Folio;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlannedApp {
    Files,
    Office,
    Usage,
}
impl PlannedApp {
    pub fn id(self) -> &'static str {
        match self {
            Self::Files => "file-manager",
            Self::Office => "office-viewer",
            Self::Usage => "sub2api-monitor",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Files => "文件管理",
            Self::Office => "Office",
            Self::Usage => "用量监控",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Files => "TF 卡文件浏览与管理",
            Self::Office => "Office 文档查看",
            Self::Usage => "查看服务用量与账户状态",
        }
    }
}
pub fn build(app: PlannedApp, size: Size) -> impl Widget {
    Container::new()
        .width(size.width)
        .height(size.height)
        .child(Center::new(
            Column::new()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .push(crate::widgets::app_badge(app.id(), 128.0))
                .push(SizedBox::square(20.0))
                .push(Text::new(app.title()).font_size(28.0).color(Folio::ink()))
                .push(SizedBox::square(12.0))
                .push(
                    Text::new(app.description())
                        .font_size(22.0)
                        .color(Folio::muted()),
                )
                .push(SizedBox::square(28.0))
                .push(
                    Container::new()
                        .width(180.0)
                        .height(42.0)
                        .color(Folio::raised())
                        .border_radius(21.0)
                        .child(Center::new(
                            Text::new("功能准备中")
                                .font_size(18.0)
                                .color(Folio::muted()),
                        )),
                ),
        ))
}
