//! Textual representation of the colors handled by the game rules.
//!
//! `parse_color` is the exact mapping the CLI has always used; `color_name`
//! is its inverse so a board can be rendered back to JSON.

use std::fmt;

use crate::pipe::Color;

/// A color name that does not belong to the game palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorParseError(pub String);

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Couleur inconnue : '{}'.", self.0)
    }
}

impl std::error::Error for ColorParseError {}

pub fn parse_color(color: &str) -> Result<Color, ColorParseError> {
    match color.to_lowercase().as_str() {
        "grey" | "gray" => Ok(Color::Grey),
        "blue" => Ok(Color::Blue),
        "lemon" => Ok(Color::Lemon),
        "brown" => Ok(Color::Brown),
        "green" => Ok(Color::Green),
        "red" => Ok(Color::Red),
        "lightgreen" | "light_green" => Ok(Color::LightGreen),
        "lightblue" | "light_blue" => Ok(Color::LightBlue),
        "pink" => Ok(Color::Pink),
        "orange" => Ok(Color::Orange),
        "purple" => Ok(Color::Purple),
        "yellow" => Ok(Color::Yellow),
        other => Err(ColorParseError(other.to_string())),
    }
}

/// Canonical name of a color, accepted back by [`parse_color`].
pub fn color_name(color: &Color) -> &'static str {
    match color {
        Color::Grey => "Grey",
        Color::Blue => "Blue",
        Color::Lemon => "Lemon",
        Color::Brown => "Brown",
        Color::Green => "Green",
        Color::Red => "Red",
        Color::LightGreen => "LightGreen",
        Color::LightBlue => "LightBlue",
        Color::Pink => "Pink",
        Color::Orange => "Orange",
        Color::Purple => "Purple",
        Color::Yellow => "Yellow",
    }
}

/// Every color of the palette, in a stable order.
pub fn palette() -> [Color; 12] {
    [
        Color::Grey,
        Color::Blue,
        Color::Lemon,
        Color::Brown,
        Color::Green,
        Color::Red,
        Color::LightGreen,
        Color::LightBlue,
        Color::Pink,
        Color::Orange,
        Color::Purple,
        Color::Yellow,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_colors_case_insensitively() {
        assert_eq!(parse_color("GREY").unwrap(), Color::Grey);
        assert_eq!(parse_color("gray").unwrap(), Color::Grey);
        assert_eq!(parse_color("light_blue").unwrap(), Color::LightBlue);
    }

    #[test]
    fn rejects_unknown_colors() {
        assert!(parse_color("chartreuse").is_err());
    }

    #[test]
    fn every_color_name_round_trips() {
        for color in palette() {
            assert_eq!(parse_color(color_name(&color)).unwrap(), color);
        }
    }
}
