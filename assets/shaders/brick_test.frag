#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
layout(set = 2, binding = 0) uniform TimeUniform_value {
    float time;
};
void main() {
    o_Target = vec4(fract(v_color+time*0.5), 1.0);
}