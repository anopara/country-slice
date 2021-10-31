#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec2 v_Uv;

void main() {
    float alpha =  1.0 - pow(v_Uv.y, 2.0);
    o_Target = vec4(0.0, 0.0, 0.0, alpha*0.7);
}