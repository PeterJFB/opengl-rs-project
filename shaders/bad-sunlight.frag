
#version 430 core

in vec3 fragment_normal;
in vec4 fragment_color;
out vec4 color;
vec3 light_direction = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    color = fragment_color * vec4(vec3(1.0f, 1.0f, 1.0f) * max(0, dot(fragment_normal, -light_direction)), 1.0f);
}