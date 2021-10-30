#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

#ifdef STANDARDMATERIAL_NORMAL_MAP
layout(location = 3) in vec4 Vertex_Tangent;
#endif

layout(location = 0) out vec3 v_WorldPosition;
layout(location = 1) out vec3 v_WorldNormal;
layout(location = 2) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

#ifdef STANDARDMATERIAL_NORMAL_MAP
layout(location = 3) out vec4 v_WorldTangent;
#endif

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

// CUSTOM STUFF

layout(location = 4) in vec2 Curve_Uv_Pos;
layout(location = 5) in int Instance_Id;
layout(location = 6) in float Sin_Offset_Per_Row;

layout(location = 5) out vec4 v_color;

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

    float bby = Sin_Offset_Per_Row;
    float bbx = Curve_Uv_Pos.x;

    vec3 final_p = p;

    if (bby > 0.1) {
        
        float freq = fit01(random_f(bby*1000.0+SEED), 0.5, 2.5) * 10.0;
        float rand_offset = random_f(bby+SEED*1234.0)*100.0;

        float sin_wave = sin(bbx*freq + rand_offset)/2.0 * STRENGTH;

        final_p = vec3(p.x, p.y + sin_wave, p.z);
    } 

    //

    float v = fit01(random_f(float(Instance_Id)), 0.15, 0.25);
    v_color = vec4(v,v,v, 1.0);

    //

    vec4 world_position = Model * vec4(final_p, 1.0);
    v_WorldPosition = world_position.xyz;
    v_WorldNormal = mat3(Model) * Vertex_Normal;
    v_Uv = Vertex_Uv;
#ifdef STANDARDMATERIAL_NORMAL_MAP
    v_WorldTangent = vec4(mat3(Model) * Vertex_Tangent.xyz, Vertex_Tangent.w);
#endif
    gl_Position = ViewProj * world_position;
}
