use std::{sync::Arc, time::Instant, ops::Range, fs::File};
use cgmath::{Vector4, InnerSpace};
use eframe::egui::{self, DragValue};
use rand::SeedableRng;

use crate::{viewport::Viewport, mesh_operation::{self, MeshOperation, ShowResponse}, vertex::Vertex};

pub struct App {
	viewport_size: egui::emath::Vec2,
	operations: Vec<(f32, MeshOperation)>,
	mesh: (Vec<Vertex>, Vec<u32>),
	should_set: bool,
	seed: i64,
	last_seed: i64,
	display: u32,
	material_threshold: f32,
	material_smoothness: f32,
	normals_calculation_time: f32,
}

impl App {
	pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Option<Self> {
		let viewport_size = egui::emath::vec2(1000.0, 600.0);
		let wgpu_render_state = cc.wgpu_render_state.as_ref()?;
		
		wgpu_render_state
			.renderer
			.write()
			.paint_callback_resources
			.insert(Viewport::new(wgpu_render_state, viewport_size));
		
		let operations = vec![
			// (0.0, MeshOperation::AddTriangleGrid(10.0, 10)),
			// (0.0, MeshOperation::DisplaceRandom(2.0, [false, true, false])),
			// (0.0, MeshOperation::Subdivide(2)),
			// (0.0, MeshOperation::DisplaceRandom(0.05, [true; 3])),
			// (0.0, MeshOperation::Smooth(0.4, 1)),

			(0.0, MeshOperation::AddTriangle(5.0)),
			// (0.0, MeshOperation::FractalTerrain(6, 2.0, 2.0)),

			// (0.0, MeshOperation::AddTriangleGrid(10.0, 4)),
			// (0.0, MeshOperation::FractalTerrain(5, 1.53, 2.0)),
			// (0.0, MeshOperation::DisplaceRandom(0.2, [true; 3])),
			// (0.0, MeshOperation::Smooth(1.0, 2)),
			// (0.0, MeshOperation::FractalTerrain(2, 0.13, 2.0)),
			// (0.0, MeshOperation::Smooth(1.0, 2)),
		];
		


		Some(
			Self {
				viewport_size,
				operations,
				mesh: (Vec::new(), Vec::new()),
				should_set: false,
				seed: -1,
				last_seed: -1,
				display: 2,
				material_threshold: 0.7,
				material_smoothness: 0.1,
				normals_calculation_time: 0.0,
			}
		)
	}

	fn viewport_widget(&mut self, ui: &mut egui::Ui) {
		let new_size = egui::emath::vec2(ui.available_width(), ui.available_height());
		egui::Frame::canvas(ui.style()).show(ui, |ui| {
			
			let (rect, response) =
				ui.allocate_exact_size(self.viewport_size, egui::Sense {
					click: true,
					drag: true,
					focusable: true,
				});
			
			let scroll = ui.input().scroll_delta;
			let resize = {
				if new_size != self.viewport_size && new_size.x >= 1.0 && new_size.y >= 1.0 {
					self.viewport_size = new_size;
					Some(new_size)
				}
				else {
					None
				}
			};

			let mesh = 
			if self.should_set {
				self.should_set = false;
				Some(self.mesh.clone())
			}
			else {
				None
			};
			let display = self.display;
			let material_threshold =  self.material_threshold;
			let material_smoothness = self.material_smoothness;

			let cb = egui_wgpu::CallbackFn::new()
			.prepare(move |device, queue, _encoder, paint_callback_resources| {
				let viewport: &mut Viewport = paint_callback_resources.get_mut().unwrap();
				if let Some(m) = &mesh {
					viewport.mesh = (m.0.clone(), m.1.clone(), None);
					viewport.should_set = true;
				}
				viewport.view_uniform.display = display;
				viewport.view_uniform.material_threshold =  material_threshold;
				viewport.view_uniform.material_smoothness = material_smoothness;
				viewport.prepare(device, queue, &response, scroll, resize);
				Vec::new()
			})
			.paint(move |_info, render_pass, paint_callback_resources| {
				let viewport: &Viewport = paint_callback_resources.get().unwrap();
				viewport.paint(render_pass);
			});
			
			let callback = egui::PaintCallback {
				rect,
				callback: Arc::new(cb),
			};
			
			ui.painter().add(callback);
		});
	}

