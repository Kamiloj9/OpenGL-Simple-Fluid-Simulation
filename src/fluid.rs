use std::time::Instant;

use glium::{Api, Frame, Surface, DrawParameters, Program, Version, glutin::surface::WindowSurface, uniform, implement_vertex, VertexBuffer, PolygonMode, program::ComputeShader, uniforms::{self, AsUniformValue, UniformValue, UniformBlock}};
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
    compute_supported: bool,
    pub particles: Vec<Particle>,
    render_program: Program,
    bounding_program: Program,
    fluid_program: Program,
    compute_shader: Option<ComputeShader>,
    density_compute_shader: Option<ComputeShader>,
    compute_fluid_mesh_shader: Option<ComputeShader>,
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
    fn as_uniform_value(&self) -> UniformValue<'_> {
        panic!("not implemented")
    }
}

impl AsUniformValue for Data {
    fn as_uniform_value(&self) -> UniformValue<'_> {
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
        let version = *display.get_opengl_version();
        let compute_supported = version >= Version(Api::Gl, 4, 3) || version >= Version(Api::GlEs, 3, 1);

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

        let compute_shader = compute_supported.then(|| ComputeShader::from_source(display, compute_shader_src).unwrap());
        let density_compute_shader = compute_supported.then(|| ComputeShader::from_source(display, density_compute_shader_src).unwrap());
        let compute_fluid_mesh_shader = compute_supported.then(|| ComputeShader::from_source(display, fluid_shader_compute_src).unwrap());

