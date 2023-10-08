use silicon::utils::ToRgba;
use anyhow::Error;
pub use image::{Rgba as ImageRgba, Pixel};

#[derive(Debug, Clone)]
pub struct Rgba(pub ImageRgba<u8>);

impl Rgba {
    pub fn to_rgba(&self) -> ImageRgba<u8> {
        self.0
    }
}

impl From<ImageRgba<u8>> for Rgba {
    fn from(rgba: ImageRgba<u8>) -> Self {
        Rgba(rgba)
    }
}

impl<'de> serde::Deserialize<'de> for Rgba {
    fn deserialize<D>(deserializer: D) -> Result<Rgba, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_str_color(&s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Rgba {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let channels = self.0.channels();
        let r = channels[0];
        let g = channels[1];
        let b = channels[2];
        let a = channels[3];
        write!(f, "#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }
}

fn parse_str_color(s: &str) -> Result<Rgba, Error> {
    let rgba = s.to_rgba()
        .map_err(|e| Error::msg(format!("Invalid color: {}", e)))?;
    Ok(Rgba(rgba))
}