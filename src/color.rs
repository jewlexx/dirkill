use thiserror::Error as AsError;
use tui::style::Color;

#[derive(Debug, AsError)]
pub enum ColorError {
    #[error("Invalid hex provided")]
    InvalidHex,
}

pub fn parse_hex(hex: impl AsRef<str>) -> Result<Color, ColorError> {
    let hex = hex.as_ref();

    if !hex.starts_with("#") {
        return Err(ColorError::InvalidHex);
    }

    // TMP
    Ok(Color::Yellow)
}
