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
    float distance;
    float3 color;
}

Optional<Hit> trace_chunk(Chunks chunks, int chunk_index, Ray ray, float max_distance)
{
    var hit : Hit;
    hit.distance = 0;
    hit.color = 1.0;

    let step_sizes = 1.0 / abs(ray.direction);
    let step_dir = int2(sign(ray.direction));
    var next_dist = (float2(step_dir) * 0.5 + 0.5 - fract(ray.origin)) / ray.direction;

    var first_hit_cell : Optional<int2> = none;
    var voxel_pos = int2(floor(ray.origin));
    for (var i = 0; i < CHUNK_SIZE * CHUNK_SIZE; i++)
    {
        let closest_dist = min(next_dist.x, next_dist.y);
        hit.distance += closest_dist;
        let step_axis = int2(next_dist == float2(closest_dist));
        let next_voxel_pos = voxel_pos + step_axis * step_dir;

        if (all(voxel_pos >= int2(0)) && all(voxel_pos < int2(CHUNK_SIZE)))
        {
            let index = int(voxel_pos.x) + int(voxel_pos.y) * CHUNK_SIZE;
            let cell = chunks[chunk_index].cells[index];

            if (cell.solid != 0 && !first_hit_cell.hasValue)
                first_hit_cell = voxel_pos;

            if (hit.distance >= max_distance)
            {
                hit.distance = max_distance;

                if (first_hit_cell.hasValue)
                {
                    let cell_center_pos = float2(first_hit_cell.value) + 0.5;
                    let hit_pos = ray.origin + ray.direction * hit.distance;

                    let distance = max(abs(hit_pos.x - cell_center_pos.x), abs(hit_pos.y - cell_center_pos.y));
                    if (distance > 0.7)
                    {
                        return none;
                    }

                    let first_hit_index = int(first_hit_cell.value.x) + int(first_hit_cell.value.y) * CHUNK_SIZE;
                    hit.color *= chunks[chunk_index].cells[first_hit_index].color;
                }
                else
                {
                    hit.color *= chunks[chunk_index].cells[index].color;
                }

                return hit;
            }
        }
        else
        {
            break;
        }

        voxel_pos = next_voxel_pos;
        next_dist -= closest_dist;
        next_dist += step_sizes * float2(step_axis);
    }

    return none;
}
