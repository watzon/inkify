#[macro_use]
extern crate anyhow;

use clap::Parser;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Error;
use lazy_static::lazy_static;
use silicon as si;
use silicon::utils::ToRgba;
use tensorflow::Tensor;
use std::collections::HashSet;
use std::io::Cursor;
use std::num::ParseIntError;
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

mod config;
mod rgba;

lazy_static! {
    static ref HIGHLIGHTING_ASSETS: si::assets::HighlightingAssets =
        silicon::assets::HighlightingAssets::new();
}

macro_rules! unwrap_or_return {
    ( $e:expr, $r:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return $r,
        }
    };
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    tensorflow_model_dir: Option<String>,
}

fn parse_font_str(s: &str) -> Vec<(String, f32)> {
    let mut result = vec![];
    for font in s.split(';') {
        let tmp = font.split('=').collect::<Vec<_>>();
        let font_name = tmp[0].to_owned();
        let font_size = tmp
            .get(1)
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(26.0);
        result.push((font_name, font_size));
    }
    result
}

fn parse_line_range(s: &str) -> Result<Vec<u32>, ParseIntError> {
    let mut result = vec![];
    for range in s.split(';') {
        let range: Vec<u32> = range
            .split('-')
            .map(|s| s.parse::<u32>())
            .collect::<Result<Vec<_>, _>>()?;
        if range.len() == 1 {
            result.push(range[0])
        } else {
            for i in range[0]..=range[1] {
                result.push(i);
            }
        }
    }
    Ok(result)
}

fn parse_str_color(s: &str) -> Result<rgba::Rgba, Error> {
    let res = s
        .to_rgba()
        .map_err(|_| format_err!("Invalid color: `{}`", s));
    Ok(rgba::Rgba(res?))
}

#[get("/")]
async fn help() -> impl Responder {
    // Respond with some help text for how to use the API,
    // formatted as JSON since this is an API.
    let json = r#"
    {
        "message": "Hello, world! Welcome to Inkify, a simple API for generating images from code. Think of it like Carbon in API form.",
        "routes": {
          "GET /": "This help text. Will always return 200, so you can use it to check if the server is up.",
          "GET /themes": "Return a list of available syntax themes.",
          "GET /languages": "Retuns a list of languages which can be parsed.",
          "GET /fonts": "Returns a list of available fonts.",
          "GET /detect": {
            "description": "Detect the language of the given code.",
            "parameters": {
                "code": "The code to detect the language of. Required."
            }
          },
          "GET /generate": {
            "description": "Generate an image from the given code.",
            "parameters": {
                "code": "The code to generate an image from. Required.",
                "language": "The language to use for syntax highlighting. Optional, will attempt to guess if not provided.",
                "theme": "The theme to use for syntax highlighting. Optional, defaults to Dracula.",
                "font": "The font to use. Optional.",
                "shadow_color": "The color of the shadow. Optional, defaults to transparent.",
                "background": "The background color. Optional, defaults to transparent.",
                "tab_width": "The tab width. Optional, defaults to 4.",
                "line_pad": "The line padding. Optional, defaults to 2.",
                "line_offset": "The line offset. Optional, defaults to 1.",
                "window_title": "The window title. Optional, defaults to \"Inkify\".",
                "no_line_number": "Whether to hide the line numbers. Optional, defaults to false.",
                "no_round_corner": "Whether to round the corners. Optional, defaults to false.",
                "no_window_controls": "Whether to hide the window controls. Optional, defaults to false.",
                "shadow_blur_radius": "The shadow blur radius. Optional, defaults to 0.",
                "shadow_offset_x": "The shadow offset x. Optional, defaults to 0.",
                "shadow_offset_y": "The shadow offset y. Optional, defaults to 0.",
                "pad_horiz": "The horizontal padding. Optional, defaults to 80.",
                "pad_vert": "The vertical padding. Optional, defaults to 100.",
                "highlight_lines": "The lines to highlight. Optional, defaults to none.",
                "background_image": "The background image for the padding area as a URL. Optional, defaults to none."
            }
          }
        }
      }
    "#;

    HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(json)
}

