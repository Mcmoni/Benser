use std::sync::Arc;

use crate::{file_output::Vertex, wgpu_util::get_gpu_instance};
use benser::layout::{layout_tree, Dimensions, LayoutBox, Rect};
use benser::style::StyledNode;
use log::debug;
use lyon::{
    geom::{euclid::Point2D, Box2D},
    lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers},
};
use paint::{build_display_list, DisplayCommand};
use wgpu::{util::DeviceExt, TextureFormat};
use wgpu_text::{
    font::FontArc,
    section::{Section, Text},
    BrushBuilder, TextBrush,
};
use winit::{event::WindowEvent, window::Window};

pub struct State {
    window: Window,
    surface: wgpu::Surface,
    surface_format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    lyon_buffer: VertexBuffers<Vertex, u16>,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub text_brush: TextBrush,
    root_node: Arc<StyledNode>,
}

impl State {
    pub async fn new(window: Window, root_node: Arc<StyledNode>) -> State {
        let size = window.inner_size();
        let instance = get_gpu_instance();

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        debug!("surface capabilities: {surface_caps:?}");
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format.into(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let font = FontArc::try_from_slice(include_bytes!("../fonts/OpenSans.ttf")).unwrap();

        let brush = BrushBuilder::using_font(font).build(
            &device,
            config.width,
            config.height,
            surface_format,
        );

        let lyon_buffer: VertexBuffers<Vertex, u16> = VertexBuffers::new();

        Self {
            window,
            surface,
            surface_format,
            device,
            queue,
            render_pipeline,
            lyon_buffer,
            config,
            window_size: size,
            text_brush: brush,
            root_node,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.text_brush.resize_view(
                self.config.width as f32,
                self.config.height as f32,
                &self.queue,
            )
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    /// Draw a filled rectangle
    fn draw_rectangle(&mut self, rect: Rect, color: [f32; 4]) {
        let min = point(rect.x, rect.y, self.window_size.into());
        let max = point(
            rect.x + rect.width,
            rect.y + rect.height,
            self.window_size.into(),
        );
        let mut fill_tessellator = FillTessellator::new();
        fill_tessellator
            .tessellate_rectangle(
                &Box2D::new(Point2D::from(min), Point2D::from(max)),
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut self.lyon_buffer, |vertex: FillVertex| Vertex {
                    position: [vertex.position().x, vertex.position().y, 0.0],
                    color,
                }),
            )
            .unwrap();
    }

    fn paint(&mut self, viewport: Dimensions) {
        let layout_root = layout_tree(&self.root_node, viewport);

        let display_commands = build_display_list(&layout_root);
        for command in display_commands {
            match command {
                DisplayCommand::SolidColor(color, rect) => self.draw_rectangle(
                    rect,
                    native_color(
                        u32::from_ne_bytes([color.r, color.g, color.b, color.a]),
                        &self.surface_format,
                    ),
                ),
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Some dummy text
        let section = Section::default()
            .add_text(Text::new("Hello World").with_scale(80.0))
            .with_screen_position((300.0, 300.0));

        // Text
        self.text_brush
            .queue(&self.device, &self.queue, vec![&section])
            .unwrap();

        // Lyon
        self.lyon_buffer.indices.clear();
        self.lyon_buffer.vertices.clear();
        // Draw shapes into the lyon_buffer here
        let mut viewport = Dimensions::default();
        viewport.content.height = output_texture.texture.height() as f32;
        viewport.content.width = output_texture.texture.width() as f32;
        self.paint(viewport);

        let vertex_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.lyon_buffer.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.lyon_buffer.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.text_brush.draw(&mut render_pass);

            // Draw lyon elements
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buf.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.lyon_buffer.indices.len() as u32, 0, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(Some(encoder.finish()));
        output_texture.present();

        Ok(())
    }
}

// Translates points from pixel coordinates to wgpu coordinates
pub fn point(x: f32, y: f32, screen: (f32, f32)) -> [f32; 2] {
    let scale_x = 2. / screen.0;
    let scale_y = 2. / screen.1;
    let new_x = -1. + (x * scale_x);
    let new_y = 1. - (y * scale_y);
    [new_x, new_y]
}

fn native_color(c: u32, format: &TextureFormat) -> [f32; 4] {
    use wgpu::TextureFormat::*;
    let f = |xu: u32| (xu & 0xff) as f32 / 255.0;

    match format {
        Rgba8UnormSrgb => hex_to_linear_rgba(c),
        Bgra8UnormSrgb => hex_to_linear_bgra(c),
        _ => [f(c >> 16), f(c >> 8), f(c), 1.0],
    }
}

fn hex_to_linear_rgba(c: u32) -> [f32; 4] {
    let f = |xu: u32| {
        let x = (xu & 0xff) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c >> 16), f(c >> 8), f(c >> 0), 1.0]
}

fn hex_to_linear_bgra(c: u32) -> [f32; 4] {
    let f = |xu: u32| {
        let x = (xu & 0xff) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c >> 0), f(c >> 8), f(c >> 16), 1.0]
}
