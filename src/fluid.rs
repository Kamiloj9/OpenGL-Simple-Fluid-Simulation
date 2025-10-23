use std::{thread::sleep, path::Display, borrow::Cow, iter, time::Instant};

use glium::{Frame, Surface, DrawParameters, Program, glutin::surface::WindowSurface, uniform, implement_vertex, VertexBuffer, PolygonMode, program::ComputeShader, implement_uniform_block, implement_buffer_content, texture::RawImage2d, uniforms::{self, AsUniformValue, UniformValue, UniformType, UniformBlock}};
use winit::dpi::Position;
use rand::Rng;

use self::marching_cubes::Grid;

#[path ="marching_cubes.rs"] mod marching_cubes;

pub struct Particle{
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3]
}

pub struct Simulation{
    draw_cubes: bool,
    pub particles: Vec<Particle>,
    render_program: Program,
    bounding_program: Program,
    fluid_program: Program,
    compute_shader: ComputeShader,
    density_compute_shader: ComputeShader,
    compute_fluid_mesh_shader: ComputeShader,
    debug_vertex_buffer: VertexBuffer<Vertex>,
    fluid_mesh_buffer: VertexBuffer<Vertex>,
    debug_particle_buffer: VertexBuffer<Offset>,
    fluid_triangles: Vec<Vertex>,
    // min max
    bounds_postion: nalgebra_glm::Vec3,
    bounds_scale: nalgebra_glm::Vec3,
    bounds_rotation: nalgebra_glm::Vec3,
    buffer: glium::uniforms::UniformBuffer<Data>,
    fluid_mesh_uniform: glium::uniforms::UniformBuffer<VertexData>,
    pub density_stage_time: u32,
    pub simulation_stage_time: u32,
    pub marching_cubes_compute_stage_time: u32,
    pub marching_cubes_cpu_time: u32,
}

const GRAVITY: f32 = -9.81;
const DUMPING: f32 = 0.7;
const MAXNUMBEROFPARTICLES: usize = 100000;
const GRIDRESOLUTION: usize = 15;

#[derive(Clone, Copy)]
struct Vertex{
    position: [f32; 3]
    
}
#[derive(Clone, Copy)]
struct Offset{
    offset: [f32; 3]
}
implement_vertex!(Vertex, position);
implement_vertex!(Offset, offset);

#[repr(C)]
#[derive(Clone, Copy)]
struct Data {
    num_of_particles: i32,
    gravity: f32,
    dumping: f32,
    delta_time: f32,
    time_steps: i32,
    mass: f32,
    gass_constant: f32,
    rest_density: f32,
    kernel_radius: f32,
    viscosity: f32,
    _padding: [f32; 2],
    world_to_local: [[f32;4];4],
    local_to_world: [[f32; 4]; 4],
    positions: [[f32; 4]; MAXNUMBEROFPARTICLES],
    velocities: [[f32; 4]; MAXNUMBEROFPARTICLES],
    // 0 density // 1 ideal density
    density: [[f32; 4]; MAXNUMBEROFPARTICLES],
    acceleration: [[f32; 4]; MAXNUMBEROFPARTICLES],
    vertex_data: [[f32; 4]; GRIDRESOLUTION.pow(4)],
    denisty_sample: [[f32; 4]; GRIDRESOLUTION.pow(4)],
    //out_triangles: [[f32; 4]; GRIDRESOLUTION.pow(3) * 15],
    iso_level: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VertexData{
    out_triangles: [[f32; 4]; GRIDRESOLUTION.pow(3) * 15],
}

impl AsUniformValue for VertexData{
    fn as_uniform_value(&self) -> UniformValue {
        panic!("not implemented")
    }
}

impl AsUniformValue for Data {
    fn as_uniform_value(&self) -> UniformValue {
        panic!("not implemented")
    }
}
impl UniformBlock for Data {
    fn build_layout(_base_offset: usize) -> glium::program::BlockLayout {
        panic!("not implemented")
    }

