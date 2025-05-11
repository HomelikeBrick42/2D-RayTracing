static const int CHUNK_SIZE = 128;

struct Cell
{
    float3 color;
    uint solid;
}

struct Chunk
{
    Cell cells[CHUNK_SIZE * CHUNK_SIZE];
}

typealias Chunks = StructuredBuffer<Chunk, Std430DataLayout>;

struct Ray
{
    float2 origin;
    float2 direction;
}

struct Hit
{
    float2 position;
    float3 color;
    float distance;
}

Optional<Hit> trace_chunk(Chunks chunks, int chunk_index, Ray ray, float max_distance)
{
    var hit : Hit;
    hit.distance = 0;
    hit.color = 1.0;

    let step_sizes = 1.0 / abs(ray.direction);
    let step_dir = int2(sign(ray.direction));
    var next_dist = (float2(step_dir) * 0.5 + 0.5 - fract(ray.origin)) / ray.direction;

    var wall_hit_pos : Optional<float2> = none;
    var wall_hit_color : Optional<float3> = none;
    var voxel_pos = int2(floor(ray.origin));
    while (true)
    {
        let closest_dist = min(next_dist.x, next_dist.y);
        hit.distance += closest_dist;

        if (all(voxel_pos >= int2(0)) && all(voxel_pos < int2(CHUNK_SIZE)))
        {
            let index = int(voxel_pos.x) + int(voxel_pos.y) * CHUNK_SIZE;
            let cell = chunks[chunk_index].cells[index];

            if (cell.solid != 0)
            {
                if (!wall_hit_pos.hasValue)
                {
                    wall_hit_pos = ray.origin + ray.direction * (hit.distance - closest_dist);
                    wall_hit_color = cell.color;
                }
            }

            if (hit.distance >= max_distance)
            {
                hit.distance = max_distance;
                hit.position = ray.origin + ray.direction * hit.distance;
                hit.color *= chunks[chunk_index].cells[index].color;

                if (cell.solid != 0)
                {
                    hit.color *= 0.5;
                }

                if (wall_hit_pos.hasValue)
                {
                    // hit.color *= clamp(1.0 - dot(hit.position - wall_hit_pos.value, ray.direction), 0.0, 1.0);

                    // let strength = 0.1;
                    // let t = clamp(strength / (pow(distance(hit.position, wall_hit_pos.value), 2.0) + strength), 0.0, 1.0);
                    // hit.color = lerp(float3(0.05, 0.05, 0.05), hit.color, t);

                    let strength = 0.1;
                    let power = 2.0;
                    let distance = distance(hit.position, wall_hit_pos.value) / 10.0;
                    // let distance = abs(hit.position.x - wall_hit_pos.value.x) + abs(hit.position.y - wall_hit_pos.value.y);
                    let t = clamp(strength / (pow(distance, power) + strength), 0.00, 1.0);
                    hit.color = lerp(float3(0.05, 0.05, 0.05), hit.color, t);
                }

                return hit;
            }
        }
        else
        {
            return none;
        }

        let step_axis = int2(next_dist == float2(closest_dist));
        voxel_pos += step_axis * step_dir;
        next_dist -= closest_dist;
        next_dist += step_sizes * float2(step_axis);
    }
}
