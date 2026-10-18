struct Camera {
    view_proj: mat4x4<f32>,
}

struct Quad {
    model: mat4x4<f32>,
    uv_scale: vec2<f32>,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> quad: Quad;
@group(1) @binding(1) var quad_texture: texture_2d<f32>;
@group(1) @binding(2) var quad_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Unit quad in the XY plane, centered at the origin, facing +Z.
// UV (0, 0) is the top-left corner, matching buffer memory order.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(1.0, 0.0),
    );
    let uv = corners[index];
    let local = vec4(uv.x - 0.5, 0.5 - uv.y, 0.0, 1.0);
    var out: VertexOutput;
    out.position = camera.view_proj * quad.model * local;
    out.uv = uv * quad.uv_scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(quad_texture, quad_sampler, in.uv);
    // Window contents are premultiplied; until blending exists, draw them
    // over black so translucent pixels stay visible but darker.
    return vec4(color.rgb, 1.0);
}
