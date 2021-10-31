#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec2 v_Uv;


float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main() {

    float v = 1.0 - max(fit01(v_Uv.y, -0.6, 1.0), 0.0);

    float alpha = pow(v, 2.0);
    o_Target = vec4(0.0, 0.0, 0.0, alpha * 0.6);
}