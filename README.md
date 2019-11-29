# compute_error_example
This repository is intended to illustrate an issue with the vulkano-shaders rust module.

# Issue
The `vulkano_shaders::shader!` macro as part of the vulkano-shaders rust module has trouble identifying storage buffers in the descriptor layout.

I am using the compute shader example from [https://vulkano.rs/guide/compute-pipeline](https://vulkano.rs/guide/compute-pipeline):
```glsl
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}
``` 
When using this shader, the generated descriptor for `(set = 0, binding = 0)` is seen as a **uniform**, leading the buffer being passed to `PersistantDescriptorSet::new()` being seen as a uniform buffer. This causes the validation-layer error that is printed on [line 14](https://github.com/Kneelawk/compute_error_example/blob/45b8ea99b4284443de4e03a836e5b17758473f3b/cargo_run_output.txt#L14) of the program's output.

I decided to run `cargo expand compute_shader` to see what the generated descriptors for the compute shader were. Most notably, the `storage` attribute of the generated `Layout`'s `DescriptorDesc` for `(0usize, 0usize)`'s `DescriptorBufferDesc` is set to `false` on [line 627](https://github.com/Kneelawk/compute_error_example/blob/a1c80e683de4a23ae165ac24b6ea722e51f325b3/compute_shader_macro_expand.rs#L627) of the macro expansion output.

# System Details
*   Version of vulkano: 0.16.0
*   OS: Linux Ubuntu 19.04
*   GPU (the selected PhysicalDevice): Nvidia GTX 1060 6GB
*   GPU Driver: NVIDIA 430.50
*   Link to main.rs: [main.rs](https://github.com/Kneelawk/compute_error_example/blob/2f9f5936e8d826ea26105beec534d873fb78ca65/src/main.rs)
*   Link to repository: [Kneelawk/compute_error_example](https://github.com/Kneelawk/compute_error_example)

## glslangValidator Details
*   Glslang Version: 7.11.3057
*   ESSL Version: OpenGL ES GLSL 3.20 glslang Khronos. 11.3057
*   GLSL Version: 4.60 glslang Khronos. 11.3057
*   SPIR-V Version 0x00010300, Revision 6
*   GLSL.std.450 Version 100, Revision 1
*   Khronos Tool ID 8
*   SPIR-V Generator Version 7
*   GL_KHR_vulkan_glsl version 100
*   ARB_GL_gl_spirv version 100

## glslc Details
`glslc` is not installed.

# Issue Link
[Here is a link](https://github.com/vulkano-rs/vulkano/issues/1283#issue-530278345) to the issue on the vulkano
repository.
