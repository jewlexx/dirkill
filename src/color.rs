use std::str::Chars;

use thiserror::Error as AsError;
use tui::style::Color;

#[derive(Debug, AsError)]
pub enum ColorError {
    #[error("Invalid hex provided")]
    InvalidHex,
    #[error("Hex value includes alpha channel")]
    IncludesAlpha,
}

fn validate_chars(chars: Chars) -> bool {
    chars.all(|c| c.is_ascii_hexdigit())
}

pub fn parse_hex(raw_hex: impl AsRef<str>) -> Result<Color, ColorError> {
    let raw_hex = raw_hex.as_ref();

    if !raw_hex.starts_with("#") {
        return Err(ColorError::InvalidHex);
    }

    let hex_value = raw_hex.trim_start_matches("#");

    validate_chars(hex_value.chars());

    let hex_value_len = hex_value.len();

    if hex_value_len == 4 || hex_value_len == 8 {
        return Err(ColorError::IncludesAlpha);
    }

    // TMP
    Ok(Color::Yellow)
}
