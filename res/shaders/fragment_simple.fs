#version 330 core

//in vec3 vertex_color;
//in vec2 v_tex_coords;
in vec3 _normal;
out vec4 color;

//uniform sampler2D tex;

void main() {
    vec3 light_dir = vec3(-5.0, 0.0, 0.0);
    float brightness = max(dot(normalize(_normal), normalize(light_dir)), 0);
    vec3 dark_color = vec3(0.6, 0.0, 0.0);
    vec3 regular_color = vec3(1.0, 0.0, 0.0);
    color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}