#version 330 core

//in vec2 tex_coords;
in vec3 position;
in vec3 normal;
//in vec3 color;
//out vec3 vertex_color;
//out vec2 v_tex_coords;
out vec3 _normal;

uniform mat4 model;
uniform mat4 proj;
uniform mat4 view;

void main(){
    //v_tex_coords = tex_coords;
    //vertex_color = color;
    //gl_Position = matrix * vec4(position, 0.0, 1.0);
    gl_Position = proj * view * model * vec4(position, 1.0);
    _normal = transpose(inverse(mat3(view * model))) * normal;
}