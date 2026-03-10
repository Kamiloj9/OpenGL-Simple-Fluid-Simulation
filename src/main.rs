use std::{fmt::format, time::Instant, env};

use glium::{Surface, implement_vertex, uniform, glutin::display::GetGlDisplay};
use winit::event;
use crate::fluid::Simulation;
mod obj_loader;
mod math;
mod camera;
mod fluid;
use nalgebra_glm as glm;

const APP_NAME: &'static str = "OpenGl Simple Fluid Simulation";

#[derive(Clone, Copy)]
struct Vertex{
    position: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position,tex_coords, color);

#[derive(Clone, Copy, Debug)]
struct VertexTeapot{
    position: [f32; 3]
}
implement_vertex!(VertexTeapot, position);

#[derive(Clone, Copy)]
struct NormalTeapot{
    normal: [f32; 3]
}
implement_vertex!(NormalTeapot, normal);

// https://matthias-research.github.io/pages/publications/sca03.pdf
// https://old.cescg.org/CESCG-2012/papers/Horvath-Real-time_particle_simulation_of_fluids.pdf
fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let event_loop = winit::event_loop::EventLoopBuilder::new().build();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title(APP_NAME)
        .with_inner_size(1280, 720)
        .build(&event_loop);

    let shape = vec![
    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0], color: [1.0, 0.0, 1.0]  },
    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0]  },

    Vertex { position: [ 0.5,  0.5], tex_coords: [1.0, 1.0], color: [1.0, 1.0, 1.0]  },
    Vertex { position: [-0.5,  0.5], tex_coords: [0.0, 1.0], color: [1.0, 0.0, 1.0]  },
    Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0], color: [1.0, 1.0, 1.0]  },
];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = include_bytes!("../res/shaders/vertex_simple.vs");
    let vertex_shader_src = std::str::from_utf8(vertex_shader_src).unwrap();

    let fragment_shader_src = include_bytes!("../res/shaders/fragment_simple.fs");
    let fragment_shader_src = std::str::from_utf8(fragment_shader_src).unwrap();


    println!("Render device: {}", display.get_opengl_renderer_string());
    println!("OpenGL version: {}", display.get_opengl_version_string());

    let image = image::load(std::io::Cursor::new(&include_bytes!("../res/ironman.png")),
        image::ImageFormat::Png).unwrap().to_rgba8();
        let image_dimensions = image.dimensions();
    
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
    
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    

    let teapot = obj_loader::load("data/teapot.obj").unwrap();
    let teapot_vertex_buffer = glium::VertexBuffer::new(&display, &teapot).unwrap();

    let params = glium::DrawParameters{depth: glium::Depth{
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
    }, ..Default::default()};//depth: glium::Depth { test: glium::DepthTest::IfLess, write: false, ..Default::default() } , ..Default::default()};
      //  depth: glium::Depth { test: glium::DepthTest::IfLess, write: true,..Default::default() }, ..Default::default()};

    

    let mut egui_glium = egui_glium::EguiGlium::new(&display, &window, &event_loop);

    

    let mut clear_color: [f32; 3] = [114.0/255.0, 181.0/255.0, 157.0/255.0];

    let mut pitch: f32 = 0.0;
    let mut yaw: f32 = 90.0;

    let mut mouse_pos_x = 0.0;
    let mut mouse_pos_y = 0.0;
    let mut enable_camera_control = false;

    let mut fps: f32 = 0.0;
    let mut delta_time: f32 = 0.0;

    let mut main_camera = camera::Camera::new(glm::vec3(0.0, 0.0, -6.0), pitch, yaw);

    

    let mut stat_time = Instant::now();
    let mut current_model_matrix = nalgebra_glm::Mat4::identity();

    let mut bounds_postion = nalgebra_glm::vec3(5.0, -2.0, 5.0f32);
    let mut bounds_scale = nalgebra_glm::vec3(5.0, 8.0, 5.0f32);
    let mut bounds_rotation = nalgebra_glm::vec3(-1.0, 0.0, 0.0);
    let mut draw_bounds = true;

    let mut simulation = Simulation::new(&display, &bounds_postion, &bounds_rotation, &bounds_scale);
    

    let mut kernel_radius = 0.5f32;
    let mut particle_mass = 1.0f32;
    let mut gass_constant  = 1.0f32;
    let mut rest_density = 15.0f32;
    let mut viscosity = 0.6f32;

    let mut draw_cubes = false;

    let mut iso_level = 1.0f32;
    let mut debug_surface_points = false;
    let mut surface_wireframe = false;
    let mut surface_color: [f32;3] = [0.0, 0.0, 0.8];
    let mut surface_alpha: f32 = 0.5;

    let spaw_particle_location = [5.0 , 0.0, 5.0f32];
    simulation.add_particle(spaw_particle_location);
    
    let mut t: f32 = 0.0;
    event_loop.run(move |ev, _, control_flow| {

        let _repaint = egui_glium.run(&window, |egui_ctx| {
            egui::SidePanel::left("Controlls").show(egui_ctx, |ui| {
                ui.heading("Fluid Simulation");
                ui.label(format!("FPS: {}", fps as u32));
                ui.label(format!("particle count {}", simulation.particles.len()));
                if simulation.compute_supported() {
                    ui.label("Backend: GPU compute");
                } else {
                    ui.label("Backend: CPU compatibility (Apple/OpenGL 4.1)");
                }
                ui.horizontal(|ui| {
                    ui.label("kernel radius");
                    ui.add(egui::Slider::new(&mut kernel_radius, 0.01..=3.0));
                });
                ui.horizontal(|ui| {
                    ui.label("particle mass");
                    ui.add(egui::Slider::new(&mut particle_mass, 0.1..=5.0));
                });
                ui.horizontal(|ui| {
                    ui.label("gass constant");
                    ui.add(egui::Slider::new(&mut gass_constant, 0.1..=5.0));
                });
                ui.horizontal(|ui| {
                    ui.label("rest density");
                    ui.add(egui::Slider::new(&mut rest_density, 0.1..=100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("viscosity factor");
                    ui.add(egui::Slider::new(&mut viscosity, 0.1..=5.0));
                });
                ui.horizontal(|ui| {
                    ui.label("iso level");
                    ui.add(egui::Slider::new(&mut iso_level, 0.001..=100.0));
                });
                ui.checkbox(&mut draw_cubes, "Draw cubes");
                ui.label(format!("Draw mode: {}", if draw_cubes { "Cubes" } else { "Surface" }));
                if !draw_cubes {
                    ui.separator();
                    ui.heading("Surface");
                    ui.checkbox(&mut debug_surface_points, "Show points (debug)");
                    ui.checkbox(&mut surface_wireframe, "Wireframe");
                    ui.horizontal(|ui| {
                        ui.label("Color");
                        egui::color_picker::color_edit_button_rgb(ui, &mut surface_color);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Alpha");
                        ui.add(egui::Slider::new(&mut surface_alpha, 0.0..=1.0));
                    });
                }

                
                
                ui.horizontal(|ui| {
                    ui.label("Clear color");
                    egui::color_picker::color_edit_button_rgb(ui, &mut clear_color);
                });
                ui.heading("Camera");
                ui.label(format!("pitch: {:.2} yaw: {:.2}", main_camera.pitch, main_camera.yaw));
                ui.label(format!("camera pos: x:{:.2} y:{:.2} z:{:.2}", main_camera.position.x, main_camera.position.y, main_camera.position.z));
                ui.heading("Bounds");

                // Bounds position
                ui.label(format!("Bounds position {:.2} {:.2} {:.2}", bounds_postion.x, bounds_postion.y, bounds_postion.z));
                ui.horizontal(|ui| {
                    ui.label("set position x");
                    ui.add(egui::Slider::new(&mut bounds_postion.x, -5.1..=5.3));
                });
                ui.horizontal(|ui| {
                    ui.label("set position y");
                    ui.add(egui::Slider::new(&mut bounds_postion.y, -5.1..=5.3));
                });
                ui.horizontal(|ui| {
                    ui.label("set position z");
                    ui.add(egui::Slider::new(&mut bounds_postion.z, -5.1..=5.3));
                });

                // Bounds rotation
                ui.label(format!("Bounds rotation {:.2} {:.2} {:.2}", bounds_rotation.x, bounds_rotation.y, bounds_rotation.z));
                ui.horizontal(|ui| {
                    ui.label("set rotation x");
                    ui.add(egui::Slider::new(&mut bounds_rotation.x, -3.14..=3.14));
                });
                ui.horizontal(|ui| {
                    ui.label("set rotation y");
                    ui.add(egui::Slider::new(&mut bounds_rotation.y, -3.14..=3.14));
                });
                ui.horizontal(|ui| {
                    ui.label("set rotation z");
                    ui.add(egui::Slider::new(&mut bounds_rotation.z, -3.14..=3.14));
                });

                // Bounds scale
                ui.label(format!("Bounds scale {:.2} {:.2} {:.2}", bounds_scale.x, bounds_scale.y, bounds_scale.z));
                ui.horizontal(|ui| {
                    ui.label("set scale x");
                    ui.add(egui::Slider::new(&mut bounds_scale.x, 0.1..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("set scale y");
                    ui.add(egui::Slider::new(&mut bounds_scale.y, 0.1..=10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("set scale z");
                    ui.add(egui::Slider::new(&mut bounds_scale.z, 0.1..=10.0));
                });

                ui.checkbox(&mut draw_bounds, "Draw bounds");

                ui.label(format!("density stage time {}ms", simulation.density_stage_time));
                ui.label(format!("simulation stage time {}ms", simulation.simulation_stage_time));
                ui.label(format!("marching cubes compute time {}ms", simulation.marching_cubes_compute_stage_time));
                ui.label(format!("marching cubes cpu time {}ms", simulation.marching_cubes_cpu_time));

                
                
            });
        });

        

        match ev {
            
            winit::event::Event::WindowEvent { event, .. } => {

                match event{
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        println!("Exit requested");
                    },
                    winit::event::WindowEvent::Resized(window_size) => {
                        display.resize(window_size.into());
                    },
                    winit::event::WindowEvent::CursorMoved { device_id: _, position, modifiers: _ } => {
                        if enable_camera_control {

                            let delta_x: f32 = (position.x - mouse_pos_x) as f32;
                            let delta_y: f32 = (position.y - mouse_pos_y) as f32;

                            pitch -= delta_y * 0.1;
                            yaw -= delta_x * 0.1;
                        }
                        mouse_pos_x = position.x;
                        mouse_pos_y = position.y;
                        
                    },
                     winit::event::WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                         

                         match input.virtual_keycode {
                            Some(winit::event::VirtualKeyCode::F) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    simulation.add_particle(spaw_particle_location);
                                }
                            },
                             Some(winit::event::VirtualKeyCode::Space) => {
                                
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Up as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Up as u32);
                                }
                                
                             },
                             Some(winit::event::VirtualKeyCode::LControl) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Down as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Down as u32);
                                }
                             },
                             Some(winit::event::VirtualKeyCode::W) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Forward as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Forward as u32);
                                }
                             },
                             Some(winit::event::VirtualKeyCode::S) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Backwards as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Backwards as u32);
                                }
                             },
                             Some(winit::event::VirtualKeyCode::A) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Left as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Left as u32);
                                }
                             },
                             Some(winit::event::VirtualKeyCode::D) => {
                                if winit::event::ElementState::Pressed == input.state{
                                    main_camera.camera_control_state |= camera::CameraMovment::Right as u32;
                                } else{
                                    main_camera.camera_control_state &= !(camera::CameraMovment::Right as u32);
                                }
                             }
                             _ => (),
                         }
                     },
                    winit::event::WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                        
                        if winit::event::MouseButton::Right == button {
                            if winit::event::ElementState::Pressed == state {
                                enable_camera_control = true;
                            } else {
                                enable_camera_control = false;
                            }
                        }
                    }
                    _ => (),
                }

                if egui_glium.on_event(&event).repaint{
                    window.request_redraw();
                }
            }

            winit::event::Event::RedrawEventsCleared => {
                window.request_redraw();
            },
            winit::event::Event::RedrawRequested(_) => {
                
                let end_time = Instant::now();
                let elapsed_time = end_time.duration_since(stat_time).as_micros();
                let elapsed_time: f32 = elapsed_time as f32 / 1_000_000.0;
                stat_time = end_time;
                delta_time = elapsed_time;
                fps = 1.0/elapsed_time;
                
                let mut frame = display.draw();
                t += 0.00002;
                
                let (width, height) =  frame.get_dimensions();
                let ar = height as f32 / width as f32;

                let proj = math::create_projection_matrix(ar, 3.141592 / 3.0, 1024.0, 0.1);
                if main_camera.camera_control_state & camera::CameraMovment::Up as u32 > 0{
                    main_camera.position = main_camera.position + glm::vec3(0.0, 1.0, 0.0) * delta_time;
                } if  main_camera.camera_control_state & camera::CameraMovment::Down as u32 > 0{
                    main_camera.position = main_camera.position - glm::vec3(0.0, 1.0, 0.0) * delta_time;
                } if main_camera.camera_control_state & camera::CameraMovment::Left as u32 > 0 {
                    main_camera.position = main_camera.position - glm::normalize(&main_camera.right) * delta_time;
                } if main_camera.camera_control_state & camera::CameraMovment::Right as u32 > 0 {
                    main_camera.position = main_camera.position + glm::normalize(&main_camera.right) * delta_time;
                } if main_camera.camera_control_state & camera::CameraMovment::Forward as u32 > 0 {
                    main_camera.position = main_camera.position + glm::normalize(&main_camera.direction) * delta_time;
                } if main_camera.camera_control_state & camera::CameraMovment::Backwards as u32 > 0 {
                    main_camera.position = main_camera.position - glm::normalize(&main_camera.direction) * delta_time;
                }
                main_camera.update_rotation(pitch, yaw);
                let view = main_camera.calculate_view_matrix();

                
                let identity = nalgebra_glm::Mat4::identity();
                let rotation = nalgebra_glm::rotate(&identity, -0.35, &nalgebra_glm::vec3(0.0, 0.0, 1.0));
                let translation = nalgebra_glm::translate(&identity, &nalgebra_glm::vec3(0.6, -1.2, 5.0));

                let model: [[f32; 4]; 4] = (translation * rotation).into();
                let uniforms = uniform! {
                    model: model,
                    
                    proj: math::mat4_to_arr(&proj),
                    view: view,
                };

                simulation.set_draw_cubes(draw_cubes);
                simulation.update_bounds(&bounds_postion, &bounds_rotation, &bounds_scale);
                simulation.simulate(delta_time, 1, &display, kernel_radius,particle_mass,gass_constant,rest_density, viscosity, iso_level);

                
                frame.clear_color_and_depth((clear_color[0], clear_color[1], clear_color[2], 1.0), 1.0);
                
                frame.draw(&teapot_vertex_buffer, &indices, &program, &uniforms, &params).unwrap();
                simulation.present(&mut frame, &params, math::mat4_to_arr(&proj), view, &display, draw_bounds, debug_surface_points, surface_wireframe, surface_color, surface_alpha);
                
                egui_glium.paint(&display, &mut frame);

                frame.finish().unwrap();
            },
            _ => (),
        }
    });
}
