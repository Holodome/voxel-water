fn trace(ray_: Ray) -> TraceResult {
    var result: TraceResult;
    result.color = vec3f(1.0);
    var ray = ray_;
    let hrec = voxel_traverse(ray);
    if hrec.id == 0u {
        result.color = vec3f(0.5);
        return result;
    }
    let srec = scatter(ray, hrec);
    result.color *= srec.attenuation;
    ray.origin = hrec.pos;
    ray.direction = normalize(srec.direction);
    result.pos = hrec.pos;
    result.id = hrec.id;
    result.normal = hrec.normal;
    result.offset_id = hrec.offset_id;
    var i: i32 = 1;
    for (; i < MAX_BOUNCE_COUNT; i += 1) {
        let hrec = voxel_traverse(ray);
        if hrec.id == 0u {
            break;
        }
        let srec = scatter(ray, hrec);
        result.color *= srec.attenuation;
        ray.origin = hrec.pos;
        ray.direction = normalize(srec.direction);
    }
    return result;
}