#[get("/themes")]
async fn themes() -> impl Responder {
    let ha = &*HIGHLIGHTING_ASSETS;
    let themes = &ha.theme_set.themes;
    let theme_keys: Vec<String> = themes.keys().map(|s| s.to_string()).collect();
    HttpResponse::Ok().json(theme_keys)
}

#[get("/languages")]
async fn languages() -> impl Responder {
    let ha = &*HIGHLIGHTING_ASSETS;
    let syntaxes = &ha.syntax_set.syntaxes();
    let mut languages = syntaxes
        .iter()
        .map(|s| s.name.to_string())
        .collect::<Vec<String>>();
    let unique_languages: HashSet<String> = languages.drain(..).collect();
    let mut unique_languages: Vec<String> = unique_languages.into_iter().collect();
    unique_languages.sort();
    HttpResponse::Ok().json(unique_languages)
}

#[get("/fonts")]
async fn fonts() -> impl Responder {
    let source = font_kit::source::SystemSource::new();
    let fonts = source.all_families().unwrap_or_default();
    HttpResponse::Ok().json(fonts)
}

#[get("/detect")]
async fn detect(info: web::Query<config::ConfigQuery>) -> impl Responder {
    let args = CliArgs::parse();
    let ha = &*HIGHLIGHTING_ASSETS;

    let (ps, _ts) = (&ha.syntax_set, &ha.theme_set);

    let mut conf = config::Config::default();
    conf.code = info.code.clone();
    if conf.code.is_empty() {
        return HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "code parameter is required"}"#);
    }

    if args.tensorflow_model_dir.is_some() {
        conf.load_tensorflow_model(args.tensorflow_model_dir.unwrap().as_str());
    }

    let input_data = Tensor::new(&[1]).with_values(&[conf.code.clone()]).unwrap();
    let predictions = unwrap_or_return!(
        conf.predict_language_with_tensorflow(ps, input_data),
        HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Failed to detect language."}"#)
    );

    let mut sorted_predictions: Vec<_> = predictions.iter().collect();
        sorted_predictions.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    let min_score = predictions.iter().map(|(_, score)| *score).fold(f32::INFINITY, f32::min);
    let max_score = predictions.iter().map(|(_, score)| *score).fold(f32::NEG_INFINITY, f32::max);

    // Normalize scores and pick top 5
    let mut normalized_predictions: Vec<_> = predictions.iter().map(|(lang, score)| {
    let normalized_score = (score - min_score) / (max_score - min_score) * 100.0;
    (lang, normalized_score)
    }).collect();

    normalized_predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let response = normalized_predictions
        .iter()
        // .take(5)
        .map(|(language, score)| format!("{{\"language\": \"{}\", \"score\": {}}}", language, score))
        .collect::<Vec<_>>()
        .join(",");

    HttpResponse::Ok()
        .append_header(("Content-Type", "application/json"))
        .body(format!("[{}]", response))
}

