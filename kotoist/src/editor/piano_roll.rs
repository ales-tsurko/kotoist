use std::sync::{Arc, Mutex};

use egui_glow::glow;
use glow::HasContext as _;
use nih_plug_egui::egui;

static mut COUNTER: f32 = 0.0;

#[derive(Default)]
pub(crate) struct PianoRoll {
    notes: Arc<Mutex<Option<NotesGl>>>,
    gutters: Arc<Mutex<Option<GuttersGl>>>,
}

impl PianoRoll {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let rect = ui.available_rect_before_wrap();
            let painter = ui.painter();

            let gutters = self.gutters.clone();

            painter.add(egui::PaintCallback {
                rect,
                callback: Arc::new(egui_glow::CallbackFn::new(move |_, gl_painter| {
                    let mut pr_gutters = gutters.lock().unwrap();
                    if pr_gutters.is_none() {
                        *pr_gutters = Some(GuttersGl::new(gl_painter.gl(), 3));
                    }

                    pr_gutters.as_ref().unwrap().paint(gl_painter.gl());
                })),
            });

            let notes_gl = self.notes.clone();

            painter.add(egui::PaintCallback {
                rect,
                callback: Arc::new(egui_glow::CallbackFn::new(move |_, gl_painter| {
                    let mut pr_notes = notes_gl.lock().unwrap();
                    if pr_notes.is_none() {
                        *pr_notes = Some(NotesGl::new(
                            gl_painter.gl(),
                            &[
                                NoteInstance {
                                    pitch: 0.0,
                                    channel: 0.0,
                                    start_time: 0.0,
                                    off_time: 1.0,
                                },
                                NoteInstance {
                                    pitch: 12.0,
                                    channel: 0.0,
                                    start_time: 0.0,
                                    off_time: 1.0,
                                },
                                NoteInstance {
                                    pitch: 16.0,
                                    channel: 15.0,
                                    start_time: 1.0,
                                    off_time: 2.0,
                                },
                                NoteInstance {
                                    pitch: 19.0,
                                    channel: 5.0,
                                    start_time: 1.0,
                                    off_time: 2.0,
                                },
                                NoteInstance {
                                    pitch: 35.0,
                                    channel: 0.0,
                                    start_time: 0.0,
                                    off_time: 1.0,
                                },
                            ],
                        ));
                    }

                    unsafe {
                        COUNTER += 1.0 / 60.0;
                        pr_notes
                            .as_ref()
                            .unwrap()
                            .paint(gl_painter.gl(), COUNTER, 8.0, 0.042);
                    }
                })),
            });
        });
    }
}

struct NotesGl {
    program: glow::Program,
    vao: glow::VertexArray,
    instance_count: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct NoteInstance {
    pitch: f32,
    channel: f32,
    start_time: f32,
    off_time: f32,
    // off_time < start_time means note is still on
    // duration can be computed as off_time - start_time if off_time > start_time
}

impl NotesGl {
    fn new(gl: &glow::Context, notes: &[NoteInstance]) -> Self {
        let vs = include_str!("note.vert");
        let fs = include_str!("note.frag");

        unsafe {
            // Compile program
            let program = gl.create_program().unwrap();

            let vs_handle = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs_handle, vs);
            gl.compile_shader(vs_handle);
            assert!(
                gl.get_shader_compile_status(vs_handle),
                "VS: {}",
                gl.get_shader_info_log(vs_handle)
            );
            gl.attach_shader(program, vs_handle);

            let fs_handle = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs_handle, fs);
            gl.compile_shader(fs_handle);
            assert!(
                gl.get_shader_compile_status(fs_handle),
                "FS: {}",
                gl.get_shader_info_log(fs_handle)
            );
            gl.attach_shader(program, fs_handle);

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "Prog: {}",
                gl.get_program_info_log(program)
            );
            gl.detach_shader(program, vs_handle);
            gl.detach_shader(program, fs_handle);
            gl.delete_shader(vs_handle);
            gl.delete_shader(fs_handle);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            // A unit quad for a single note instance
            let vertices: [f32; 8] = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
            let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

