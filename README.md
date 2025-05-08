# Benser

Benser is a lightweight browser engine written in Rust. It parses HTML and CSS, computes layouts, and renders content using GPU acceleration via `wgpu`. The project is modular, with separate crates for handling HTML parsing, CSS styling, layout computation, and rendering.

## Features

- **HTML Parsing**: Implements a tokenizer and parser based on the HTML5 specification.
- **CSS Parsing and Styling**: Parses CSS stylesheets and applies styles to the DOM tree.
- **Layout Engine**: Computes box dimensions and layouts based on CSS rules.
- **GPU Rendering**: Uses `wgpu` for rendering content to the screen or output files.
- **Text Rendering**: Supports text rendering with `wgpu-text`.
- **Modular Design**: Organized into multiple crates for better maintainability.

## Project Structure

```
benser-main/
├── benser/          # Core library for CSS and layout computation
├── html/            # HTML parsing and DOM manipulation
├── paint/           # Display list generation for rendering
├── wgpu-renderer/   # GPU-based rendering engine
└── .github/         # CI workflows
```

### Crates

- **`benser`**: Handles CSS parsing, style computation, and layout tree generation.
- **`html`**: Provides HTML parsing and DOM tree construction.
- **`paint`**: Converts layout trees into display lists for rendering.
- **`wgpu-renderer`**: Renders the display list using `wgpu`.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)

### Building the Project

Clone the repository and build the workspace:

```sh
git clone <repository-url>
cd benser-main
cargo build --release
```

### Running the Renderer

To render an HTML file with a CSS stylesheet:

```sh
cargo run --package wgpu-renderer -- --html-file rainbow.html --css-file rainbow.css --output output.png
```

This will generate a PNG file (`output.png`) with the rendered content.

### Running Tests

Run the tests for all crates:

```sh
cargo test
```

## Examples

The `wgpu-renderer/examples` directory contains sample HTML and CSS files. For example, the `rainbow` example demonstrates nested colored blocks.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
```
