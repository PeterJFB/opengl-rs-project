#version 430 core

out vec4 color;

uniform float time;

void main()
{

    color = vec4(1.0f * (mod(time, 3.0f) / 3.0f), 1.0f, 1.0f, 1.0f);
}