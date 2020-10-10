# GLSL to Rust struct translator
* Ensure alignment equality between generated Rust code and input GLSL code
* If needed, the system should try to solve for a structure with the same fields and better alignment for glsl

## Roadmap
* Extremely basic translation from a GLSL structure to a Rust one
* GLSL Layout only
* Tests! Tests galore!
* Try to match up the alignment rules
    * C/x86-64 and glsl/std140
* Array support
* Auto-reorder to minimize memory usage 
* Layout qualifiers
* Other platforms (with different alignment rules), std430
* Eventually release this to the peeps in the Vulkan Discord and watch them poke holes in it
    * Recover from that emotionally and code-wise (probably)
* Hints about _why_ certain descisions were made in text form

## The How
1. Take the original GLSL struct and boil it down to a set (not list!) of names paired with types we support
2. There are two things we are solving for:
    1. The alignment requirements of Rust types
    2. The alignment requirements of GLSL types
    * These must be solved simultaneously so that both conditions are satisfied with the fewest padding shims across both.

## Example 1:
```glsl
struct Particle {
    vec3 position;
    vec3 velocity;
    float mass;
    float charge;
};
```
Could be translated to:
```glsl
struct Particle {
    vec3 position;
    float mass;
    vec3 velocity;
    float charge;
};
```
And the corresponding Rust code:
```rust
#[repr(C)]
pub struct Particle {
    pub position: [f32; 3],
    pub mass: f32,
    pub velocity: [f32; 3],
    pub charge: f32,
}
```

## Example 2:
```glsl
struct Vertex {
    vec3 position;
    vec3 color;
};
```
Could be translated to:
```glsl
struct Vertex {
    vec3 color;
    vec3 position;
};
```
(Note there are no ordering gaurantees)

And the corresponding Rust code:
```rust
#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub color: [f32; 3],
    _pad0: u32,
    pub pos: [f32; 3],
    _pad1: u32,
}
```
