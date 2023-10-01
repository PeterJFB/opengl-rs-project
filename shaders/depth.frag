#version 430 core

out vec4 color;

void main()
{
    color = vec4(vec3(1.0f, 1.0f, 1.0f) * clamp(log2(3 - gl_FragCoord[2]) / log2(3) , 0.0f, 1.0f), 1.0f);
}