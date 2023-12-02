const pi = 3.14159265359;
const two_pi = 6.28318530718;

const MAT_DIFFUSE: i32 = 0;
const MAT_METAL: i32 = 1;
const MAT_DIELECTRIC: i32 = 2;

const VOXEL_SIZE: f32 = 0.5;
const MAXIMUM_TRAVERSAL_DISTANCE: i32 = 128;
const MAX_BOUNCE_COUNT: i32 = 4;

struct Onb {
    u: vec3f, 
    v: vec3f, 
    w: vec3f
};

struct VertexInput {
    @location(0) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) ray_direction: vec3f,
    @location(2) ray_origin: vec3f
};

struct FragmentOutput {
    @location(0) color: vec4f,
    @location(1) normal: vec4f,
    @location(2) material_id: f32,
    @location(3) offset_id: f32,
    @location(4) cache_tail: f32
};

struct TraceResult {
    color: vec3f,
    normal: vec3f,
    front_normal: vec3f,
    pos: vec3f,
    id: u32,
    offset_id: i32
};

struct Material {
    albedo: vec3f,
    fuzz: f32,
    refractive_index: f32,
    kind: i32
};

struct Ray {
    origin: vec3f,
    direction: vec3f
};

struct HitRecord {
    normal: vec3f,
    pos: vec3f,
    offset_id: i32,
    t: f32,
    id: u32,
};

struct ScatterRecord {
    attenuation: vec3f,
    direction: vec3f
};

struct RandomSeed {
    value: u32,
    p0: u32, p1: u32, p2: u32
};

@group(0) @binding(0) var voxel_data: texture_3d<u32>;
@group(0) @binding(1) var<uniform> random_seed: RandomSeed;
@group(0) @binding(2) var<uniform> inverse_projection_matrix: mat4x4f;
@group(0) @binding(3) var<uniform> projection_matrix: mat4x4f;
@group(0) @binding(4) var<uniform> view_matrix: mat4x4f;

@group(0) @binding(5) var prev_tex_sampler: sampler;
@group(0) @binding(6) var<uniform> prev_view_matrix: mat4x4f;
@group(0) @binding(7) var<uniform> reproject: f32;
@group(0) @binding(8) var<uniform> materials: array<Material, 256>;

@group(1) @binding(0) var prev_color_tex: texture_2d<f32>;
@group(1) @binding(1) var prev_normal_tex: texture_2d<f32>;
@group(1) @binding(2) var prev_mat_tex: texture_2d<f32>;
@group(1) @binding(3) var prev_offset_tex: texture_2d<f32>;
@group(1) @binding(4) var prev_cache_tail_tex: texture_2d<f32>;

var<private> rng_state: u32;
var<private> is_in_water: bool = false;

@vertex 
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pos = (in.uv * 2.0) - vec2f(1.0);
    out.clip_position = vec4f(pos, 0.0, 1.0);
    out.uv = in.uv;
    let t1 = inverse_projection_matrix * vec4f(pos, -1.0, 1.0);
    let t2 = view_matrix * vec4f(t1.xyz, 0.0);
    out.ray_direction = t2.xyz;
    out.ray_origin = vec3f(view_matrix[3][0], view_matrix[3][1], view_matrix[3][2]);
    return out;
}


fn construct_onb(n: vec3f) -> Onb {
    let w = normalize(n);
    var a: vec3f;
    if abs(w.x) > 0.9 {
        a = vec3f(0.0, 1.0, 0.0);
    } else {
        a = vec3f(1.0, 0.0, 0.0);
    }
    let v = normalize(cross(w, a));
    let u = normalize(cross(w, v));
    return Onb(u, v, w);
}

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

fn random_vec3f() -> vec3f {
    return vec3f(random_f32(), random_f32(), random_f32());
}

fn random_vec3f_range(low: f32, high: f32) -> vec3f {
    return vec3f(low) + random_vec3f() * (high - low);
}

fn random_in_unit_sphere() -> vec3f {
    var p: vec3f;
    loop {
        p = random_vec3f_range(-1.0, 1.0);
        if (dot(p, p) < 1.0) {
            break;
        }
    }
    return p;
}

