# GLSL to Rust struct translator
* Ensure alignment equality between generated Rust code and input GLSL code
* If needed, the system should try to solve for a structure with the same fields and better alignment for glsl

## Roadmap
* Extremely basic translation from a GLSL structure to a Rust one
* Try to match up the alignment rules
* Test harness
* Test cases for odd alignments in GLSL.
* Eventually release this to the peeps in the Vulkan Discord and watch them poke holes in it
    * Recover from that emotionally and code-wise (probably)
