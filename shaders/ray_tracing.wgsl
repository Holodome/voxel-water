struct VertexInput {
    @location(0) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) ray_direction: vec3f,
    @location(2) ray_origin: vec3f
};

struct Material {
    color: vec3f,
};

struct Ray {
    origin: vec3f,
    direction: vec3f
};

struct HitRecord {
    normal: vec3f,
    pos: vec3f,
    t: f32,
    id: u32,
    has_hit: bool
};

struct ScatterRecord {
    weight: vec3f,
    direction: vec3f
};

const VOXEL_SIZE: f32 = 1.0;
const MAXIMUM_TRAVERSAL_DISTANCE: i32 = 128;
const MAX_BOUNCE_COUNT: i32 = 4;

@group(0) @binding(0)
var voxel_data: texture_3d<u32>;

@group(0) @binding(1) 
var<uniform> random_seed: u32;

@group(0) @binding(2) 
var<uniform> inverse_projection_matrix: mat4x4f;

@group(0) @binding(3)
var<uniform> projection_matrix: mat4x4f;

@group(0) @binding(4)
var<uniform> view_matrix: mat4x4f;

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

    loop {
        if (t_max.x < t_max.y && t_max.x < t_max.z) {
            record.t = t_max.x;
            record.normal = vec3f(-step.x, 0.0, 0.0);
            t_max.x += t_delta.x;
            current_voxel.x += stepi.x;
        } else if (t_max.y < t_max.z) {
            record.t = t_max.y;
            record.normal = vec3f(0.0, -step.y, 0.0);
            t_max.y += t_delta.y;
            current_voxel.y += stepi.y;
        } else {
            record.t = t_max.z;
            record.normal = vec3f(0.0, 0.0, -step.z);
            t_max.z += t_delta.z;
            current_voxel.z += stepi.z;
        }

        record.id = textureLoad(voxel_data, current_voxel, 0).r;
        if (record.id != 0u) {
            record.pos = ray_at(ray, record.t);
            record.has_hit = true;
            break;
        }

        i += 1;
        if (i >= MAXIMUM_TRAVERSAL_DISTANCE) {
            break;
        }
    }

    return record;
}

fn scatter(ray: Ray, hrec: HitRecord) -> ScatterRecord {
    var srec: ScatterRecord;

    var material: Material;
    material.color = vec3f(0.0, 0.8, 0.0);

    let target_vec = hrec.pos + hrec.normal + random_in_unit_sphere();
    srec.direction = target_vec - hrec.pos;
    srec.weight = material.color;

    return srec;
}

fn background(ray: Ray) -> vec3f {
    return vec3f(0.5);
    /*
    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * vec3f(1.0, 1.0, 1.0) + t * vec3f(0.5, 0.7, 1.0);
    */
}

fn trace(ray_: Ray) -> vec3f {
    var result = vec3f(1.0);
    var ray = ray_;

    var i: i32 = 0;
    for (; i < MAX_BOUNCE_COUNT; i += 1) {
        let hrec = voxel_traverse(ray);
        if (!hrec.has_hit) {
            break;
        }

        let srec = scatter(ray, hrec);
        result *= srec.weight;
        ray.origin = hrec.pos;
        ray.direction = normalize(srec.direction);

        if i > 3 {
            let p = max(max(result.x, result.y), result.z);
            if random_f32() > min(p, 0.95) {
                break;
            }
            result *= 1.0 / p;
        }
    }

    if (i != MAX_BOUNCE_COUNT) {
        result *= background(ray);
    }

    return result;
}

fn ray_color(ray: Ray) -> vec3f {
    return trace(ray);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    rng_state = xorshift32(bitcast<u32>(in.uv.x * 123456789.0 + in.uv.y) ^ random_seed);

    let ray = Ray(
        in.ray_origin,
        in.ray_direction
    );

    return vec4f(ray_color(ray), 1.0);
}
