#version 330
in vec2 f_texcoord;
out vec4 color;
uniform sampler2D tex;
uniform vec4 color_multiply;
void main()
{
    color = texture(tex, f_texcoord) * color_multiply;
}
