attribute vec2 vert_position;
attribute vec4 vert_color;
uniform   mat4 projection;
varying   vec4 frag_color;

void main() {
    gl_Position = projection * vec4(vert_position, 0, 1);
    frag_color = vert_color;
}
