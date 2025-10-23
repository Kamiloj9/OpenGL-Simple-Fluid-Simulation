#version 430

in vec3 position;

uniform mat4 view;
uniform mat4 proj;

#define GRIDRESOLUTION 10

layout(std140) buffer buf_out {
	vec4 out_triangles[GRIDRESOLUTION * GRIDRESOLUTION * GRIDRESOLUTION * 15];
};

void main(){
    gl_Position = proj * view * vec4(out_triangles[gl_VertexID].xyz, 1.0);
}