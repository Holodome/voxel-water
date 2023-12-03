struct VertexInput {
    @location(0) uv: vec2f
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) blur_uv0: vec2f,
    @location(1) blur_uv1: vec2f,
    @location(2) blur_uv2: vec2f,
    @location(3) blur_uv3: vec2f,
    @location(4) blur_uv4: vec2f,
    @location(5) blur_uv5: vec2f,
    @location(6) blur_uv6: vec2f,
    @location(7) blur_uv7: vec2f,
    @location(8) blur_uv8: vec2f,
    @location(9) blur_uv9: vec2f,
    @location(10) blur_uv10: vec2f
};

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(0) var samp: sampler;
@group(1) @binding(1) var<uniform> target_size: vec2f;

@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = (in.uv * 2.0) - vec2f(1.0);
    out.pos = vec4f(pos, 0.0, 1.0);
    let center_uv = (pos + vec2f(1.0)) * 0.5;
    let px_size = 1.0 / target_size.x;

    out.blur_uv0 = center_uv + vec2f(px_size * f32(-5), 0.0);
    out.blur_uv1 = center_uv + vec2f(px_size * f32(-4), 0.0);
    out.blur_uv2 = center_uv + vec2f(px_size * f32(-3), 0.0);
    out.blur_uv3 = center_uv + vec2f(px_size * f32(-2), 0.0);
    out.blur_uv4 = center_uv + vec2f(px_size * f32(-1), 0.0);
    out.blur_uv5 = center_uv + vec2f(px_size * f32(0), 0.0);
    out.blur_uv6 = center_uv + vec2f(px_size * f32(1), 0.0);
    out.blur_uv7 = center_uv + vec2f(px_size * f32(2), 0.0);
    out.blur_uv8 = center_uv + vec2f(px_size * f32(3), 0.0);
    out.blur_uv9 = center_uv + vec2f(px_size * f32(4), 0.0);
    out.blur_uv10 = center_uv + vec2f(px_size * f32(5), 0.0);

    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var out_color = vec4f(0.0);
    out_color += textureSample(tex, samp, in.blur_uv0) * 0.0093;
    out_color += textureSample(tex, samp, in.blur_uv1) * 0.028002;
    out_color += textureSample(tex, samp, in.blur_uv2) * 0.065984;
    out_color += textureSample(tex, samp, in.blur_uv3) * 0.121703;
    out_color += textureSample(tex, samp, in.blur_uv4) * 0.175713;
    out_color += textureSample(tex, samp, in.blur_uv5) * 0.198596;
    out_color += textureSample(tex, samp, in.blur_uv6) * 0.175713;
    out_color += textureSample(tex, samp, in.blur_uv7) * 0.121703;
    out_color += textureSample(tex, samp, in.blur_uv8) * 0.065984;
    out_color += textureSample(tex, samp, in.blur_uv9) * 0.028002;
    out_color += textureSample(tex, samp, in.blur_uv10) * 0.0093;
    return out_color;
}
