attribute vec4 a_position;
uniform vec2 u_node_position;
uniform float u_time;

void main() {
    gl_Position = vec4(u_node_position, 0.0, 0.0) + a_position;
}
