#version 460 core
out vec4 FragColor;

layout(location = 0) in vec2 uv;
layout(location = 1) uniform sampler2D tex;

void main() {
    FragColor = texture(tex, uv);
}
