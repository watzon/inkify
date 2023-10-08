# Inkify

Unfortunately [Carbon](https://carbon.now.sh) has been without an API for too long, and I've run into a few cases where one would be useful for a project. So I present to you, Inkify, an API for generating beautiful pictures of your code.

## Usage

Inkify relies on the [silicon](https://github.com/Aloxaf/silicon) library for generating photos, and takes much the same arguments as the silicon CLI does. Arguments are passed as query parameters to the `/generate` route, and are as follows:

- code: The code to generate an image from. Required.
- language: The language to use for syntax highlighting. Optional, will attempt to guess if not provided.
- theme: The theme to use for syntax highlighting. Optional, defaults to Dracula.
- font: The font to use. Optional, defaults to Fira Code.
- shadow_color: The color of the shadow. Optional, defaults to transparent.
- background: The background color. Optional, defaults to transparent.
- tab_width: The tab width. Optional, defaults to 4.
- line_pad: The line padding. Optional, defaults to 2.
- line_offset: The line offset. Optional, defaults to 1.
- window_title: The window title. Optional, defaults to \"Inkify\".
- no_line_number: Whether to hide the line numbers. Optional, defaults to false.
- no_round_corner: Whether to round the corners. Optional, defaults to false.
- no_window_controls: Whether to hide the window controls. Optional, defaults to false.
- shadow_blur_radius: The shadow blur radius. Optional, defaults to 0.
- shadow_offset_x: The shadow offset x. Optional, defaults to 0.
- shadow_offset_y: The shadow offset y. Optional, defaults to 0.
- pad_horiz: The horizontal padding. Optional, defaults to 80.
- pad_vert: The vertical padding. Optional, defaults to 100.
- highlight_lines: The lines to highlight. Optional, defaults to none.
- background_image: The background image for the padding area as a URL. Optional, defaults to none.

### Routes

#### `GET /`

The index route is used as a help/ping route. It will always return a 200 response if the API is live, and the body is a JSON object containing a message and a list of routes.

#### `GET /generate`

The generate route is used to generate images. It takes the arguments listed above as query parameters, and returns a PNG image.

#### `GET /themes`

The themes route is used to get a list of available themes. It takes no arguments, and returns a JSON object containing a list of themes.

#### `GET /fonts`

The fonts route is used to get a list of available fonts. It takes no arguments, and returns a JSON object containing a list of fonts.

#### `GET /languages`

The languages route is used to get a list of available languages. It takes no arguments, and returns a JSON object containing a list of languages supported by the [syntect](https://github.com/trishume/syntect) library (which is used by silicon under the hood).

## Deployment

Inkify is written in Rust using the [actix-web](https://actix.rs) framework, and can be deployed as a standalone binary. It can also be deployed as a Docker container, and a Dockerfile is provided for this purpose. The Dockerfile also installs all nerd fonts by default, allowing you to use any of them as the font for your code.

## Contributing

Contributions are welcome, and can be made by opening a pull request. Please make sure to lint your code using `cargo clippy` before submitting a pull request.

## License

Inkify is licensed under the MIT license. See the [LICENSE](LICENSE) file for more information.
