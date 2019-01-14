#version 330
in vec2 f_texcoord;
out vec4 color;
uniform sampler2D tex;
void main()
{
    color = texture(tex, f_texcoord);
    if(color.a <= 0.5)
        discard;
}
