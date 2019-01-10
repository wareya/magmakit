#version 330
in vec2 position;
out vec2 v_tex_coords;
uniform mat4 matrix_view;
uniform mat4 matrix_command;
uniform mat4 matrix_sprite;
void main()
{
    v_tex_coords = position;
    gl_Position = matrix_view * matrix_command * vec4(position, 0.0, 1.0);
}