    fn matches(_: &glium::program::BlockLayout, _base_offset: usize) -> Result<(), uniforms::LayoutMismatchError> {
        Ok(())
    }
}
impl UniformBlock for VertexData {
    fn build_layout(_base_offset: usize) -> glium::program::BlockLayout {
        panic!("not implemented")
    }

    fn matches(_: &glium::program::BlockLayout, _base_offset: usize) -> Result<(), uniforms::LayoutMismatchError> {
        Ok(())
    }
}

impl Simulation {
    pub fn new(display: &glium::Display<WindowSurface>, bounds_position: &nalgebra_glm::Vec3, bounds_rotation: &nalgebra_glm::Vec3, bounds_scale: &nalgebra_glm::Vec3) -> Self {

        let vertex_shader_src = include_bytes!("../res/shaders/cube.vs");
        let vertex_shader_src = std::str::from_utf8(vertex_shader_src).unwrap();
    
        let fragment_shader_src = include_bytes!("../res/shaders/cube.fs");
        let fragment_shader_src = std::str::from_utf8(fragment_shader_src).unwrap();

        let vertex_shader_bounds_src = include_bytes!("../res/shaders/bounds.vs");
        let vertex_shader_bounds_src = std::str::from_utf8(vertex_shader_bounds_src).unwrap();

        let fragment_shader_bounds_src = include_bytes!("../res/shaders/bounds.fs");
        let fragment_shader_bounds_src = std::str::from_utf8(fragment_shader_bounds_src).unwrap();

        let compute_shader_src = include_bytes!("../res/shaders/fluid.compute");
        let compute_shader_src = std::str::from_utf8(compute_shader_src).unwrap();

        let fluid_shader_compute_src = include_bytes!("../res/shaders/marching_cubes.compute");
        let fluid_shader_compute_src = std::str::from_utf8(fluid_shader_compute_src).unwrap();

        let density_compute_shader_src = include_bytes!("../res/shaders/calc_density.compute");
        let density_compute_shader_src = std::str::from_utf8(density_compute_shader_src).unwrap();

        let fluid_vertex_shader_src = include_bytes!("../res/shaders/fluid.vs");
        let fluid_vertex_shader_src = std::str::from_utf8(fluid_vertex_shader_src).unwrap();

        let fluid_fragment_shader_src = include_bytes!("../res/shaders/fluid.fs");
        let fluid_fragment_shader_src = std::str::from_utf8(fluid_fragment_shader_src).unwrap();

        let shape = vec![
            Vertex { position: [-0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, -0.5] },
            Vertex { position: [0.5, 0.5, -0.5] },
            Vertex { position: [0.5, 0.5, -0.5] },
            Vertex { position: [-0.5, 0.5, -0.5] },
            Vertex { position: [-0.5, -0.5, -0.5] },

            Vertex { position: [-0.5, -0.5, 0.5] },
            Vertex { position: [0.5, -0.5, 0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, -0.5, 0.5] },

            Vertex { position: [-0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, -0.5] },
            Vertex { position: [-0.5, -0.5, -0.5] },
            Vertex { position: [-0.5, -0.5, -0.5] },
            Vertex { position: [-0.5, -0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, 0.5] },

            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [0.5, 0.5, -0.5] },
            Vertex { position: [0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, 0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },

            Vertex { position: [-0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, -0.5] },
            Vertex { position: [0.5, -0.5, 0.5] },
            Vertex { position: [0.5, -0.5, 0.5] },
            Vertex { position: [-0.5, -0.5, 0.5] },
            Vertex { position: [-0.5, -0.5, -0.5] },

            Vertex { position: [-0.5, 0.5, -0.5] },
            Vertex { position: [0.5, 0.5, -0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, 0.5] },
            Vertex { position: [-0.5, 0.5, -0.5] }
        ];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();

        let instance_data = vec![Offset{ offset: [0.0,0.0,0.0]}; MAXNUMBEROFPARTICLES];
        let fluid_mesh_data = vec![Vertex{position: [0.0, 0.0, 0.0]}; GRIDRESOLUTION.pow(3) * 15];

        Self{
            draw_cubes: false,
            particles: Vec::new(),
            render_program: glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap(),
            bounding_program: glium::Program::from_source(display, vertex_shader_bounds_src, fragment_shader_bounds_src, None).unwrap(),
            fluid_program: glium::Program::from_source(display, fluid_vertex_shader_src, fluid_fragment_shader_src, None).unwrap(),
            debug_vertex_buffer: vertex_buffer,
            bounds_postion: bounds_position.clone(),
            bounds_scale: bounds_scale.clone(),
            bounds_rotation: bounds_rotation.clone(),
            compute_shader: ComputeShader::from_source(display, compute_shader_src).unwrap(),
            density_compute_shader: ComputeShader::from_source(display, density_compute_shader_src).unwrap(),
            compute_fluid_mesh_shader: ComputeShader::from_source(display, fluid_shader_compute_src).unwrap(),
            debug_particle_buffer: glium::VertexBuffer::dynamic(display, &instance_data).unwrap(),
            fluid_mesh_buffer: glium::VertexBuffer::dynamic(display, &fluid_mesh_data).unwrap(),
            buffer: glium::uniforms::UniformBuffer::empty(display).unwrap(),
            fluid_mesh_uniform: glium::uniforms::UniformBuffer::empty(display).unwrap(),
            fluid_triangles: vec![Vertex{position: [0.0, 0.0, 0.0]}; (GRIDRESOLUTION.pow(3) * 15) as usize],
            density_stage_time: 0,
            simulation_stage_time: 0,
            marching_cubes_compute_stage_time: 0,
            marching_cubes_cpu_time: 0,
        }
    }

