struct Camera {
    view_proj: mat4x4<f32>,
}

struct Quad {
    model: mat4x4<f32>,
    uv_scale: vec2<f32>,
    // x: decode sRGB (imported UNORM textures), y: flip vertically.
    flags: vec2<f32>,
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
    let corner = corners[index];
    let local = vec4(corner.x - 0.5, 0.5 - corner.y, 0.0, 1.0);
    var uv = corner;
    if quad.flags.y > 0.5 {
        uv.y = 1.0 - uv.y;
    }
    var out: VertexOutput;
    out.position = camera.view_proj * quad.model * local;
    out.uv = uv * quad.uv_scale;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(quad_texture, quad_sampler, in.uv).rgb;
    if quad.flags.x > 0.5 {
        color = srgb_to_linear(color);
    }
    // Window contents are premultiplied; until blending exists, draw them
    // over black so translucent pixels stay visible but darker.
    return vec4(color, 1.0);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(high, low, c <= vec3(0.04045));
}
