use anyhow::Error;
use silicon::formatter::{ImageFormatter, ImageFormatterBuilder};
use silicon::utils::{Background, ShadowAdder};
use std::io::Write;
use std::path::PathBuf;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

use crate::rgba::{ImageRgba, Rgba};

type FontList = Vec<(String, f32)>;
type Lines = Vec<u32>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    /// Background image URL
    pub background_image: Option<Vec<u8>>,

    /// Background color of the image
    pub background: Rgba,

    /// The code to highlight.
    pub code: String,

    /// The fallback font list. eg. 'Hack; SimSun=31'
    pub font: Option<FontList>,

    /// Lines to high light. rg. '1-3; 4'
    pub highlight_lines: Option<Lines>,

    /// The language for syntax highlighting. You can use full name ("Rust") or file extension ("rs").
    pub language: Option<String>,

    /// Pad between lines
    pub line_pad: u32,

    /// Line number offset
    pub line_offset: u32,

    /// Hide the window controls.
    pub no_window_controls: bool,

    /// Show window title
    pub window_title: Option<String>,

    /// Hide the line number.
    pub no_line_number: bool,

    /// Don't round the corner
    pub no_round_corner: bool,

    /// Pad horiz
    pub pad_horiz: u32,

    /// Pad vert
    pub pad_vert: u32,

    /// Color of shadow
    pub shadow_color: Rgba,

    /// Blur radius of the shadow. (set it to 0 to hide shadow)
    pub shadow_blur_radius: f32,

    /// Shadow's offset in Y axis
    pub shadow_offset_y: i32,

    /// Shadow's offset in X axis
    pub shadow_offset_x: i32,

    /// Tab width
    pub tab_width: u8,

    /// The syntax highlight theme. It can be a theme name or path to a .tmTheme file.
    pub theme: String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            background_image: None,
            background: Rgba(ImageRgba([0, 0, 0, 0])),
            code: "".to_owned(),
            font: None,
            highlight_lines: None,
            language: None,
            line_pad: 2,
            line_offset: 1,
            no_window_controls: false,
            window_title: None,
            no_line_number: false,
            no_round_corner: false,
            pad_horiz: 80,
            pad_vert: 100,
            shadow_color: Rgba(ImageRgba([0, 0, 0, 0])),
            shadow_blur_radius: 0.0,
            shadow_offset_y: 0,
            shadow_offset_x: 0,
            tab_width: 4,
            theme: "Dracula".to_owned(),
        }
    }

    pub fn language<'a>(&self, ps: &'a SyntaxSet) -> Result<&'a SyntaxReference, Error> {
        let language = match &self.language {
            Some(language) => ps
                .find_syntax_by_token(language)
                .ok_or_else(|| Error::msg(format!("Invalid language: {}", language)))?,
            None => {
                let first_line = self.code.lines().next().unwrap_or_default();
                ps.find_syntax_by_first_line(first_line).unwrap_or_else(|| {
                    // hyperpolyglot requires a file, so we need to create a temp file
                    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                    write!(temp_file, "{}", self.code).unwrap();
                    let language = hyperpolyglot::detect(temp_file.path()).unwrap();
                    match language {
                        Some(language) => ps.find_syntax_by_token(language.language()).unwrap(),
                        None => ps.find_syntax_by_token("log").unwrap(),
                    }
                })
            },
        };

        Ok(language)
    }

    pub fn theme(&self, ts: &ThemeSet) -> Result<Theme, Error> {
        if let Some(theme) = ts.themes.get(&self.theme) {
            Ok(theme.clone())
        } else {
            ThemeSet::get_theme(PathBuf::from(&self.theme))
                .map_err(|e| Error::msg(format!("Invalid theme: {}", e)))
        }
    }

    pub fn get_formatter(&self) -> Result<ImageFormatter, Error> {
        let formatter = ImageFormatterBuilder::new()
            .line_pad(self.line_pad)
            .window_controls(!self.no_window_controls)
            .window_title(self.window_title.clone())
            .line_number(!self.no_line_number)
            .font(self.font.clone().unwrap_or_default())
            .round_corner(!self.no_round_corner)
            .shadow_adder(self.get_shadow_adder()?)
            .tab_width(self.tab_width)
            .highlight_lines(self.highlight_lines.clone().unwrap_or_default())
            .line_offset(self.line_offset);

        Ok(formatter.build()?)
    }

    pub fn get_shadow_adder(&self) -> Result<ShadowAdder, Error> {
        Ok(ShadowAdder::new()
            .background(match &self.background_image {
                Some(path) => Background::Image(image::load_from_memory(path)?.to_rgba8()),
                None => Background::Solid(self.background.to_rgba()),
            })
            .shadow_color(self.shadow_color.to_rgba())
            .blur_radius(self.shadow_blur_radius)
            .pad_horiz(self.pad_horiz)
            .pad_vert(self.pad_vert)
            .offset_x(self.shadow_offset_x)
            .offset_y(self.shadow_offset_y))
    }
}

/// Query parameters for the /generate endpoint, using Option to make all options
/// with defaults optional.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConfigQuery {
    /// Background image URL
    pub background_image: Option<String>,

    /// Background color of the image
    pub background: Option<String>,

    /// The code to highlight.
    pub code: String,

    /// The fallback font list. eg. 'Hack; SimSun=31'
    pub font: Option<String>,

    /// Lines to high light. rg. '1-3; 4'
    pub highlight_lines: Option<String>,

    /// The language for syntax highlighting. You can use full name ("Rust") or file extension ("rs").
    pub language: Option<String>,

    /// Pad between lines
    pub line_pad: Option<u32>,

    /// Line number offset
    pub line_offset: Option<u32>,

    /// Hide the window controls.
    pub no_window_controls: Option<bool>,

    /// Show window title
    pub window_title: Option<String>,

    /// Hide the line number.
    pub no_line_number: Option<bool>,

    /// Don't round the corner
    pub no_round_corner: Option<bool>,

    /// Pad horiz
    pub pad_horiz: Option<u32>,

    /// Pad vert
    pub pad_vert: Option<u32>,

    /// Color of shadow
    pub shadow_color: Option<String>,

    /// Blur radius of the shadow. (set it to 0 to hide shadow)
    pub shadow_blur_radius: Option<f32>,

    /// Shadow's offset in Y axis
    pub shadow_offset_y: Option<i32>,

    /// Shadow's offset in X axis
    pub shadow_offset_x: Option<i32>,

    /// Tab width
    pub tab_width: Option<u8>,

    /// The syntax highlight theme. It can be a theme name or path to a .tmTheme file.
    pub theme: Option<String>,
}
