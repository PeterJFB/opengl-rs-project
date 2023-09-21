#version 430 core

in layout(location=0) vec3 position;
in layout(location=1) vec4 vertex_color;
out vec4 fragment_color;

mat4x4 matrix = mat4(1);
uniform float time;

void main()
{
    // matrix[0][0] = mod(time, 3.0f) / 3.0f; // a
    // matrix[1][0] = mod(time, 3.0f) / 3.0f; // b
    // matrix[3][0] = mod(time, 3.0f) / 3.0f; // c
    // matrix[0][1] = mod(time, 3.0f) / 3.0f; // d
    // matrix[1][1] = mod(time, 3.0f) / 3.0f; // e
    // matrix[3][1] = mod(time, 3.0f) / 3.0f; // f
    fragment_color = vertex_color;
    gl_Position = matrix * vec4(position, 1.0f);
}