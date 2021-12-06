#version 430
layout(local_size_x = 1, local_size_y = 1) in;

// Same as the OpenGL defined struct: DrawElementsIndirectCommand
struct DrawCommand {
    uint count;         // Num elements (vertices)
    uint instanceCount; // Number of instances to draw (a.k.a primcount)
    uint firstIndex;    // Specifies a byte offset (cast to a pointer type) into the buffer bound to GL_ELEMENT_ARRAY_BUFFER to start reading indices from.
    uint baseVertex;    // Specifies a constant that should be added to each element of indices​ when chosing elements from the enabled vertex arrays.
    uint baseInstance;  // Specifies the base instance for use in fetching instanced vertex attributes.
};

// Command buffer backed by a Shader Storage Object Buffer (SSBO)
layout(std140, binding = 0) writeonly buffer draw_commands {
    DrawCommand cmds[];
};

layout (std430, binding=2) buffer transforms_buffer { 
    mat4 transforms[];
};

struct CurveData {
    uint points_count;
    uint pad0;
    uint pad1;
    uint pad2;
    vec4 positions[1000];
};

layout (std430, binding=3) buffer curves_buffer { 
    CurveData curves[];
};

layout(rgba32f) uniform image2D road_mask;

ivec2 ws_pos_to_pixel_coord(vec3 ws_pos, ivec2 img_dims) {
    // the img is from -10 to 10 in world space TODO: this should be a uniform coming from a struct on CPU side...
    vec2 texture_uv = (ws_pos / 20.0 + 0.5).xz;
    return ivec2(texture_uv.x * img_dims.x, texture_uv.y * img_dims.y);
}

void main() {

    ivec2 dims = imageSize(road_mask);
    const uint idx = gl_GlobalInvocationID.x;

    uint instance_offset = atomicAdd(cmds[0].instanceCount, curves[idx].points_count); //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/atomicAdd.xhtml 

    for (int i; i<curves[idx].points_count; i++) {
        vec4 pt_position = curves[idx].positions[i];

        ivec2 pixel_coord = ws_pos_to_pixel_coord(pt_position.xyz, dims);
        vec4 pixel = imageLoad(road_mask, pixel_coord);

        transforms[instance_offset+i] = transpose(mat4(
            0.1, 0.0, 0.0, pt_position.x,
            0.0, 0.1, 0.0, pt_position.y + pow(pixel.x, 0.3),
            0.0, 0.0, 0.1, pt_position.z,
            0.0, 0.0, 0.0, 1.0
        ));
    }
}  