#version 450
out vec4 o_Target;
in vec2 v_Uv;


float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main() {



    float v = 1.0 - max(fit01(v_Uv.y, -0.6, 1.0), 0.0);

    float alpha = pow(v, 2.0);
    o_Target = vec4(0.0, 0.0, 0.0, alpha * 0.6);
    
    

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