            let vbo_vertices = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_vertices));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );

            let ebo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::STATIC_DRAW,
            );

            // a_pos
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 2 * 4, 0);

            // Instance data
            let mut instance_data = Vec::with_capacity(notes.len() * 4);
            for n in notes {
                instance_data.push(n.pitch);
                instance_data.push(n.channel);
                instance_data.push(n.start_time);
                instance_data.push(n.off_time);
            }

            let vbo_instances = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo_instances));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&instance_data),
                glow::STATIC_DRAW,
            );

            let stride = 4 * 4; // pitch, channel, start_time, off_time (4 floats)
            let mut offset = 0;
            // a_pitch
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 1, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(1, 1);
            offset += 4;
            // a_channel
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(2, 1);
            offset += 4;
            // a_start_time
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(3, 1, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(3, 1);
            offset += 4;
            // a_off_time
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_pointer_f32(4, 1, glow::FLOAT, false, stride, offset);
            gl.vertex_attrib_divisor(4, 1);

            gl.bind_vertex_array(None);

            Self {
                program,
                vao,
                instance_count: notes.len() as i32,
            }
        }
    }

    fn paint(&self, gl: &glow::Context, time: f32, beats_to_show: f32, visual_width: f32) {
        unsafe {
            gl.use_program(Some(self.program));
            let u_time = gl.get_uniform_location(self.program, "u_time");
            let u_beats = gl.get_uniform_location(self.program, "u_beats");
            let u_visual_width = gl.get_uniform_location(self.program, "u_visual_width");
            gl.uniform_1_f32(u_time.as_ref(), time);
            gl.uniform_1_f32(u_beats.as_ref(), beats_to_show);
            gl.uniform_1_f32(u_visual_width.as_ref(), visual_width);

            gl.bind_vertex_array(Some(self.vao));
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.draw_elements_instanced(
                glow::TRIANGLES,
                6,
                glow::UNSIGNED_INT,
                0,
                self.instance_count,
            );
            gl.bind_vertex_array(None);
        }
    }
}

pub(crate) struct GuttersGl {
    program: glow::Program,
    vao: glow::VertexArray,
    vertex_count: i32,
}

impl GuttersGl {
    fn new(gl: &glow::Context, octaves: usize) -> Self {
        let total_semitones = octaves * 12;
        let key_width = 2.0 / total_semitones as f32;

        fn is_white_key(semitone: u8) -> bool {
            matches!(semitone % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
        }

        let vs = r#"#version 330
            layout(location = 0) in vec2 a_pos;
            layout(location = 1) in vec3 a_color;
            out vec3 v_color;
            void main() {
                gl_Position = vec4(a_pos, 0.0, 1.0);
                v_color = a_color;
            }
        "#;

        let fs = r#"#version 330
            in vec3 v_color;
            out vec4 out_color;
            void main() {
                out_color = vec4(v_color, 1.0);
            }
        "#;

        let program = unsafe {
            let program = gl.create_program().unwrap();
            let vs_handle = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs_handle, vs);
            gl.compile_shader(vs_handle);
            assert!(
                gl.get_shader_compile_status(vs_handle),
                "VS: {}",
                gl.get_shader_info_log(vs_handle)
            );
            gl.attach_shader(program, vs_handle);

            let fs_handle = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs_handle, fs);
            gl.compile_shader(fs_handle);
            assert!(
                gl.get_shader_compile_status(fs_handle),
                "FS: {}",
                gl.get_shader_info_log(fs_handle)
            );
            gl.attach_shader(program, fs_handle);

            gl.link_program(program);
            assert!(
                gl.get_program_link_status(program),
                "Prog: {}",
                gl.get_program_info_log(program)
            );
            gl.detach_shader(program, vs_handle);
            gl.detach_shader(program, fs_handle);
            gl.delete_shader(vs_handle);
            gl.delete_shader(fs_handle);

            program
        };

        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };

        let mut vertices = Vec::new();
        for i in 0..total_semitones {
            let left = -1.0 + i as f32 * key_width;
            let right = left + key_width;
            let white = is_white_key(i as u8);

            let (r, g, b) = if white {
                (0.03, 0.03, 0.03)
            } else {
                (0.0, 0.0, 0.0)
            };

            // Triangle 1
            vertices.extend_from_slice(&[left, -1.0, r, g, b]);
            vertices.extend_from_slice(&[left, 1.0, r, g, b]);
            vertices.extend_from_slice(&[right, 1.0, r, g, b]);

            // Triangle 2
            vertices.extend_from_slice(&[right, 1.0, r, g, b]);
            vertices.extend_from_slice(&[right, -1.0, r, g, b]);
            vertices.extend_from_slice(&[left, -1.0, r, g, b]);
        }

        let vertex_count = (vertices.len() / 5) as i32;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );

            // a_pos (2 floats)
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 5 * 4, 0);

            // a_color (3 floats)
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 5 * 4, 2 * 4);

            gl.bind_vertex_array(None);
        }

        Self {
            program,
            vao,
            vertex_count,
        }
    }

    fn paint(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
            gl.bind_vertex_array(None);
        }
    }
}
