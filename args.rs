use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// CSS file to use
    pub css_file: PathBuf,

    /// HTML file to use
    pub html_file: PathBuf,

    /// File to output to
    pub output: Option<PathBuf>,

    /// Viewport width
    #[arg(long = "width")]
    pub viewport_width: Option<f32>,

    /// Viewport height
    #[arg(long = "height")]
    pub viewport_height: Option<f32>,
}
