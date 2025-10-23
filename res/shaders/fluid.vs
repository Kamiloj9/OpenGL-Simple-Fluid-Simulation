#version 430

in vec3 position;

uniform mat4 view;
uniform mat4 proj;

#define GRIDRESOLUTION 15

// Read transform from the same SSBO layout as compute shader
layout(std140, row_major) buffer buf {
    int num_of_particles;
    float gravity;
    float dumping;
    float delta_time;
    int time_steps;
    float mass;
    float gass_constant;
    float rest_density;
    float kernel_radius;
    float viscosity;
    mat4 world_to_local;
    mat4 local_to_world;
    vec4 pos[100000];
    vec4 vel[100000];
    vec4 density[100000];
    vec4 acceleration[100000];
    vec4 vertex_data[GRIDRESOLUTION * GRIDRESOLUTION*GRIDRESOLUTION*GRIDRESOLUTION];
    vec4 density_sample[GRIDRESOLUTION * GRIDRESOLUTION*GRIDRESOLUTION*GRIDRESOLUTION];
    float iso_level;
};

layout(std140) buffer buf_out {
	vec4 out_triangles[GRIDRESOLUTION * GRIDRESOLUTION * GRIDRESOLUTION * 15];
};

void main(){
    // Transform local-space surface points to world
    vec4 local = out_triangles[gl_VertexID];
    vec4 world = transpose(local_to_world) * vec4(local.xyz, 1.0);
    gl_Position = proj * view * world;
}
