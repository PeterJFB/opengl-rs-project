#version 430 core

out vec4 color;

float downscale = 8.0f;

void main()
{
    float pos = floor(gl_FragCoord[0] / downscale) + floor(gl_FragCoord[1] / downscale) + floor(gl_FragCoord[2] * 20.0f);

    float color1_flag = mod(pos, 2.0f);
    float color2_flag = - (color1_flag - 1);

    color = vec4(0.941f, 0.941f, 0.059f, 1.0f) * vec4(color1_flag, color1_flag, 1.0f, 1.0f) +
            vec4(0.960f, 0.725f, 0.259f, 1.0f) * vec4(color2_flag, color2_flag, 1.0f, 1.0f);

    color = vec4(vec3(color[0], color[1], color[2]) * (1.0f - gl_FragCoord[2]) * 1.25f - 0.9f, 1.0f);

}