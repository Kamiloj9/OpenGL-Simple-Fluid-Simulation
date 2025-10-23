#version 330 core

in vec3 position;

in vec3 offset;
out vec3 _offset;

uniform mat4 view;
uniform mat4 proj;

mat4 scale(float scaleX, float scaleY, float scaleZ) {
    return mat4(
        vec4(scaleX, 0.0, 0.0, 0.0),
        vec4(0.0, scaleY, 0.0, 0.0),
        vec4(0.0, 0.0, scaleZ, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}

mat4 translationMatrix(vec3 translation) {
    return mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(translation, 1.0)
    );
}

void main(){
    _offset = offset;
    gl_Position = proj * view * translationMatrix(offset) * scale(0.1, 0.1, 0.1) * vec4(position, 1.0);
}