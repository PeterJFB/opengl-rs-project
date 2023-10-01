#version 430 core

in vec3 fragment_normal;
out vec4 color;

void main()
{
    color = vec4(fragment_normal, 1.0f);
}