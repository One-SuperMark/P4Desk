//! Original P4Desk desktop icons. Source SVGs and the deterministic baker live in
//! `assets/app_icons/` and `scripts/generate-desktop-assets.py` (MIT).

#[cfg(not(target_endian = "little"))]
compile_error!("P4Desk RGB565 icon assets require a little-endian target");

pub struct AppIconAsset {
    pub width: u32,
    pub height: u32,
    /// Unassociated RGB565 colors, one little-endian u16 per pixel.
    pub rgb565: &'static [u8],
    /// Linear pixel coverage, one u8 per pixel (0 transparent, 255 opaque).
    pub alpha: &'static [u8],
}

impl AppIconAsset {
    pub fn get_rgb565_slice(&self) -> &'static [u16] {
        let pixels = self.width as usize * self.height as usize;
        assert_eq!(self.rgb565.len(), pixels * 2, "invalid RGB565 asset length");
        assert_eq!(
            self.rgb565
                .as_ptr()
                .align_offset(core::mem::align_of::<u16>()),
            0,
            "RGB565 asset must be aligned to 2 bytes"
        );
        // SAFETY: icon storage below has explicit repr(align(2)); length is
        // checked above, bytes have static lifetime and every u16 bit pattern
        // is valid. The compile-time endian gate matches the generated format.
        unsafe { core::slice::from_raw_parts(self.rgb565.as_ptr().cast::<u16>(), pixels) }
    }
}

#[repr(C, align(2))]
struct AlignedRgb565<const N: usize>([u8; N]);

pub const DESKTOP_ICON_SIDE: u32 = 146;
const SIDE: u32 = DESKTOP_ICON_SIDE;
const RGB565_BYTES: usize = SIDE as usize * SIDE as usize * 2;

macro_rules! icon_asset {
    ($storage:ident, $asset:ident, $id:literal) => {
        static $storage: AlignedRgb565<RGB565_BYTES> = AlignedRgb565(*include_bytes!(concat!(
            "../../../assets/app_icons/",
            $id,
            ".rgb565"
        )));
        static $asset: AppIconAsset = AppIconAsset {
            width: SIDE,
            height: SIDE,
            rgb565: &$storage.0,
            alpha: include_bytes!(concat!("../../../assets/app_icons/", $id, ".alpha")),
        };
    };
}

icon_asset!(CLOCK_PIXELS, CLOCK_ICON, "clock");
icon_asset!(TIMER_PIXELS, TIMER_ICON, "timer");
icon_asset!(NOTES_PIXELS, NOTES_ICON, "notes");
icon_asset!(CALCULATOR_PIXELS, CALCULATOR_ICON, "calculator");
icon_asset!(MAC_PIXELS, MAC_ICON, "mac");
icon_asset!(SETTINGS_PIXELS, SETTINGS_ICON, "settings");
icon_asset!(DISPLAY_PIXELS, DISPLAY_ICON, "display");
icon_asset!(SCREEN_PIXELS, SCREEN_ICON, "screen");

pub fn get_app_icon_asset(id: &str) -> Option<&'static AppIconAsset> {
    match id {
        "clock" => Some(&CLOCK_ICON),
        "timer" => Some(&TIMER_ICON),
        "notes" => Some(&NOTES_ICON),
        "calculator" => Some(&CALCULATOR_ICON),
        "mac" => Some(&MAC_ICON),
        "settings" => Some(&SETTINGS_ICON),
        "display" => Some(&DISPLAY_ICON),
        "screen" => Some(&SCREEN_ICON),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDS: [&str; 8] = [
        "clock",
        "timer",
        "notes",
        "calculator",
        "mac",
        "settings",
        "display",
        "screen",
    ];

    #[test]
    fn resource_dimensions_lengths_and_alignment_match_renderer_contract() {
        for id in IDS {
            let asset = get_app_icon_asset(id).expect("known desktop icon");
            assert_eq!((asset.width, asset.height), (146, 146));
            assert_eq!(asset.rgb565.len(), 42632);
            assert_eq!(asset.alpha.len(), 21316);
            assert_eq!(asset.rgb565.as_ptr() as usize % 2, 0);
            assert_eq!(asset.get_rgb565_slice().len(), 21316);
        }
        assert!(get_app_icon_asset("unknown").is_none());
    }

    #[test]
    fn aligned_words_decode_every_pixel_as_little_endian() {
        for id in IDS {
            let asset = get_app_icon_asset(id).unwrap();
            for (bytes, word) in asset.rgb565.chunks_exact(2).zip(asset.get_rgb565_slice()) {
                assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), *word);
            }
        }
    }

    #[test]
    fn rounded_icons_have_transparency_and_antialiased_coverage() {
        for id in IDS {
            let alpha = get_app_icon_asset(id).unwrap().alpha;
            for corner in [0, 145, 146 * 145, 21315] {
                assert_eq!(alpha[corner], 0);
            }
            assert!(alpha.contains(&255));
            assert!(alpha.iter().any(|value| *value > 0 && *value < 255));
        }
    }
}
