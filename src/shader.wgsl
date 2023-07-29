struct Camera {
    position: vec2<f32>,
    player_position: vec2<f32>,
    aspect_ratio: f32,
    vertical_view_height: f32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(f32((vertex_index >> 0u) & 1u), f32((vertex_index >> 1u) & 1u));
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    return out;
}

struct Ray {
    origin: vec2<f32>,
    direction: vec2<f32>,
};

struct Hit {
    hit: bool,
    distance: f32,
    point: vec2<f32>,
    normal: vec2<f32>,
    color: vec3<f32>,
};

fn trace(ray: Ray) -> Hit {
    var hit: Hit;
    hit.hit = true;
    hit.distance = 5.0;
    hit.point = ray.origin * ray.direction * hit.distance;
    hit.normal = -ray.direction;
    hit.color = vec3<f32>(ray.direction * 0.5 + 0.5, 0.0);
    return hit;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let player_to_pixel = (in.uv - 0.5) * vec2<f32>(camera.aspect_ratio, 1.0) * camera.vertical_view_height - (camera.player_position - camera.position);
    let pixel_distance = length(player_to_pixel);

    var ray: Ray;
    ray.origin = camera.player_position;
    ray.direction = player_to_pixel / pixel_distance;

    let hit = trace(ray);
    if hit.hit && hit.distance <= pixel_distance {
        return vec4<f32>(hit.color, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
