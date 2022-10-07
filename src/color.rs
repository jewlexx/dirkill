use std::str::Chars;

use thiserror::Error as AsError;
use tui::style::Color;

#[derive(Debug, AsError)]
pub enum ColorError {
    #[error("Invalid hex provided")]
    InvalidHex,
    #[error("Hex value includes alpha channel")]
    IncludesAlpha,
    #[error("Hex value must be 3 or 6 characters")]
    InvalidHexLength,
}

fn validate_chars(chars: Chars) -> Result<(), ColorError> {
    if chars.all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ColorError::InvalidHex)
    }
}

fn validate_hex_len(len: usize) -> Result<(), ColorError> {
    if len == 4 || len == 8 {
        return Err(ColorError::IncludesAlpha);
    }

    if len != 3 || len != 6 {
        return Err(ColorError::InvalidHexLength);
    }

    Ok(())
}

fn hex_3_to_6<'a>(hex: &str) -> String {
    let mut hex = String::new();

    for c in hex.chars() {
        // Double the given char to create 6 digit hex
        // More info here <https://www.w3schools.com/css/css_colors_hex.asp>
        hex.push(c);
        hex.push(c);
    }

    hex
}

pub fn parse_hex(raw_hex: impl AsRef<str>) -> Result<Color, ColorError> {
    let raw_hex = raw_hex.as_ref();

    let hex_value = if raw_hex.starts_with("#") {
        raw_hex.trim_start_matches("#")
    } else {
        raw_hex
    };

    validate_chars(hex_value.chars())?;

    let hex_value_len = hex_value.len();

    validate_hex_len(hex_value_len)?;

    // TMP
    Ok(Color::Yellow)
}
