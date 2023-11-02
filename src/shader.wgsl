@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

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

    textureStore(output_texture, coords, vec4<f32>(1.0, 0.0, 0.0, 1.0));
}
