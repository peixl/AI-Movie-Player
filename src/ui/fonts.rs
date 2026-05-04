use std::path::Path;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

const CJK_FONT_NAME: &str = "system_cjk_fallback";

pub fn install(ctx: &egui::Context) {
    let Some((font_path, font_bytes)) = load_system_cjk_font() else {
        log::warn!("No system CJK font found; Chinese UI text may not render correctly");
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(CJK_FONT_NAME.to_owned(), Arc::new(FontData::from_owned(font_bytes)));

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(family_fonts) = fonts.families.get_mut(&family) {
            family_fonts.retain(|name| name != CJK_FONT_NAME);
            family_fonts.push(CJK_FONT_NAME.to_owned());
        }
    }

    ctx.set_fonts(fonts);
    log::info!("Loaded CJK UI font: {}", font_path.display());
}

fn load_system_cjk_font() -> Option<(&'static Path, Vec<u8>)> {
    for candidate in system_cjk_font_candidates() {
        let path = Path::new(candidate);
        match std::fs::read(path) {
            Ok(bytes) => return Some((path, bytes)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => log::debug!("Skipping CJK font {}: {}", path.display(), err),
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn system_cjk_font_candidates() -> &'static [&'static str] {
    &[
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
    ]
}

#[cfg(target_os = "windows")]
fn system_cjk_font_candidates() -> &'static [&'static str] {
    &[
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyhbd.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\Deng.ttf",
        r"C:\Windows\Fonts\NotoSansCJK-Regular.ttc",
    ]
}

#[cfg(target_os = "linux")]
fn system_cjk_font_candidates() -> &'static [&'static str] {
    &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf",
        "/usr/share/fonts/truetype/arphic/uming.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn system_cjk_font_candidates() -> &'static [&'static str] {
    &[]
}

#[cfg(test)]
mod tests {
    use super::system_cjk_font_candidates;

    #[test]
    fn cjk_font_candidates_are_configured_for_supported_platforms() {
        if cfg!(any(target_os = "macos", target_os = "windows", target_os = "linux")) {
            assert!(!system_cjk_font_candidates().is_empty());
        }
    }
}
