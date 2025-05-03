use ratatui::style::Color;

fn validate_chars(string: impl AsRef<str>) -> anyhow::Result<()> {
    let mut chars = string.as_ref().chars();

    if chars.all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        anyhow::bail!("Invalid hex")
    }
}

fn validate_hex_len(len: usize) -> anyhow::Result<()> {
    if len == 4 || len == 8 {
        anyhow::bail!("Alpha is included in hex");
    }

    if len != 3 && len != 6 {
        anyhow::bail!("Invalid hex length");
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

pub fn parse_hex(raw_hex: impl AsRef<str>) -> anyhow::Result<Color> {
    let raw_hex = raw_hex.as_ref();

    let hex_value = raw_hex.trim_start_matches('#');

    validate_chars(hex_value)?;

    let hex_value_len = hex_value.len();

    validate_hex_len(hex_value_len)?;

    let hex_valid = if hex_value_len == 3 {
        hex_3_to_6(hex_value)
    } else {
        hex_value.to_string()
    };

    let hex_u32 = u32::from_str_radix(&hex_valid, 16)?;

    #[allow(clippy::cast_possible_truncation)]
    let r = (hex_u32 >> 16) as u8;
    let g = ((hex_u32 >> 8) & 0x00FF) as u8;
    let b = (hex_u32 & 0x0000_00FF) as u8;

    // TMP
    Ok(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("#fff").unwrap(), Color::Rgb(255, 255, 255));
        assert_eq!(parse_hex("#000").unwrap(), Color::Rgb(0, 0, 0));
        assert_eq!(parse_hex("#c19c00").unwrap(), Color::Rgb(193, 156, 0));
    }

    #[rstest]
    #[case("fff", "fff")]
    #[case("#fff", "fff")]
    fn test_trim_start_matches(#[case] raw_hex: &str, #[case] expected: &str) {
        assert_eq!(raw_hex.trim_start_matches('#'), expected);
    }
}
