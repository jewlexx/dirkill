use std::{num::ParseIntError, str::Chars};

use ratatui::style::Color;
use thiserror::Error as AsError;

#[derive(Debug, AsError, PartialEq, Eq)]
pub enum ColorError {
    #[error("Invalid hex provided")]
    InvalidHex,
    #[error("Hex value includes alpha channel")]
    IncludesAlpha,
    #[error("Hex value must be 3 or 6 characters")]
    InvalidHexLength,
    #[error("Failed to parse hex value")]
    ParseError(#[from] ParseIntError),
}

fn validate_chars(mut chars: Chars) -> Result<(), ColorError> {
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

    if len != 3 && len != 6 {
        return Err(ColorError::InvalidHexLength);
    }

    Ok(())
}

fn hex_3_to_6(hex_trois: &str) -> String {
    let mut hex = String::new();

    for c in hex_trois.chars() {
        // Double the given char to create 6 digit hex
        // More info here <https://www.w3schools.com/css/css_colors_hex.asp>
        hex.push(c);
        hex.push(c);
    }

    hex
}

pub fn parse_hex(raw_hex: impl AsRef<str>) -> Result<Color, ColorError> {
    let raw_hex = raw_hex.as_ref();

    let hex_value = if raw_hex.starts_with('#') {
        raw_hex.trim_start_matches('#')
    } else {
        raw_hex
    };

    validate_chars(hex_value.chars())?;

    let hex_value_len = hex_value.len();

    validate_hex_len(hex_value_len)?;

    let hex_valid = if hex_value_len == 3 {
        hex_3_to_6(hex_value)
    } else {
        hex_value.to_string()
    };

    let hex_usize = usize::from_str_radix(&hex_valid, 16)?;

    let r = (hex_usize >> 16) as u8;
    let g = ((hex_usize >> 8) & 0x00FF) as u8;
    let b = (hex_usize & 0x0000_00FF) as u8;

    // TMP
    Ok(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("#fff"), Ok(Color::Rgb(255, 255, 255)));
        assert_eq!(parse_hex("#000"), Ok(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_hex("#c19c00"), Ok(Color::Rgb(193, 156, 0)));
    }
}
