// Compute shader for array multiplication
@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@group(0) @binding(1)
var<uniform> multiplier: f32;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) {
        return;
    }
    
    data[index] = data[index] * multiplier;
}
