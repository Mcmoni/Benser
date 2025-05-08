use crate::args::Args;
use crate::wgpu_util::get_gpu_instance;
use benser::css::Parser as css_parser;
use benser::layout::{layout_tree, Dimensions};
use benser::style::style_tree;
use html::parser::Parser as html_parser;
use image::ImageFormat;
use std::fs;
use std::fs::File;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub async fn run(args: Args) {
    // Read input files
    let css_source = fs::read_to_string(args.css_file).unwrap();
    let html_source = fs::read_to_string(args.html_file).unwrap();

    // Create a virtual viewport
    let mut viewport = Dimensions::default();
    if let Some(viewport_width) = args.viewport_width {
        viewport.content.width = viewport_width;
    } else {
        viewport.content.width = 500.0;
    }
    if let Some(viewport_height) = args.viewport_height {
        viewport.content.height = viewport_height;
    } else {
        viewport.content.height = 256.0;
    }

    // Parsing and rendering:
    let root_node = html_parser::from_string(&html_source).run();
    let stylesheet = css_parser::parse(&css_source);
    let style_root = style_tree(&root_node, &stylesheet);
    let layout_root = layout_tree(&style_root, viewport);

    // Create the output file:
    File::create(&args.output.clone().unwrap()).unwrap();

    // Write to the file
    // let canvas = paint(&layout_root, viewport.content);
    let (texture_width, texture_height) = (
        viewport.content.width as u32,
        viewport.content.height as u32,
    );

    // Set up wgpu
    let instance = get_gpu_instance();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .await
        .unwrap();

    let texture_extent3d = wgpu::Extent3d {
        width: texture_width,
        height: texture_height,
        depth_or_array_layers: 1,
    };

    dbg!(&texture_width);
    dbg!(&texture_height);
    dbg!(&texture_extent3d);

    let texture_desc = wgpu::TextureDescriptor {
        size: texture_extent3d,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    // Create output buffer
    let u32_size = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (u32_size
        * round_up_to_multiple(texture_width, 256)
        * round_up_to_multiple(texture_height, 256))
        as wgpu::BufferAddress;
    dbg!(&output_buffer_size);
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    // Render pipeline

    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                format: texture_desc.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    const VERTICES: &[Vertex] = &[
        Vertex {
            position: [0.0, 0.5, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    {
        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
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
        };
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render_pass.set_pipeline(&render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    let bytes_per_row = u32_size * texture_width;
    let bytes_per_row = round_up_to_multiple(bytes_per_row, 256);

    dbg!(bytes_per_row);

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(round_up_to_multiple(texture_height, 256)),
            },
        },
        texture_desc.size,
    );

    queue.submit(Some(encoder.finish()));

    {
        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
            round_up_to_multiple(texture_width, 256),
            round_up_to_multiple(texture_height, 256),
            data,
        )
        .unwrap();

        buffer
            .save_with_format(&args.output.clone().unwrap(), ImageFormat::Png)
            .unwrap();
    }

    output_buffer.unmap();
}

/// Round up a number to the nearest multiple
fn round_up_to_multiple(number: u32, multiple: u32) -> u32 {
    if multiple == 0 {
        return number;
    }
    let remainder = number % multiple;
    if remainder == 0 {
        return number;
    }
    number + multiple - remainder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounding() {
        assert_eq!(round_up_to_multiple(128, 256), 256);
        assert_eq!(round_up_to_multiple(256, 256), 256);
        assert_eq!(round_up_to_multiple(500, 256), 512);
    }
}
