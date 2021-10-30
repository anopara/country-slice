#version 450
layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

// Custom attribs
layout(location = 1) in int Instance_Id;
layout(location = 2) in vec2 Curve_Uv_Pos;

layout(location = 0) out vec3 v_color;

float random_f(float x) {
    return fract(sin(x*12.9898) * 43758.5453);
}

float fit01(float x, float min, float max) {
    return x * (max-min) + min;
}

void main() {

    vec3 p = Vertex_Position;

    float WALL_HEIGHT = 1.4;
    float SEED = 112.0;
    float STRENGTH = 0.075;

    // Add wavery pattern

    float bby = Curve_Uv_Pos.y; 
    float bbx = Curve_Uv_Pos.x;

    vec3 final_p = p;

    if (bby > 0.1) {
        
        float freq = fit01(random_f(bby*1000.0+SEED), 0.5, 3.5) * 10.0;
        float rand_offset = bby*100.0*SEED;

        float sin_wave = sin(bbx*freq + rand_offset)/2.0 * STRENGTH;

        final_p = vec3(p.x, p.y + sin_wave, p.z);
    } 
    

    // OUT-------------

    float v = fit01(random_f(float(Instance_Id)), 0.15, 0.35);
    v_color = vec3(v,v,v);

    gl_Position = ViewProj * Model * vec4(final_p, 1.0);
}