#[get("/generate")]
async fn generate(info: web::Query<config::ConfigQuery>) -> impl Responder {
    let args = CliArgs::parse();
    let ha = &*HIGHLIGHTING_ASSETS;

    let (ps, ts) = (&ha.syntax_set, &ha.theme_set);

    let mut conf = config::Config::default();
    conf.code = info.code.clone();
    if conf.code.is_empty() {
        return HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "code parameter is required"}"#);
    }

    if args.tensorflow_model_dir.is_some() {
        conf.load_tensorflow_model(args.tensorflow_model_dir.unwrap().as_str());
    }

    conf.language = info.language.clone();
    if let Some(theme) = info.theme.clone() {
        conf.theme = theme;
    }
    if let Some(font) = info.font.clone() {
        conf.font = Some(parse_font_str(&font));
    }
    if let Some(shadow_color) = info.shadow_color.clone() {
        conf.shadow_color = parse_str_color(shadow_color.as_str()).unwrap();
    }
    if let Some(background) = info.background.clone() {
        conf.background = parse_str_color(background.as_str()).unwrap();
    }
    if let Some(tab_width) = info.tab_width {
        conf.tab_width = tab_width;
    }
    if let Some(line_pad) = info.line_pad {
        conf.line_pad = line_pad;
    }
    if let Some(line_offset) = info.line_offset {
        conf.line_offset = line_offset;
    }
    if let Some(window_title) = info.window_title.clone() {
        conf.window_title = Some(window_title);
    }
    if let Some(no_line_number) = info.no_line_number {
        conf.no_line_number = no_line_number;
    }
    if let Some(no_round_corner) = info.no_round_corner {
        conf.no_round_corner = no_round_corner;
    }
    if let Some(no_window_controls) = info.no_window_controls {
        conf.no_window_controls = no_window_controls;
    }
    if let Some(shadow_blur_radius) = info.shadow_blur_radius {
        conf.shadow_blur_radius = shadow_blur_radius;
    }
    if let Some(shadow_offset_x) = info.shadow_offset_x {
        conf.shadow_offset_x = shadow_offset_x;
    }
    if let Some(shadow_offset_y) = info.shadow_offset_y {
        conf.shadow_offset_y = shadow_offset_y;
    }
    if let Some(pad_horiz) = info.pad_horiz {
        conf.pad_horiz = pad_horiz;
    }
    if let Some(pad_vert) = info.pad_vert {
        conf.pad_vert = pad_vert;
    }
    if let Some(highlight_lines) = info.highlight_lines.clone() {
        conf.highlight_lines = Some(parse_line_range(highlight_lines.as_str()).unwrap());
    }
    if let Some(background_image) = info.background_image.clone() {
        // If a background image is provided, it will be as a URL. We need
        // to download it and add it to the config as a Vec<u8>.
        let res = reqwest::get(background_image.as_str()).await;
        if let Ok(mut res) = res {
            let mut buf = vec![];
            while let Ok(Some(chunk)) = res.chunk().await {
                buf.extend_from_slice(&chunk);
            }
            conf.background_image = Some(buf);
        }
    }

    let syntax = unwrap_or_return!(
        conf.language(ps),
        HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Unable to determine language, please provide one explicitly"}"#)
    );

    let theme = unwrap_or_return!(
        conf.theme(ts),
        HttpResponse::BadRequest()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Invalid theme"}"#)
    );

    let mut h = HighlightLines::new(syntax, &theme);
    let highlight = unwrap_or_return!(
        LinesWithEndings::from(conf.code.as_ref())
            .map(|line| h.highlight_line(line, ps))
            .collect::<Result<Vec<_>, _>>(),
        HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Failed to highlight code"}"#)
    );

    let mut formatter = unwrap_or_return!(
        conf.get_formatter(),
        HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Failed to get formatter"}"#)
    );

    let image = formatter.format(&highlight, &theme);
    let mut buffer: Vec<u8> = Vec::new();
    unwrap_or_return!(
        image.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png),
        HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json"))
            .body(r#"{"error": "Failed to write image"}"#)
    );

    // Return the image as a PNG.
    HttpResponse::Ok()
        .append_header(("Content-Type", "image/png"))
        .body(buffer)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
    let server = HttpServer::new(|| {
        App::new()
            .service(help)
            .service(themes)
            .service(languages)
            .service(fonts)
            .service(detect)
            .service(generate)
    })
    .bind((host.clone(), port.parse::<u16>().unwrap()))?
    .run();

    println!("Inkify listening on {}:{}", host, port);
    println!("Visit http://{}:{}/ to get started.", host, port);
    server.await
}
