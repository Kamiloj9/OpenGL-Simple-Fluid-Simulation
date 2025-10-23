#version 330 core

out vec4 color;
in vec3 _offset;

void main(){
    if(dot(_offset, _offset) == 0.0)
        discard;
    color = vec4(1.0, 1.0, 1.0, 1.0);
}