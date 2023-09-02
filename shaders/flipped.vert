#version 430 core

in vec3 position;

void main()
{
    vec3 transformed_position = position * vec3(-1.0f, -1.0f, 1.0f);

    gl_Position = vec4(transformed_position, 1.0f);
}