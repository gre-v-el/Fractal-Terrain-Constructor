use eframe::egui_wgpu::wgpu;
use eframe::egui;
use wgpu::util::DeviceExt;

use crate::{vertex::{self, Vertex}, camera, view, mesh_operation::wireframe_indices};

pub struct Viewport {
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
    pub triangle_pipeline: wgpu::RenderPipeline,
    pub wireframe_pipeline: wgpu::RenderPipeline,
	pub indices: u32,
	pub camera: camera::OrbitCamera,
	pub view_uniform: view::ViewUniform,
	pub uniform_buffer: wgpu::Buffer,
	pub uniform_bind_group: wgpu::BindGroup,
	pub depth_texture: wgpu::Texture,
	pub depth_texture_view: wgpu::TextureView,

	pub mesh: (Vec<Vertex>, Vec<u32>, Option<Vec<u32>>),
	pub should_set: bool,
}

impl Viewport {
	pub fn new(render_state: &egui_wgpu::RenderState, size: egui::emath::Vec2) -> Self {
		let device = &render_state.device;

		// let (vertices, indices) = terrain_gen::generate();
		let vertices = Vec::<Vertex>::new();
		let indices = Vec::<u32>::new();

		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(vertices.as_slice()),
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(indices.as_slice()),
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
		});

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
		});

		
		let camera = camera::OrbitCamera {
			origin: cgmath::Point3 { x: 0.0, y: 0.0, z: 0.0 },
			radius: 10.0,
			y_rot: 20.0,
			x_rot: 40.0,
			aspect: size.x / size.y,
		};

		let view_uniform = view::ViewUniform::new(camera.get_uniform(), 1, 0.7, 0.1);

		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&[view_uniform]),
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
		});

		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer { 
						ty: wgpu::BufferBindingType::Uniform, 
						has_dynamic_offset: false, 
						min_binding_size: None,
					},
					count: None,
				}
			]
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: uniform_buffer.as_entire_binding(),
			}],
		});
		
		let depth_texture = Self::create_depth_texture(&device, size.x as u32, size.y as u32);
		let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default()); 

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[
				&uniform_bind_group_layout,
			],
			push_constant_ranges: &[],
		});

		let pipeline_bindings = [Some(wgpu::ColorTargetState { 
			format: render_state.target_format, 
			blend: Some(wgpu::BlendState::REPLACE), 
			write_mask: wgpu::ColorWrites::all(),
		})];
		let mut pipeline_desc = wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState { 
				module: &shader, 
				entry_point: "vs_main", 
				buffers: &[vertex::Vertex::buffer_layout()] 
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &pipeline_bindings,
			}),
			primitive: wgpu::PrimitiveState { 
				topology: wgpu::PrimitiveTopology::TriangleList, 
				strip_index_format: None, 
				front_face: wgpu::FrontFace::Ccw, 
				cull_mode: None, 
				unclipped_depth: false, 
				polygon_mode: wgpu::PolygonMode::Fill, 
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				bias: wgpu::DepthBiasState::default(),
				stencil: wgpu::StencilState::default(),
			}),
			// depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		};
		let triangle_pipeline = device.create_render_pipeline(&pipeline_desc);
		pipeline_desc.primitive.topology = wgpu::PrimitiveTopology::LineList;
		let wireframe_pipeline = device.create_render_pipeline(&pipeline_desc);

		Self { 
			vertex_buffer,
			index_buffer,
			triangle_pipeline, 
			wireframe_pipeline, 
			indices: indices.len() as u32,
			camera,
			view_uniform,
			uniform_buffer,
			uniform_bind_group,
			depth_texture,
			depth_texture_view,
			mesh: (Vec::new(), Vec::new(), None),
			should_set: false,
		}
	}
	
	fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
		device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d {
				width: width,
				height: height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Depth32Float,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		})
	}

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, response: &egui::Response, scroll: egui::emath::Vec2, resize: Option<egui::emath::Vec2>) {

		// resize
		if let Option::Some(size) = resize {
			self.depth_texture = Self::create_depth_texture(device, size.x as u32, size.y as u32);
			self.depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
			self.camera.aspect = size.x / size.y;
		}

		// set new mesh
		if self.should_set { // todo: wireframe is calculated every time, despite the fact that it is stored, not ideal
			self.should_set = false;

			self.index_buffer.destroy();
			self.vertex_buffer.destroy();

			self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: None,
				contents: bytemuck::cast_slice(self.mesh.0.as_slice()),
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			});
	
			let indices = 
				if self.view_uniform.display == 0 
				{
					if let Some(w) = &self.mesh.2 {
						w.as_slice()
					}
					else {
						let w = wireframe_indices(self.mesh.1.as_slice());
						self.mesh.2 = Some(w);
						
						if let Some(w) = &self.mesh.2 {
							w.as_slice()
						}
						else {
							&[]
						}
					}
				} 
				else {
					self.mesh.1.as_slice()
				};
			self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: None,
				contents: bytemuck::cast_slice(indices),
				usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
			});

			self.indices = self.mesh.1.len() as u32;
		}

		// move camera
		self.camera.update(response, scroll);
		
		self.view_uniform.camera = self.camera.get_uniform();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.view_uniform]));
    }

    pub fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
		render_pass.set_pipeline(if self.view_uniform.display == 0 {&self.wireframe_pipeline} else {&self.triangle_pipeline});
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}