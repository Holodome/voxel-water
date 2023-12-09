fn voxel_traverse(ray: Ray) -> HitRecord {
    var record: HitRecord;
    let origin = ray.origin;
    let direction = normalize(ray.direction);
    let step = vec3f(sign(direction.x), sign(direction.y), sign(direction.z));
    var current_voxel = vec3i(floor(origin));
    let next_bound = vec3f(current_voxel + (step + vec3i(1)) / 2);
    var t_max = (next_bound - origin) / direction;
    let t_delta = direction * step;
    for (var i: i32 = 0; i<MAXIMUM_TRAVERSAL_DISTANCE; i+=1) {
        if t_max.x < t_max.y && t_max.x < t_max.z {
            record.offset_id = current_voxel.x;
            record.t = t_max.x;
            record.normal = vec3f(-step.x, 0.0, 0.0);
            t_max.x += t_delta.x;
            current_voxel.x += step.x;
        } else if t_max.y < t_max.z {
            record.offset_id = current_voxel.y;
            record.t = t_max.y;
            record.normal = vec3f(0.0, -step.y, 0.0);
            t_max.y += t_delta.y;
            current_voxel.y += step.y;
        } else {
            record.offset_id = current_voxel.z;
            record.t = t_max.z;
            record.normal = vec3f(0.0, 0.0, -step.z);
            t_max.z += t_delta.z;
            current_voxel.z += step.z;
        }
        record.id = textureLoad(voxel_data, current_voxel, 0).r;
        if record.id != 0u {
            record.pos = ray_at(ray, record.t + 0.001);
            break;
        }
    }
    return record;
}
