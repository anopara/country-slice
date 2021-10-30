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

layout(location = 0) out vec3 v_color;

float random_f(float x) {
    return fract(sin(x*12.9898) * 43758.5453);
}

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);

    float v = random_f(float(Instance_Id));
    v_color = vec3(v,v,v);
}

