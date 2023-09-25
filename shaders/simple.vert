#version 430 core

in layout(location=0) vec3 position;
in layout(location=1) vec4 vertex_color;
out vec4 fragment_color;

uniform mat4x4 matrix;
uniform float time;

void main()
{
    fragment_color = vertex_color;
    gl_Position = matrix * vec4(position, 1.0f);
}