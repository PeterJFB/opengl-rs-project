#version 430 core

out vec4 color;

void main()
{
    color = vec4(vec3(1.0f, 1.0f, 1.0f) * (gl_FragCoord[2]) * 0.8f + 0.2f, 1.0f);
}