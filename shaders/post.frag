#version 450

layout (location = 0) in vec2 v_texcoords;

layout (location = 0) out vec4 frag_color;

layout (location = 0) uniform sampler2D traced_img;

void main() {
    vec4 color = texture(traced_img, v_texcoords);
    frag_color = color;
    // frag_color = vec4(v_texcoords.x, v_texcoords.y, 0.5, 1.0);
}