#version 450
out vec4 o_Target;
in vec2 v_Uv;
in vec3 v_pos_ws;

uniform sampler2D ourTexture;

float sample_texture_ws(vec2 pos_ws) {
    vec2 texture_uv = (pos_ws / 20.0 + 0.5);
    return texture(ourTexture, texture_uv).x;
}


float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main() {



    float v = 1.0 - max(fit01(v_Uv.y, -0.3, 1.0), 0.0);

    // check if we are overlapping with the road
    float road_value = sample_texture_ws(v_pos_ws.xz);
    v = v * (1.0 - pow(min(road_value*2.0, 1.0), 2.0));

    float alpha = pow(v, 3.0);
    o_Target = vec4(0.0, 0.0, 0.0, alpha * 0.7);

    // DUMMY WIREFRAME
    
    /*
    float THICKNESS = 0.05;

    float alpha = 0.0;
    if (v_Uv.x < THICKNESS || v_Uv.x > 1.0 - THICKNESS || v_Uv.y < THICKNESS || v_Uv.y > 1.0 - THICKNESS) {
        alpha = 1.0;
    } 


    o_Target = vec4(1.0, 0.0, 0.0, alpha);
    */
    
}