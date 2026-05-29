pub const SHADER_SOURCE: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
};

@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv_rect: vec4<f32>,
    @location(3) viewport: vec4<f32>,
    @location(4) flags: vec4<f32>
) -> VertexOut {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );
    let corner = corners[vertex_index];
    let px = vec2<f32>(rect.x + rect.z * corner.x, rect.y + rect.w * corner.y);
    let ndc = vec2<f32>(
        (px.x / viewport.x) * 2.0 - 1.0,
        1.0 - (px.y / viewport.y) * 2.0
    );
    var out: VertexOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.color = color;
    out.uv = vec2<f32>(
        uv_rect.x + (uv_rect.z - uv_rect.x) * corner.x,
        uv_rect.y + (uv_rect.w - uv_rect.y) * corner.y
    );
    out.size = vec2<f32>(rect.z, rect.w);
    out.radius = flags.x;
    return out;
}

fn rounded_rect_sdf(p: vec2<f32>, size: vec2<f32>, r: f32) -> f32 {
    let half = size * 0.5;
    let q = abs(p - half) - half + r;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}

@fragment
fn fs_rounded(in: VertexOut) -> @location(0) vec4<f32> {
    if in.radius <= 0.5 {
        return in.color;
    }
    let d = rounded_rect_sdf(in.uv * in.size, in.size, in.radius);
    let aa = 1.0;
    let alpha = 1.0 - smoothstep(-aa, aa, d);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

@fragment
fn fs_textured(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas_texture, atlas_sampler, in.uv);
    return texel * in.color;
}
"#;