fn align_to_direction(n: vec3f, cos_theta: f32, phi: f32) -> vec3f {
    let sin_theta = sqrt(saturate(1.0 - cos_theta * cos_theta));
    let onb = construct_onb(n);
    return (onb.u * cos(phi) + onb.v * sin(phi)) * sin_theta + 
            n * cos_theta;
}

fn sample_cosine_weighted_hemisphere(n: vec3f) -> vec3f {
    let r0 = random_f32();
    let r1 = random_f32();
    let cos_theta = sqrt(r0);
    return align_to_direction(n, cos_theta, r1 * two_pi);
}

fn sample_ggx_distribution(n: vec3f, alpha_sq: f32) -> vec3f {
    let r0 = random_f32();
    let r1 = random_f32();
    let cos_theta = sqrt(saturate((1.0 - r0) / (r0 * (alpha_sq - 1.0) + 1.0)));
    return align_to_direction(n, cos_theta, r1 * two_pi);
}

fn schlick(cosine: f32, refractive_index: f32) -> f32 {
    let r0_ = (1.0 - refractive_index) / (1.0 + refractive_index);
    let r0 = r0_ * r0_;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

fn ray_at(ray: Ray, t: f32) -> vec3f {
    return ray.origin + ray.direction * t;
}

fn safe_sign(x: f32) -> f32 {
    if (x <= 0.0) {
        return -1.0;
    }
    return 1.0;
}

fn voxel_traverse(ray: Ray) -> HitRecord {
    var record: HitRecord;
    let origin = ray.origin;
    let direction = normalize(ray.direction);

    let step = vec3f(
        safe_sign(direction.x),
        safe_sign(direction.y),
        safe_sign(direction.z),
    );
    let stepi = vec3i(step);
    var current_voxel = vec3i(floor(origin / VOXEL_SIZE));
    let next_bound = vec3f(current_voxel + (stepi + vec3i(1)) / 2) * VOXEL_SIZE;

    var t_max = (next_bound - origin) / direction;
    let t_delta = VOXEL_SIZE / direction * step;
    var i: i32 = 0;

    var original_id = textureLoad(voxel_data, current_voxel, 0).r;
    loop {
        if (t_max.x < t_max.y && t_max.x < t_max.z) {
            record.offset_id = current_voxel.x;
            record.t = t_max.x;
            record.normal = vec3f(-step.x, 0.0, 0.0);
            t_max.x += t_delta.x;
            current_voxel.x += stepi.x;
        } else if (t_max.y < t_max.z) {
            record.offset_id = current_voxel.y;
            record.t = t_max.y;
            record.normal = vec3f(0.0, -step.y, 0.0);
            t_max.y += t_delta.y;
            current_voxel.y += stepi.y;
        } else {
            record.offset_id = current_voxel.z;
            record.t = t_max.z;
            record.normal = vec3f(0.0, 0.0, -step.z);
            t_max.z += t_delta.z;
            current_voxel.z += stepi.z;
        }

        record.id = textureLoad(voxel_data, current_voxel, 0).r;
        if is_in_water {
            if (record.id != 2u) {
                if (record.id == 0u) {
                    record.id = original_id;
                    record.pos = ray_at(ray, record.t + 0.001);
                } else {
                    record.pos = ray_at(ray, record.t);
                }
                break;
            }
        } else {
            if (record.id != 0u) {
                record.pos = ray_at(ray, record.t + 0.001);
                break;
            }
        }
        original_id = record.id;

        i += 1;
        if (i >= MAXIMUM_TRAVERSAL_DISTANCE) {
            break;
        }
    }

    return record;
}

fn scatter(ray: Ray, hrec: HitRecord) -> ScatterRecord {
    var srec: ScatterRecord;

    let material = materials[hrec.id];
    switch material.kind {
        case 0 /* MAT_DIFFUSE */, default: {
            srec.direction = sample_cosine_weighted_hemisphere(hrec.normal);
            srec.attenuation = material.albedo;
        }
        case 1 /* MAT_METAL */: {
            let alpha_sq = material.fuzz * material.fuzz;
            let microfacet_n = sample_ggx_distribution(hrec.normal, alpha_sq);
            srec.direction = microfacet_n;
            srec.attenuation = material.albedo;
        }
        case 2 /* MAT_DIELECTRIC */: {
            ///*
            var refraction_ratio = material.refractive_index;
            if dot(ray.direction, hrec.normal) <= 0.0 {
                refraction_ratio = 1.0 / refraction_ratio;
            }
            let cos_theta = min(dot(-ray.direction, hrec.normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

            if refraction_ratio * sin_theta > 1.0 || 
               schlick(cos_theta, refraction_ratio) > random_f32() {
                srec.direction = reflect(ray.direction, hrec.normal);
                srec.attenuation = vec3f(1.0);
            } else {
                srec.direction = refract(ray.direction, hrec.normal, 
                                            refraction_ratio);
                is_in_water = !is_in_water;
                srec.attenuation = material.albedo;
            }
        }
    }

    return srec;
}

fn background(ray: Ray) -> vec3f {
    return vec3f(0.5);
}

fn trace(ray_: Ray) -> TraceResult {
    var result: TraceResult;
    result.color = vec3f(1.0);
    var ray = ray_;

    var i: i32 = 0;
    for (; i < MAX_BOUNCE_COUNT; i += 1) {
        let hrec = voxel_traverse(ray);
        if (hrec.id == 0u) {
            if (i == 0) {
                result.color = background(ray);
            }
            break;
        }

        let srec = scatter(ray, hrec);
        result.color *= srec.attenuation;
        ray.origin = hrec.pos;
        ray.direction = normalize(srec.direction);

        if (i == 0) {
            result.pos = hrec.pos;
            result.id = hrec.id;
            result.normal = hrec.normal;
            result.offset_id = hrec.offset_id;
        }

        /*
        if i > 3 {
            let p = max(max(result.x, result.y), result.z);
            if random_f32() > min(p, 0.95) {
                break;
            }
            result *= 1.0 / p;
        }
        */
    }

    return result;
}

fn temporal_reverse_reprojection(fs: TraceResult, uv: vec2f) -> FragmentOutput {
    var result: FragmentOutput;
    result.color = vec4f(fs.color, 1.0);
    result.normal = vec4f((fs.normal + vec3f(1.0)) * 0.5, 0.0);
    result.material_id = f32(fs.id);
    result.offset_id = f32(fs.offset_id);

    let point = projection_matrix * prev_view_matrix * vec4f(fs.pos, 1.0);
    let p = point.xyz / point.w;
    let prev_uv1 = (p.xy + vec2f(1.0)) * 0.5;
    let prev_uv = vec2f(prev_uv1.x, 1.0 - prev_uv1.y);
    
    let prev_normal = textureSample(prev_normal_tex, prev_tex_sampler, prev_uv).rgb;
    let prev_offset_id = textureSample(prev_offset_tex, prev_tex_sampler, prev_uv).r;
    let prev_mat_id = textureSample(prev_mat_tex, prev_tex_sampler, prev_uv).r;
    let prev_cache_tail = textureSample(prev_cache_tail_tex, prev_tex_sampler, prev_uv).r;
    let prev_color = textureSample(prev_color_tex, prev_tex_sampler, prev_uv).rgb;
    
    if (result.material_id != 0.0) {
        if (prev_uv.x > 0.0 && prev_uv.x < 1.0 &&
            prev_uv.y > 0.0 && prev_uv.y < 1.0 &&
            result.material_id == prev_mat_id && 
            distance(result.normal.xyz, prev_normal) < 0.1 && 
            result.offset_id == prev_offset_id
        ) {
            let alpha = (1.0 / 9.0) * reproject;
            result.cache_tail = (1.0 - alpha) * prev_cache_tail;
            result.color = vec4f((alpha * result.color.xyz) + (1.0 - alpha) * prev_color, 1.0);
        } else {
            // missed the cache
            result.cache_tail = 1.0;
        }
    }

    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    rng_state = xorshift32(bitcast<u32>(in.uv.x * 123.0 +
                                        in.uv.y * 987.0) 
                           * random_seed.value);

    let ray = Ray(
        in.ray_origin,
        normalize(in.ray_direction)
    );

    return temporal_reverse_reprojection(trace(ray), in.uv);
}
