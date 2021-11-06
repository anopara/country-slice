#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec3 v_Position;


layout(set = 2, binding = 0) uniform MousePosition_x {
    float x;
};

layout(set = 3, binding = 0) uniform MousePosition_z {
    float z;
};


void main() {

    float dist = distance(v_Position, vec3(x, 0.0, z));

    o_Target = vec4(dist, dist, dist, 1.0);
}