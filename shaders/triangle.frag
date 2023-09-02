#version 430 core

out vec4 color;

float downscale = 20.0f;

float size = 200.0f;

void main()
{
    // ... yikes ...
    if (
        size - abs(size - gl_FragCoord[0]) > gl_FragCoord[1] // Branching! :o
    ) {
        color = vec4(0.941f, 0.941f, 0.059f, 1.0f);
    } else {
        color = vec4(0.0f, 0.0f, 0.0f, 1.0f);
    }

}