    pub fn set_draw_cubes(&mut self, val: bool){
        self.draw_cubes = val;
    }

    pub fn add_particle(&mut self, position: [f32; 3]){
        let mut rng = rand::thread_rng();

        let mut num = 32;
        while self.particles.len() < MAXNUMBEROFPARTICLES && num > 0 {
            self.particles.push(Particle{position: [position[0] + rng.gen_range(-0.2..0.2), position[1] + rng.gen_range(-0.2..0.2),position[2] + rng.gen_range(-0.2..0.2) ],
                 velocity: [0.0, -2.0, 0.0],//[rng.gen_range(-0.01..0.01), rng.gen_range(-10.0..0.01), rng.gen_range(-0.01..0.01)],
                acceleration: [0.0, 0.0, 0.0]});
            num -= 1;
        }
    }

    pub fn update_bounds(&mut self, bounds_position: &nalgebra_glm::Vec3, bounds_rotation: &nalgebra_glm::Vec3, bounds_scale: &nalgebra_glm::Vec3){
        self.bounds_postion = bounds_position.clone();
        self.bounds_rotation = bounds_rotation.clone();
        self.bounds_scale = bounds_scale.clone();
    }

    pub fn simulate(&mut self, time_step: f32, num_of_steps: u32, display: &glium::Display<WindowSurface>
        ,kernel_radius: f32, particle_mass: f32, gass_constant: f32, rest_density: f32, viscosity: f32, iso_level: f32){

            
        let identity = nalgebra_glm::Mat4::identity();
        
        let trans = nalgebra_glm::translate(&identity, &self.bounds_postion);
        
        let scale = nalgebra_glm::scale(&identity, &self.bounds_scale);
        
        let mut rot = nalgebra_glm::rotate(&identity, self.bounds_rotation[0], &nalgebra_glm::vec3(1.0, 0.0, 0.0));
        rot = nalgebra_glm::rotate(&rot, self.bounds_rotation[1], &nalgebra_glm::vec3(0.0, 1.0, 0.0));
        rot = nalgebra_glm::rotate(&rot, self.bounds_rotation[2], &nalgebra_glm::vec3(0.0, 0.0, 1.0));
        
        let local_to_world = trans * rot * scale;

        
        let world_to_local = nalgebra_glm::inverse(&local_to_world);
        
        {
            let mut mapping = self.buffer.map();
            mapping.num_of_particles = self.particles.len() as i32;
            mapping.gravity = GRAVITY;
            mapping.dumping = DUMPING;
            mapping.local_to_world = local_to_world.into();
            mapping.world_to_local = world_to_local.into();
            mapping.delta_time = time_step;
            mapping.time_steps = num_of_steps as i32;
            mapping.kernel_radius = kernel_radius;
            mapping.mass = particle_mass;
            mapping.gass_constant = gass_constant;
            mapping.rest_density = rest_density;
            mapping.viscosity = viscosity;
            mapping.iso_level = iso_level;
            for (i, pos) in mapping.positions.iter_mut().enumerate(){
                if i >= self.particles.len(){
                    break;
                }
                *pos = [self.particles[i].position[0], self.particles[i].position[1], self.particles[i].position[2], 0.0];
            }
            for (i, vel) in mapping.velocities.iter_mut().enumerate(){
                if i >= self.particles.len(){
                    break;
                }
                *vel = [self.particles[i].velocity[0], self.particles[i].velocity[1], self.particles[i].velocity[2], 0.0];
            }
            for (i, vel) in mapping.acceleration.iter_mut().enumerate(){
                if i >= self.particles.len(){
                    break;
                }
                *vel = [self.particles[i].acceleration[0], self.particles[i].acceleration[1], self.particles[i].acceleration[2], 0.0];
            }
            
        }
        
        let uniforms = uniform! {
            buf: &*self.buffer,
            buf_out: &*self.fluid_mesh_uniform
        };

        let start = Instant::now();
        self.density_compute_shader.execute(uniforms,self.particles.len() as u32, 1, 1);
        self.density_stage_time = (Instant::now().duration_since(start).as_micros()) as u32;
        

        let start = Instant::now();
        self.compute_shader.execute(uniforms, self.particles.len() as u32, 1, 1);
        self.simulation_stage_time = Instant::now().duration_since(start).as_micros() as u32;
        
        
        let start = Instant::now();
        if !self.draw_cubes {
            
            self.compute_fluid_mesh_shader.execute(uniforms, GRIDRESOLUTION as u32, GRIDRESOLUTION as u32, GRIDRESOLUTION as u32);
            
        }
        self.marching_cubes_compute_stage_time = Instant::now().duration_since(start).as_micros() as u32;
        
        
        let mapping = self.buffer.map();
            
        for (i, pos) in mapping.positions.iter().enumerate() {
            if i >= self.particles.len(){
                break;
            }

            self.particles[i].position = [pos[0], pos[1], pos[2]];
                
        }
        for (i, vel) in mapping.velocities.iter().enumerate(){
            if i >= self.particles.len(){
                break;
            }

            self.particles[i].velocity = [vel[0], vel[1], vel[2]];
        }
        for (i, vel) in mapping.acceleration.iter().enumerate(){
            if i >= self.particles.len(){
                break;
            }

            self.particles[i].acceleration = [vel[0], vel[1], vel[2]];
        }

        

        //let mut vertex_data = Vec::new();
        //self.fluid_triangles.clear();
        //self.fluid_triangles = vec![Vertex{position: [0.0,0.0,0.0]}; GRIDRESOLUTION.pow(3) *5];
        //let isolevel = 0.9;
        let start = Instant::now();
        if !self.draw_cubes {

        // let mapping2 = self.fluid_mesh_uniform.map();
        
        
        //let step_size = 1.0 / (GRIDRESOLUTION - 1) as f32;
        // for x in 0..GRIDRESOLUTION{
        //     //let value_x = -0.5 + (i as f32 * step_size);
        //     for y in 0..GRIDRESOLUTION{
        //         //let value_y = -0.5 + (j as f32 * step_size);
        //         for z in 0..GRIDRESOLUTION{
        //             //let value_z = -0.5 + (k as f32 * step_size);

        //             // let x_min = -0.5 + x as f32 * step_size;
        //             // let x_max = x_min + step_size;
        //             // let y_min = -0.5 + y as f32 * step_size;
        //             // let y_max = y_min + step_size;
        //             // let z_min = -0.5 + z as f32 * step_size;
        //             // let z_max = z_min + step_size;

        //             // // Wierzchołki małego sześcianu w tablicy
        

        //             // let mut densities = [0.0f32; 8];
        //             // let mut max_density = 0.01;

        //             // for (vert_id, v) in vertices.iter().enumerate() {
        //             //     let mut new_density = 0.0f32;
        //             //     let position = nalgebra_glm::vec3(v[0], v[1], v[2]);
    
        //             //     for (_id, pos) in mapping.positions.iter().enumerate(){
        //             //         if _id >= mapping.num_of_particles as usize {
        //             //             break;
        //             //         }
        //             //         let particle_position = nalgebra_glm::vec3(pos[0], pos[1], pos[2]);
        //             //         new_density += mapping.mass * spiky_kernel(position - particle_position, 0.5);
        //             //     }
        //             //     densities[vert_id] = new_density;
        //             // };

        //              let mut positions = [[0.0f32, 0.0,0.0]; 8];
        //              let mut densities = [0.0f32; 8];

        //             for i in 0..8 {
        //                 let index = i + x*GRIDRESOLUTION + y *GRIDRESOLUTION * GRIDRESOLUTION + z * GRIDRESOLUTION* GRIDRESOLUTION *GRIDRESOLUTION;
        //                 let density = mapping.denisty_sample[index as usize];
        //                 let data = mapping.vertex_data[index];
        //                 positions[i] = [data[0], data[1], data[2]];
        //                 densities[i] = density[0];
        //                 // if density[0] > 0.0{
        //                 // println!("pos {:?} for x {x} y {y} z{z} i{i} density{}", positions[i], density[0]);
        //                 // }
        //             }

                    
                    
        //             // let positions = [
        //             //     [vertices[0][0], vertices[0][1], vertices[0][2]],
        //             //     [vertices[1][0], vertices[1][1], vertices[1][2]],
        //             //     [vertices[2][0], vertices[2][1], vertices[2][2]],
        //             //     [vertices[3][0], vertices[3][1], vertices[3][2]],
        //             //     [vertices[4][0], vertices[4][1], vertices[4][2]],
        //             //     [vertices[5][0], vertices[5][1], vertices[5][2]],
        //             //     [vertices[6][0], vertices[6][1], vertices[6][2]],
        //             //     [vertices[7][0], vertices[7][1], vertices[7][2]],
        //             // ];

        //             let mut vertex_data = vec![];

        //             let grid = Grid{
        //                 verticies: positions,
        //                 values: densities,
        //                 isolevel: iso_level
        //             };

        //             grid.create_mesh(&mut vertex_data);

        //             // let grid = GridCell{
        //             //     positions: positions,
        //             //     value: densities
        //             // };
        //             // let mc = MarchingCubes::new(isolevel, grid);
        //             // //let _tris = mc.polygonise(&mut triangles);
        //             // println!("num of triangles{}",mc.polygonise(&mut triangles));

        //             vertex_data.iter().for_each(|pos| {
        //                 self.fluid_triangles.push(Vertex{position: *pos});
        //                 //self.fluid_triangles.push(Vertex{position: triangle.positions[1]});
        //                 //self.fluid_triangles.push(Vertex{position: triangle.positions[2]});
        //             });
        //         }
        //     }
        // }

        //self.fluid_triangles.resize(GRIDRESOLUTION.pow(3) * 15, Vertex{position: [0.0, 0.0, 0.0]});
        
    }
    self.marching_cubes_cpu_time = Instant::now().duration_since(start).as_micros() as u32;
    //self.marching_cubes_cpu_time = (self.marching_cubes_cpu_time as f32 / 1_000.0) as u32;
    }

