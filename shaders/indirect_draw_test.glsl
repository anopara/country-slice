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

float curve_idx_to_roadmask_value(int i, int idx, ivec2 dims) {
    ivec2 pixel_coord = ws_pos_to_pixel_coord(curves[idx].positions[i].xyz, dims);
    return imageLoad(road_mask, pixel_coord).x;
}

void main() {

    ivec2 dims = imageSize(road_mask);
    const uint idx = gl_GlobalInvocationID.x;

    // check whether points are above the road
    int arch_points = 0;
    for (int i; i<curves[idx].points_count; i++) {
        float pixel = curve_idx_to_roadmask_value(i, idx, dims);

        if (pixel > 0) {
            arch_points += 1;
        }
    }
    // TODO: need to take into account the points that is before the arch, and the point after (if it exists) -- to make sure the points start and end at Y 0
    // that would be our curve chunk, we need to know its length
    // per line segment, we want to know its length and divide into N random bricks (this way we know how many bricks we actually need in total)

    uint instance_offset = atomicAdd(cmds[0].instanceCount, arch_points); //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/atomicAdd.xhtml 

   
    // now we actually want to march through the valid segments and write the transform data

    // first version can just be uniform size bricks

    for (int i; i<curves[idx].points_count; i++) {
        vec4 pt_position = curves[idx].positions[i];

        ivec2 pixel_coord = ws_pos_to_pixel_coord(pt_position.xyz, dims);
        vec4 pixel = imageLoad(road_mask, pixel_coord);

        if (pixel.x > 0) {
            transforms[instance_offset+i] = transpose(mat4(
                0.1, 0.0, 0.0, pt_position.x,
                0.0, 0.1, 0.0, pt_position.y + pow(pixel.x, 0.3),
                0.0, 0.0, 0.1, pt_position.z,
                0.0, 0.0, 0.0, 1.0
            ));
        }
    }
}  