#version 330
layout(location = 0) in vec2 a_pos;
layout(location = 1) in float a_pitch;
layout(location = 2) in float a_channel;
layout(location = 3) in float a_start_time;
layout(location = 4) in float a_off_time;

uniform float u_time;
uniform float u_beats; // how many beats to show
uniform float u_visual_width;

out vec4 v_color;

// on key width normalized [0.0, 1.0]
const float note_width_norm = 1.0/36.0;


vec4 get_note_color(float pitch, float channel) {
    return vec4(0.2 + (channel / 15.0), 0.2 + (pitch/35.0), 0.9, 0.7);
}

void main() {
    float elapsed = u_time - a_start_time;

    if (elapsed < 0.0) {
        gl_Position = vec4(-2.0, -2.0, 0.0, 1.0);
        v_color = vec4(0.0);
        return;
    }

    float fraction = elapsed / u_beats;

    float y_head = 1.0 - fraction * 2.0;

    float length;
    if (a_off_time > a_start_time) {
        float end_elapsed = a_off_time - a_start_time;
        float fraction_end = end_elapsed / u_beats;
        length = fraction_end * 2.0;
    } else {
        length = fraction * 2.0;
    }

    float y_tail = y_head + length;
    float draw_y = y_head + (a_pos.y + 0.5) * (y_tail - y_head);

    float x_note_pos = a_pitch / 36.0 * 2.0 - 1.0;
    float x_center = x_note_pos + note_width_norm;
    float draw_x = x_center + a_pos.x * u_visual_width;

    gl_Position = vec4(draw_x, draw_y, 0.0, 1.0);
    v_color = get_note_color(a_pitch, a_channel);
}
