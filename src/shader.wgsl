@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Camera {
    position: vec2<f32>,
    height: f32,
    player_position: vec2<f32>,
}

@group(1)
@binding(0)
var<uniform> camera: Camera;

@compute
@workgroup_size(16, 16)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let size = vec2<i32>(textureDimensions(output_texture));
    let coords = vec2<i32>(global_id.xy);

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let aspect = f32(size.x) / f32(size.y);
    let uv = vec2<f32>(coords) / vec2<f32>(size);

    let world_position = (uv - 0.5) * vec2<f32>(aspect * camera.height, camera.height) + camera.position;

    textureStore(output_texture, coords, clamp(vec4<f32>(world_position - camera.player_position, 0.0, 1.0), vec4<f32>(0.0), vec4<f32>(1.0)));
}