	fn generate_mesh(&mut self, range: Range<usize>) -> (Vec<Vertex>, Vec<u32>) {
		let operations: &mut [(f32, MeshOperation)] = &mut self.operations[range];
		let mut verts = Vec::<Vertex>::new();
		let mut inds = Vec::<u32>::new();

		let seed = 
			if self.seed >= 0 {
				self.seed as u64
			} 
			else {
				rand::random::<u32>() as u64
			};
		self.last_seed = seed as i64;

		let mut random = rand::rngs::StdRng::seed_from_u64(seed);

		let mut start = Instant::now();
		for op in operations.iter_mut() {
			(verts, inds) = op.1.execute(verts, inds, &mut random);
			op.0 = start.elapsed().as_secs_f32();
			start = Instant::now();
		}

		let normals_start = Instant::now();
		(verts, inds) = Self::calculate_normals(verts, inds);
		self.normals_calculation_time = normals_start.elapsed().as_secs_f32();

		if self.display == 0 {
			
		}

		(verts, inds)
	}

	fn calculate_normals(verts: Vec<Vertex>, inds: Vec<u32>) -> (Vec<Vertex>, Vec<u32>) {
		let mut verts: Vec<Vertex> = verts.iter().map(|v| Vertex {pos: v.pos, normal: [0.0, 0.0, 0.0, 0.0]}).collect();

		for i in 0..(inds.len() / 3) {
			let i = [
				inds[3*i+0] as usize,
				inds[3*i+1] as usize,
				inds[3*i+2] as usize,
			];
			//    1
			//  /   \
			// 2-----0
			const IDS: [[usize; 3]; 3] = [
				[0, 1, 2],
				[1, 2, 0],
				[2, 0, 1],
			];

			for [id1, id2, id3] in IDS {
				let (i1, i2, i3) = (i[id1], i[id2], i[id3]);

				let disp1 = Vector4::from(verts[i2].pos).truncate() - Vector4::from(verts[i1].pos).truncate();
				let disp2 = Vector4::from(verts[i3].pos).truncate() - Vector4::from(verts[i1].pos).truncate();
				let normal: [f32; 4] = disp1.cross(disp2).normalize().extend(1.0).into();
				verts[i1].normal[0] += normal[0];
				verts[i1].normal[1] += normal[1];
				verts[i1].normal[2] += normal[2];
				verts[i1].normal[3] += normal[3];
			}
		}

		(verts.iter_mut().map(|v| {
			v.normal[0] /= v.normal[3];
			v.normal[1] /= v.normal[3];
			v.normal[2] /= v.normal[3];

			*v
		}).collect(), inds)
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		
		egui::SidePanel::left("left_panel")
		.show(ctx, |ui| {

			ui.add_space(10.0);

			ui.label("seed:");
			ui.add(DragValue::new(&mut self.seed).clamp_range(-1..=i64::MAX));
			ui.horizontal(|ui| {
				if ui.button("retrieve").clicked() {
					self.seed = self.last_seed;
				}
				if ui.button("reset").clicked() {
					self.seed = -1;
				}
			});

			ui.add_space(20.0);
			ui.separator();

			let mut responses: Vec<ShowResponse> = Vec::new();
			egui::ScrollArea::vertical().max_height(ui.available_height() - 130.0).show(ui, |ui| {
				responses = self.operations.iter_mut().map(|op| {
					let ret = op.1.show(ui, op.0);
					ui.separator();
					ret
				}).collect();
			});

			ui.separator();
			
			for (i, r) in responses.iter().enumerate() {
				match r {
					ShowResponse::Delete => {
						self.operations.remove(i);
						break;
					},
					ShowResponse::MoveDown => {
						if i + 1 == self.operations.len() { break; }
						self.operations.swap(i, i+1);
						break;
					},
					ShowResponse::MoveUp => {
						if i == 0 { break; }
						self.operations.swap(i, i-1);
						break;
					},
					ShowResponse::GenerateUpToThis => {
						self.mesh = self.generate_mesh(0..(i+1));
						self.should_set = true;
						for o in i+1..self.operations.len() {
							self.operations[o].0 = 0.0;
						}
						break;
					},
					ShowResponse::None => {}
				};
			}

			ui.add_space(10.0);

			if self.normals_calculation_time > 0.0 {
				ui.label(format!("normals calculation time: {:.2}s", self.normals_calculation_time));
			}
			else {
				ui.label("");
			}

			ui.add_space(20.0);

			ui.vertical_centered(|ui| {
				let mut selection: Option<mesh_operation::MeshOperation> = None;

				egui::ComboBox::from_id_source("combobox").selected_text("Add Operation").show_ui(ui, |ui| {

					for def in MeshOperation::DEFAULTS {
						ui.selectable_value(&mut selection, Some(def), def.caption());
					}
				});

				if let Some(operation) = selection {
					self.operations.push((0.0, operation));
				}

				if ui.button("build").clicked() {
					self.mesh = self.generate_mesh(0..self.operations.len());
					self.should_set = true;
				}

				
				if ui.button("export obj").clicked() {
					use std::io::prelude::*;
					
					let mut file = File::create(format!("{}.obj", self.last_seed)).unwrap();

					file.write("# vertices\n".as_bytes()).unwrap();
					for v in &self.mesh.0 {
						file.write(format!("v {} {} {}\n", v.pos[0], v.pos[1], v.pos[2]).as_bytes()).unwrap();
					}
					file.write("\n# normals\n".as_bytes()).unwrap();
					for v in &self.mesh.0 {
						file.write(format!("vn {} {} {}\n", v.normal[0], v.normal[1], v.normal[2]).as_bytes()).unwrap();
					}

					file.write("\n# faces\n".as_bytes()).unwrap();
					for i in 0..self.mesh.1.len()/3 {
						let (i1, i2, i3) = (self.mesh.1[3*i+0]+1, self.mesh.1[3*i+1]+1, self.mesh.1[3*i+2]+1);
						file.write(format!("f {i1}//{i1} {i2}//{i2} {i3}//{i3}\n").as_bytes()).unwrap();
					}

				}
			});			
		});

		egui::TopBottomPanel::bottom("bottom_panel")
		.show(ctx, |ui| {
			ui.add_space(20.0);
			ui.horizontal(|ui| {
				let prev_display = self.display;
				ui.radio_value(&mut self.display, 0, "wireframe");
				ui.radio_value(&mut self.display, 1, "flat");
				ui.radio_value(&mut self.display, 2, "smooth");

				if self.display != prev_display && prev_display * self.display == 0 {
					self.should_set = true;
				}

				
				ui.add_space(40.0);

				ui.vertical(|ui| {
					ui.label("Material threshold:");
					ui.add(egui::Slider::new(&mut self.material_threshold, 0.0..=1.0));
				});
				ui.vertical(|ui| {
					ui.label("Material smoothness:");
					ui.add(egui::Slider::new(&mut self.material_smoothness, 0.01..=1.0));
				});
			});
			ui.add_space(20.0);
		});

		egui::CentralPanel::default()
		.frame(egui::Frame{
			inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
			outer_margin: egui::style::Margin::symmetric(0.0, 0.0),
			rounding: egui::Rounding::none(),
			shadow: eframe::epaint::Shadow::NONE,
			stroke: eframe::epaint::Stroke::NONE,
			..Default::default()
		})
		.show(ctx, |ui| {
			self.viewport_widget(ui);
		});
	}
}