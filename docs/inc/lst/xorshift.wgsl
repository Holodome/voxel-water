@group(0) @binding(1) var<uniform> random_seed: u32;
var<private> rng_state: u32;

fn xorshift32(state: u32) -> u32 {
    var x = state;
    x ^= x << 13u;
    x ^= x >> 17u;
    x ^= x << 5u;
    return x;
}

fn random_u32() -> u32 {
    let x = xorshift32(rng_state);
    rng_state = x;
    return x;
}

fn random_f32() -> f32 {
    let u = random_u32();
    return f32(u) * bitcast<f32>(0x2F800000u);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    rng_state = xorshift32(bitcast<u32>(in.uv.x * 123456789.0 + in.uv.y) ^ random_seed);
    ...
