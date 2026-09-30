use crate::storage::LocalSettings;
use tiny_flutter::theme::{Folio, GlassBackdrop};

pub fn configure(settings: &LocalSettings) {
    crate::icon_theme::configure(settings.icon_theme);
    crate::icon_theme::configure_light(Some(settings.light_icons()));
    Folio::configure(settings.light_appearance, settings.glass_amount);
    Folio::set_backdrop(GlassBackdrop {
        pixels: if settings.light_appearance {
            include_bytes!("../../../assets/wallpaper/glass-light.rgb888")
        } else {
            include_bytes!("../../../assets/wallpaper/glass-dark.rgb888")
        },
        width: 256,
        height: 150,
    });
}
