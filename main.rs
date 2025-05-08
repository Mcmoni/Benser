use benser::css::Parser as css_parser;
use benser::style::style_tree;
use clap::Parser;
use html::parser::Parser as html_parser;
use std::fs;
use std::sync::Arc;
use wgpu_renderer::args::Args;
use wgpu_renderer::{browser, file_output};

fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Some(ref _path) = args.output {
        pollster::block_on(file_output::run(args))
    } else {
        let html_source = fs::read_to_string(&args.html_file).unwrap();
        let css_source = fs::read_to_string(&args.css_file).unwrap();
        let root_node = html_parser::from_string(&html_source).run();
        let stylesheet = css_parser::parse(&css_source);
        let style_root = style_tree(&root_node, &stylesheet);

        pollster::block_on(browser::run(Arc::new(style_root)));
    }
}