        Self{
            draw_cubes: false,
            compute_supported,
            particles: Vec::new(),
            render_program: glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap(),
            bounding_program: glium::Program::from_source(display, vertex_shader_bounds_src, fragment_shader_bounds_src, None).unwrap(),
            fluid_program: glium::Program::from_source(display, fluid_vertex_shader_src, fluid_fragment_shader_src, None).unwrap(),
            debug_vertex_buffer: vertex_buffer,
            bounds_postion: bounds_position.clone(),
            bounds_scale: bounds_scale.clone(),
            bounds_rotation: bounds_rotation.clone(),
            compute_shader,
            density_compute_shader,
            compute_fluid_mesh_shader,
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

    pub fn compute_supported(&self) -> bool {
        self.compute_supported
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

    fn compute_particle_densities(
        &self,
        kernel_radius: f32,
        particle_mass: f32,
        gass_constant: f32,
        rest_density: f32,
    ) -> Vec<[f32; 4]> {
        let dt = 1.0 / 160.0;
        let mut densities = vec![[0.0; 4]; self.particles.len()];

        for (particle_id, particle) in self.particles.iter().enumerate() {
            let position = nalgebra_glm::vec3(particle.position[0], particle.position[1], particle.position[2]);
            let mut new_density = 0.0;
            let mut new_near_density = 0.0;

            for other in &self.particles {
                let predicted_position = nalgebra_glm::vec3(
                    other.position[0] + other.velocity[0] * dt,
                    other.position[1] + other.velocity[1] * dt,
                    other.position[2] + other.velocity[2] * dt,
                );

                let diff = position - predicted_position;
                let distance_squared = nalgebra_glm::dot(&diff, &diff);
                if distance_squared < kernel_radius * kernel_radius {
                    let distance = distance_squared.sqrt();
                    new_density += spiky_kernel(distance, kernel_radius);
                    new_near_density += spiky_kernel_near(distance, kernel_radius);
                }
            }

            new_density *= particle_mass;
            new_near_density *= particle_mass;
            let pressure = gass_constant * (new_density - rest_density);
            densities[particle_id] = [new_density, pressure, new_near_density, pressure];
        }

        densities
    }

    fn simulate_particles_cpu(
        &mut self,
        time_step: f32,
        num_of_steps: u32,
        world_to_local: &nalgebra_glm::Mat4,
        local_to_world: &nalgebra_glm::Mat4,
        kernel_radius: f32,
        particle_mass: f32,
        viscosity: f32,
        densities: &[[f32; 4]],
    ) {
        let dt = if num_of_steps == 0 {
            1.0 / 160.0
        } else {
            (time_step / num_of_steps as f32).max(1.0 / 160.0)
        };
        let gravity_force = nalgebra_glm::vec3(0.0, GRAVITY, 0.0);

        for _ in 0..num_of_steps.max(1) {
            let prev_positions: Vec<_> = self
                .particles
                .iter()
                .map(|particle| nalgebra_glm::vec3(particle.position[0], particle.position[1], particle.position[2]))
                .collect();
            let prev_velocities: Vec<_> = self
                .particles
                .iter()
                .map(|particle| nalgebra_glm::vec3(particle.velocity[0], particle.velocity[1], particle.velocity[2]))
                .collect();
            let prev_accelerations: Vec<_> = self
                .particles
                .iter()
                .map(|particle| nalgebra_glm::vec3(particle.acceleration[0], particle.acceleration[1], particle.acceleration[2]))
                .collect();

            for particle_id in 0..self.particles.len() {
                let position = prev_positions[particle_id];
                let velocity = prev_velocities[particle_id];
                let acceleration = prev_accelerations[particle_id];

                let mut pressure_force = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                let mut viscosity_force = nalgebra_glm::vec3(0.0, 0.0, 0.0);
                let surface_force = nalgebra_glm::vec3(0.0, 0.0, 0.0);

                for other_id in 0..self.particles.len() {
                    if other_id == particle_id {
                        continue;
                    }

                    let dist = position - prev_positions[other_id];
                    let len = nalgebra_glm::length(&dist);
                    if len > kernel_radius {
                        continue;
                    }

                    if len == 0.0 {
                        pressure_force -= nalgebra_glm::vec3(1.0, 0.0, 0.0);
                    } else if densities[other_id][0] > 0.01 {
                        let pressure_scalar = (densities[other_id][1] + densities[particle_id][1]) / (2.0 * densities[other_id][0]);
                        pressure_force -= pressure_kernel_derivative(dist, kernel_radius) * pressure_scalar;

                        let near_pressure_scalar =
                            (densities[other_id][3] + densities[particle_id][3]) / (2.0 * densities[other_id][2].max(0.01));
                        pressure_force -= pressure_near_kernel_derivative(dist, kernel_radius) * near_pressure_scalar;
                    }

                    if len != 0.0 && densities[other_id][0] > 0.01 {
                        let laplacian = viscosity_laplacian(dist, kernel_radius);
                        let mut scalar = viscosity * particle_mass * laplacian / densities[other_id][0];
                        scalar += viscosity * particle_mass * laplacian / densities[other_id][2].max(0.01);
                        viscosity_force += (prev_velocities[other_id] - velocity) * scalar;
                    }
                }

                let new_acceleration = gravity_force * time_step / dt + pressure_force + viscosity_force + surface_force;
                let vel_half_step = velocity + acceleration * dt * 0.5 * DUMPING;
                let mut next_pos = position + vel_half_step * dt * 0.9;
                let mut next_vel = vel_half_step + new_acceleration * dt * 0.5;

                let mut local_position = world_to_local * nalgebra_glm::vec4(next_pos.x, next_pos.y, next_pos.z, 1.0);
                let mut local_velocity = world_to_local * nalgebra_glm::vec4(next_vel.x, next_vel.y, next_vel.z, 0.0);

                for axis in 0..3 {
                    if local_position[axis].abs() >= 0.5 {
                        local_position[axis] = if local_position[axis] >= 0.5 { 0.5 } else { -0.5 };
                        local_velocity[axis] *= -DUMPING;
                    }
                }

                let world_position =
                    local_to_world * nalgebra_glm::vec4(local_position.x, local_position.y, local_position.z, 1.0);
                let world_velocity =
                    local_to_world * nalgebra_glm::vec4(local_velocity.x, local_velocity.y, local_velocity.z, 0.0);
                next_pos = nalgebra_glm::vec3(world_position.x, world_position.y, world_position.z);
                next_vel = nalgebra_glm::vec3(world_velocity.x, world_velocity.y, world_velocity.z);

                self.particles[particle_id].position = [next_pos.x, next_pos.y, next_pos.z];
                self.particles[particle_id].velocity = [next_vel.x, next_vel.y, next_vel.z];
                self.particles[particle_id].acceleration = [new_acceleration.x, new_acceleration.y, new_acceleration.z];
            }
        }
    }

    fn generate_surface_cpu(
        &mut self,
        local_to_world: &nalgebra_glm::Mat4,
        kernel_radius: f32,
        particle_mass: f32,
        iso_level: f32,
    ) {
        let step_size = 1.0 / (GRIDRESOLUTION - 1) as f32;
        let mut triangles = Vec::with_capacity(GRIDRESOLUTION.pow(3) * 15);

        for x in 0..(GRIDRESOLUTION - 1) {
            for y in 0..(GRIDRESOLUTION - 1) {
                for z in 0..(GRIDRESOLUTION - 1) {
                    let local_positions = cube_corner_positions(x, y, z, step_size);
                    let mut world_positions = [[0.0f32; 3]; 8];
                    let mut densities = [0.0f32; 8];

                    for (i, local_position) in local_positions.iter().enumerate() {
                        let world = local_to_world
                            * nalgebra_glm::vec4(local_position[0], local_position[1], local_position[2], 1.0);
                        let world_pos = [world.x, world.y, world.z];
                        world_positions[i] = world_pos;
                        densities[i] = self.sample_density_world(world_pos, kernel_radius, particle_mass);
                    }

                    let grid = Grid {
                        verticies: world_positions,
                        values: densities,
                        isolevel: iso_level,
                    };
                    grid.create_mesh(&mut triangles);
                }
            }
        }

        self.fluid_triangles.fill(Vertex { position: [0.0, 0.0, 0.0] });
        for (dst, src) in self.fluid_triangles.iter_mut().zip(triangles.iter()) {
            *dst = Vertex { position: *src };
        }
    }

    fn sample_density_world(&self, sample_position: [f32; 3], kernel_radius: f32, particle_mass: f32) -> f32 {
        let sample = nalgebra_glm::vec3(sample_position[0], sample_position[1], sample_position[2]);
        self.particles
            .iter()
            .map(|particle| {
                let particle_position = nalgebra_glm::vec3(particle.position[0], particle.position[1], particle.position[2]);
                particle_mass * spiky_kernel(nalgebra_glm::distance(&sample, &particle_position), kernel_radius)
            })
            .sum()
    }

    pub fn simulate(&mut self, time_step: f32, num_of_steps: u32, _display: &glium::Display<WindowSurface>
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
        
        if self.compute_supported {
            let uniforms = uniform! {
                buf: &*self.buffer,
                buf_out: &*self.fluid_mesh_uniform
            };

            let start = Instant::now();
            self.density_compute_shader.as_ref().unwrap().execute(uniforms, self.particles.len() as u32, 1, 1);
            self.density_stage_time = Instant::now().duration_since(start).as_micros() as u32;

            let start = Instant::now();
            self.compute_shader.as_ref().unwrap().execute(uniforms, self.particles.len() as u32, 1, 1);
            self.simulation_stage_time = Instant::now().duration_since(start).as_micros() as u32;

            let start = Instant::now();
            if !self.draw_cubes {
                self.compute_fluid_mesh_shader.as_ref().unwrap().execute(uniforms, GRIDRESOLUTION as u32, GRIDRESOLUTION as u32, GRIDRESOLUTION as u32);
            }
            self.marching_cubes_compute_stage_time = Instant::now().duration_since(start).as_micros() as u32;

            let mapping = self.buffer.map();

            for (i, pos) in mapping.positions.iter().enumerate() {
                if i >= self.particles.len() {
                    break;
                }
                self.particles[i].position = [pos[0], pos[1], pos[2]];
            }
            for (i, vel) in mapping.velocities.iter().enumerate() {
                if i >= self.particles.len() {
                    break;
                }
                self.particles[i].velocity = [vel[0], vel[1], vel[2]];
            }
            for (i, acc) in mapping.acceleration.iter().enumerate() {
                if i >= self.particles.len() {
                    break;
                }
                self.particles[i].acceleration = [acc[0], acc[1], acc[2]];
            }

            let start = Instant::now();
            if !self.draw_cubes {
                let mesh_mapping = self.fluid_mesh_uniform.map();
                for (dst, src) in self.fluid_triangles.iter_mut().zip(mesh_mapping.out_triangles.iter()) {
                    *dst = Vertex { position: [src[0], src[1], src[2]] };
                }
            }
            self.marching_cubes_cpu_time = Instant::now().duration_since(start).as_micros() as u32;
        } else {
            let start = Instant::now();
            let densities = self.compute_particle_densities(kernel_radius, particle_mass, gass_constant, rest_density);
            self.density_stage_time = Instant::now().duration_since(start).as_micros() as u32;

            let start = Instant::now();
            self.simulate_particles_cpu(
                time_step,
                num_of_steps,
                &world_to_local,
                &local_to_world,
                kernel_radius,
                particle_mass,
                viscosity,
                &densities,
            );
            self.simulation_stage_time = Instant::now().duration_since(start).as_micros() as u32;
            self.marching_cubes_compute_stage_time = 0;

            let start = Instant::now();
            if !self.draw_cubes {
                self.generate_surface_cpu(&local_to_world, kernel_radius, particle_mass, iso_level);
            } else {
                self.fluid_triangles.fill(Vertex { position: [0.0, 0.0, 0.0] });
            }
            self.marching_cubes_cpu_time = Instant::now().duration_since(start).as_micros() as u32;
        }
    }

    pub fn present(&self, frame: &mut Frame, draw_parameters: &DrawParameters, proj: [[f32;4];4], view: [[f32;4];4], _display: &glium::Display<WindowSurface>, draw_bounds: bool, debug_surface_points: bool, surface_wireframe: bool, surface_color: [f32;3], surface_alpha: f32){

        let uniforms = uniform! {
                        //tex: &texture,
                        //model: model,
                        proj: proj,
                        view: view,
                    };

        let indices_tri = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let indices_points = glium::index::NoIndices(glium::index::PrimitiveType::Points);

        if !self.draw_cubes{
            self.fluid_mesh_buffer.write(&self.fluid_triangles);

            let color_vec = [surface_color[0], surface_color[1], surface_color[2], surface_alpha];
            let uniforms_mc = uniform! {
                proj: proj,
                view: view,
                color: color_vec
            };

            let mut params = draw_parameters.clone();
            if surface_wireframe {
                params.polygon_mode = PolygonMode::Line;
            }
            // enable alpha blending for surface
            params.blend = glium::Blend::alpha_blending();
            if debug_surface_points {
                params.point_size = Some(3.0);
                let _ = frame.draw(&self.fluid_mesh_buffer, &indices_points, &self.fluid_program, &uniforms_mc, &params);
            } else {
                let _ = frame.draw(&self.fluid_mesh_buffer, &indices_tri, &self.fluid_program, &uniforms_mc, &params);
            }
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

        let _ = frame.draw((&self.debug_vertex_buffer, self.debug_particle_buffer.per_instance().unwrap()), &indices_tri, &self.render_program, &uniforms, draw_parameters);
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

            let _res = frame.draw(&self.debug_vertex_buffer, &indices_tri, &self.bounding_program, &uniforms, &draw_wireframe_params);
        }
    }
    
}

fn cube_corner_positions(x: usize, y: usize, z: usize, step_size: f32) -> [[f32; 3]; 8] {
    let x_min = -0.5 + x as f32 * step_size;
    let x_max = x_min + step_size;
    let y_min = -0.5 + y as f32 * step_size;
    let y_max = y_min + step_size;
    let z_min = -0.5 + z as f32 * step_size;
    let z_max = z_min + step_size;

    [
        [x_min, y_min, z_min],
        [x_max, y_min, z_min],
        [x_max, y_max, z_min],
        [x_min, y_max, z_min],
        [x_min, y_min, z_max],
        [x_max, y_min, z_max],
        [x_max, y_max, z_max],
        [x_min, y_max, z_max],
    ]
}

fn spiky_kernel(dist: f32, kernel_radius: f32) -> f32 {
    if dist > kernel_radius {
        return 0.0;
    }
    315.0 / (64.0 * std::f32::consts::PI * kernel_radius.powf(9.0)) * (kernel_radius.powf(2.0) - dist.powf(2.0)).powf(3.0)
}

fn spiky_kernel_near(dist: f32, kernel_radius: f32) -> f32 {
    if dist > kernel_radius {
        return 0.0;
    }

    let scale = 15.0 / (std::f32::consts::PI * kernel_radius.powf(6.0));
    let value = kernel_radius - dist;
    value * value * value * scale
}

fn pressure_kernel_derivative(distance: nalgebra_glm::Vec3, kernel_radius: f32) -> nalgebra_glm::Vec3 {
    let len = nalgebra_glm::length(&distance);
    if len == 0.0 || len > kernel_radius {
        return nalgebra_glm::vec3(0.0, 0.0, 0.0);
    }

    let value = (-45.0 * (kernel_radius - len).powf(2.0)) / (std::f32::consts::PI * kernel_radius.powf(6.0) * len);
    distance * value
}

fn pressure_near_kernel_derivative(distance: nalgebra_glm::Vec3, kernel_radius: f32) -> nalgebra_glm::Vec3 {
    let len = nalgebra_glm::length(&distance);
    if len == 0.0 || len > kernel_radius {
        return nalgebra_glm::vec3(0.0, 0.0, 0.0);
    }

    let scale = 45.0 / (kernel_radius.powf(6.0) * std::f32::consts::PI);
    let value = -(kernel_radius - len).powf(2.0) * scale;
    distance * value
}

fn viscosity_laplacian(distance: nalgebra_glm::Vec3, kernel_radius: f32) -> f32 {
    let len = nalgebra_glm::length(&distance);
    if len == 0.0 {
        return 0.0;
    }

    (-45.0 * (len.powf(2.0) - kernel_radius * len)) / (kernel_radius.powf(6.0) * std::f32::consts::PI * len)
}
