struct Item {
    // Destination in NDC: left, top, right, bottom.
    rect: vec4<f32>,
    // Source in UV: left, top, right, bottom.
    uv: vec4<f32>,
    // x: decode sRGB, y: flip vertically, z: force opaque.
    flags: vec4<f32>,
}

@group(0) @binding(0) var<uniform> item: Item;
@group(0) @binding(1) var item_texture: texture_2d<f32>;
@group(0) @binding(2) var item_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(1.0, 0.0),
    );
    let corner = corners[index];
    var v = corner.y;
    if item.flags.y > 0.5 {
        v = 1.0 - v;
    }
    var out: VertexOutput;
    out.position = vec4(mix(item.rect.xy, item.rect.zw, corner), 0.0, 1.0);
    out.uv = vec2(mix(item.uv.x, item.uv.z, corner.x), mix(item.uv.y, item.uv.w, v));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(item_texture, item_sampler, in.uv);
    if item.flags.x > 0.5 {
        color = vec4(srgb_to_linear(color.rgb), color.a);
    }
    if item.flags.z > 0.5 {
        color.a = 1.0;
    }
    // Premultiplied alpha, blended by the pipeline.
    return color;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(high, low, c <= vec3(0.04045));
}
