//! Single WGSL shader module containing every fragment entry point used by
//! the render pipelines.
//!
//! The fragment entry points are wired to `PipelineKind` in
//! `render::wgpu::pipeline::pipeline_table()`. Keep the table in sync if you
//! add, remove, or rename a fragment entry.
//!
//! Entry summary:
//! - `vs_main` - shared vertex shader for every pipeline.
//! - `fs_main` - solid color output (SolidRect, Border, Path, TextGlyph).
//! - `fs_rounded` - rounded-rect SDF output (RoundedRect). The `0.5` cutoff
//!   here is mirrored in `render::wgpu::constants::ROUNDED_RECT_RADIUS_THRESHOLD`
//!   on the CPU side; the CPU uses it to decide which pipeline a rect goes to.
//! - `fs_linear_gradient` - two-stop linear gradient output (LinearGradient).
//! - `fs_textured` - atlas-textured color output (Image, Svg).
//!
//! All three fragment entries share `VertexOut`; the `radius` and `uv` fields
//! are only meaningful for the entries that read them.

pub const SHADER_SOURCE: &str = r#"
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) world_pos: vec2<f32>,
    @location(5) gradient: vec4<f32>,
    @location(6) gradient_end_color: vec4<f32>,
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
    @location(4) flags: vec4<f32>,
    @location(5) gradient: vec4<f32>,
    @location(6) gradient_end_color: vec4<f32>
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
    out.world_pos = px;
    out.gradient = gradient;
    out.gradient_end_color = gradient_end_color;
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
fn fs_linear_gradient(in: VertexOut) -> @location(0) vec4<f32> {
    let start = in.gradient.xy;
    let end = in.gradient.zw;
    let axis = end - start;
    let denom = dot(axis, axis);
    if denom <= 0.0001 {
        return in.color;
    }
    let t = clamp(dot(in.world_pos - start, axis) / denom, 0.0, 1.0);
    return mix(in.color, in.gradient_end_color, t);
}

@fragment
fn fs_textured(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(atlas_texture, atlas_sampler, in.uv);
    return texel * in.color;
}
"#;
