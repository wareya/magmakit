#version 330
in vec2 position;
out vec2 f_texcoord;
uniform mat4 matrix_view;
uniform mat4 matrix_command;
uniform vec2 tex_topleft;
uniform vec2 tex_bottomright;
void main()
{
    f_texcoord = (position+tex_topleft) * (tex_bottomright-tex_topleft);
    gl_Position = matrix_view * matrix_command * vec4(position, 0.0, 1.0);
}