    pub fn present(&self, frame: &mut Frame, draw_parameters: &DrawParameters, proj: [[f32;4];4], view: [[f32;4];4], display: &glium::Display<WindowSurface>, draw_bounds: bool){

        let uniforms = uniform! {
                        //tex: &texture,
                        //model: model,
                        proj: proj,
                        view: view,
                    };

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        if !self.draw_cubes{
            self.fluid_mesh_buffer.write(&self.fluid_triangles);

            let uniforms_mc = uniform! {
                //tex: &texture,
                //model: model,
                proj: proj,
                view: view,
                data_in: &self.fluid_mesh_uniform
            };

            //let fluid_vertex_buffer = glium::VertexBuffer::dynamic(display, &self.fluid_triangles).unwrap();
            let _unused =frame.draw(&self.fluid_mesh_buffer, &indices, &self.fluid_program, &uniforms_mc, draw_parameters);
        }else{
        

        let mut instance_data = Vec::new();
        for i in 0..self.particles.len(){
            instance_data.push(Offset{offset: self.particles[i].position});
            //self.debug_particle_buffer.write(Offset{offset: self.particles[i].position});
        }
        for _i in self.particles.len()..MAXNUMBEROFPARTICLES{
            instance_data.push(Offset{offset: [0.0,0.0,0.0]});
        }
        self.debug_particle_buffer.write(&instance_data);
        // let uniforms = uniform! {
        //             //tex: &texture,
        //             //model: model,
        //             proj: proj,
        //             view: view,
        //         };

        let _ = frame.draw((&self.debug_vertex_buffer, self.debug_particle_buffer.per_instance().unwrap()), &indices, &self.render_program, &uniforms, draw_parameters);
        }

        if draw_bounds{

            let mut draw_wireframe_params = draw_parameters.clone();
            draw_wireframe_params.polygon_mode = PolygonMode::Line;
            // draw wireframe

            let identity = nalgebra_glm::Mat4::identity();
            
            let trans = nalgebra_glm::translate(&identity, &self.bounds_postion);
            // model is from -0.5 ro 0.5
            let scale = self.bounds_scale * 1.0;
            let scale = nalgebra_glm::scale(&identity, &scale);
            
            let mut rot = nalgebra_glm::rotate(&identity, self.bounds_rotation[0], &nalgebra_glm::vec3(1.0, 0.0, 0.0));
            rot = nalgebra_glm::rotate(&rot, self.bounds_rotation[1], &nalgebra_glm::vec3(0.0, 1.0, 0.0));
            rot = nalgebra_glm::rotate(&rot, self.bounds_rotation[2], &nalgebra_glm::vec3(0.0, 0.0, 1.0));
            
            let local_to_world = trans * rot * scale;
            let model: [[f32; 4]; 4] = local_to_world.into();

            let uniforms = uniform! {
                //tex: &texture,
                model: model,
                proj: proj,
                view: view,
            };

            let _res = frame.draw(&self.debug_vertex_buffer, &indices, &self.bounding_program, &uniforms, &draw_wireframe_params);
        }
    }
    
}

 fn spiky_kernel(distance: nalgebra_glm::Vec3, kernel_radius: f32) -> f32{
     let dist = nalgebra_glm::length(&distance);
     if dist > kernel_radius{ 
        return 0.0;
     }
     return 315.0f32/(64.0f32 * 3.1415926535897932384626433832795 as f32 * kernel_radius.powf(9.0) as f32 ) * (kernel_radius.powf(2.0) as f32  - dist.powf(2.0) as f32 ).powf(3.0) as f32;
 }
