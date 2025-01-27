use std::collections::{HashMap, LinkedList};

use ggez::graphics::{self, BlendComponent, BlendFactor, BlendMode, BlendOperation};

#[derive(Default, Clone)]
struct PixBuf {
    pix: Vec<u8>,
    w: usize,
    h: usize,
}

#[derive(Default, Clone)]
pub struct Painter {
    pub(crate) shapes: Vec<egui::ClippedPrimitive>,
    pub(crate) textures_delta: LinkedList<egui::TexturesDelta>,
    paint_jobs: Vec<(egui::TextureId, graphics::Mesh, graphics::Rect)>,
    textures: HashMap<egui::TextureId, graphics::Image>,
    images: HashMap<egui::TextureId, PixBuf>,
}

impl Painter {
    pub fn draw(&mut self, canvas: &mut graphics::Canvas, scale_factor: f32) {
        let prev_blend = canvas.blend_mode();
        canvas.set_blend_mode(BlendMode {
            color: BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: BlendFactor::OneMinusDstAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
        });
        for (id, mesh, clip) in self.paint_jobs.iter() {
            canvas.set_scissor_rect(*clip).unwrap();
            canvas.draw_textured_mesh(
                mesh.clone(),
                self.textures[id].clone(),
                graphics::DrawParam::default().scale([scale_factor, scale_factor]),
            );
        }
        canvas.set_default_scissor_rect();
        canvas.set_blend_mode(prev_blend);
    }

    pub fn clear(&mut self) {
        self.paint_jobs.clear();
    }

    pub fn update(&mut self, ctx: &mut ggez::Context, scale_factor: f32) {
        // Create and free textures
        while let Some(textures_delta) = self.textures_delta.pop_front() {
            self.update_textures(ctx, textures_delta);
        }

        // generating meshes
        for egui::ClippedPrimitive {
            primitive,
            clip_rect,
        } in self.shapes.iter()
        {
            match primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    if mesh.vertices.len() < 3 {
                        continue;
                    }

                    let vertices = mesh
                        .vertices
                        .iter()
                        .map(|v| graphics::Vertex {
                            position: [v.pos.x, v.pos.y],
                            uv: [v.uv.x, v.uv.y],
                            color: egui::Rgba::from(v.color).to_array(),
                        })
                        .collect::<Vec<_>>();

                    self.paint_jobs.push((
                        mesh.texture_id,
                        graphics::Mesh::from_data(
                            ctx,
                            graphics::MeshData {
                                vertices: vertices.as_slice(),
                                indices: mesh.indices.as_slice(),
                            },
                        ),
                        graphics::Rect::new(
                            clip_rect.min.x * scale_factor,
                            clip_rect.min.y * scale_factor,
                            (clip_rect.max.x - clip_rect.min.x) * scale_factor,
                            (clip_rect.max.y - clip_rect.min.y) * scale_factor,
                        ),
                    ));
                }
                egui::epaint::Primitive::Callback(_) => {
                    panic!("Custom rendering callbacks are not implemented yet");
                }
            }
        }
    }

    pub fn update_textures(
        &mut self,
        ctx: &mut ggez::Context,
        textures_delta: egui::TexturesDelta,
    ) {
        // set textures
        for (id, delta) in &textures_delta.set {
            let pixbuf = PixBuf::from_image_data(&delta.image);
            if let Some(pos) = delta.pos {
                eprintln!("Error: Non-zero offset texture updates are not implemented yet");
                let Some(mut img) = self.images.remove(id) else {
                    eprintln!("Got update request for unknown image");
                    continue;
                };
                img.blit(&pixbuf, (pos[0], pos[1]));
            } else {
                self.textures.insert(*id, pixbuf.to_texture(ctx));
                self.images.insert(*id, pixbuf);
            }
        }

        // free textures
        for id in &textures_delta.free {
            self.textures.remove(id);
            self.images.remove(id);
        }
    }
}

impl PixBuf {
    fn from_color(color: &egui::ColorImage) -> Self {
        let mut pix: Vec<u8> = Vec::with_capacity(color.pixels.len() * 4);
        for pixel in &color.pixels {
            pix.extend(pixel.to_array());
        }
        Self {
            pix,
            w: color.width(),
            h: color.height(),
        }
    }

    fn from_font(font: &egui::FontImage) -> Self {
        let mut pix: Vec<u8> = Vec::with_capacity(font.pixels.len() * 4);
        for pixel in font.srgba_pixels(None) {
            pix.extend(pixel.to_array());
        }
        Self {
            pix,
            w: font.width(),
            h: font.height(),
        }
    }

    fn from_image_data(img: &egui::ImageData) -> Self {
        match &img {
            egui::ImageData::Color(image) => Self::from_color(image),
            egui::ImageData::Font(image) => Self::from_font(image),
        }
    }

    fn to_texture(&self, ctx: &mut ggez::Context) -> graphics::Image {
        graphics::Image::from_pixels(
            ctx,
            self.pix.as_slice(),
            graphics::ImageFormat::Rgba8UnormSrgb,
            self.w as u32,
            self.h as u32,
        )
    }

    fn blit(&mut self, pix: &PixBuf, pos: (usize, usize)) {
        for row in pos.1..pos.1 + pix.h {
            let dst = row * self.w + pos.0;
            let src = row * pix.h;
            for (i, j) in (dst..dst + pix.w).zip(src..src + pix.w) {
                println!("{i} <- {j}");
            }
        }
    }
}
