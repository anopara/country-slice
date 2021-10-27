#version 450

layout(location = 0) in vec2 _uv;
layout(location = 0) out vec4 o_Target;

void main() {

    vec2 point[5];
    point[0] = vec2(0.83,0.75);
    point[1] = vec2(0.60,0.07);
    point[2] = vec2(0.28,0.64);
    point[3] = vec2(0.31,0.26);
    point[4] = vec2(0.21,0.56);


    //float m_dist = 1.0;
    float thres = 0.1;
    float c = 0.0;

    for (int i = 0; i < 5; i++) {
        float dist = distance(_uv, point[i]);
        if ( dist < thres ) {
            c = 1.0;
        }
    }

    o_Target = vec4(c,c,c, 1.0);
}