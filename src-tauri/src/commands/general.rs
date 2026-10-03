use crate::error::Result;
use tauri::window::Color;

// index.html's #initial-loader colours, and tauri.conf.json's first one (a test holds them equal).
const DARK: Color = Color(0x0a, 0x0f, 0x1c, 0xff);
const LIGHT: Color = Color(0xee, 0xf1, 0xf6, 0xff);

/// What the window paints before the page does, so a resize never flashes the other theme.
#[tauri::command]
pub fn set_window_background(window: tauri::WebviewWindow, dark: bool) -> Result<()> {
    log::debug!("set_window_background: dark {dark}");
    window.set_background_color(Some(if dark { DARK } else { LIGHT }))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(c: Color) -> String {
        format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
    }

    #[test]
    fn the_window_colours_match_the_loader_and_the_config() {
        let html = include_str!("../../../index.html");
        let config = include_str!("../../tauri.conf.json");
        for colour in [DARK, LIGHT] {
            assert!(html.contains(&format!("background: {};", hex(colour))));
        }
        assert!(config.contains(&format!("\"backgroundColor\": \"{}\"", hex(DARK))));
    }
}
