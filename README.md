OpenGL Simple Fluid Simulation (Rust)
====================================

A small real‑time fluid simulation and renderer written in Rust.
It uses SPH‑style particle simulation and renders either instanced
cubes or an extracted surface via a marching‑cubes compute shader.

- Rendering: `glium` + `winit`
- UI: `egui`
- Math: `nalgebra_glm`
- Surface extraction: GLSL compute shader (OpenGL 4.3+)

Requirements
------------

- Rust (stable)
- GPU/driver with OpenGL 4.3 or newer (for compute shaders)

Build & Run
-----------

- Debug: `cargo run`
- Release: `cargo run --release`

Controls
--------

- Movement: `W / A / S / D`, `Space` (up), `Left Ctrl` (down)
- Mouse: look control when enabled (see UI panel)
- UI is shown on the left (Egui)

UI (left panel)
---------------

- Simulation parameters: kernel radius, particle mass, gas constant,
  rest density, viscosity, iso level
- Draw mode:
  - `Draw cubes` on: instanced particles
  - `Draw cubes` off: surface via marching cubes
    - Surface options: wireframe, color, alpha, (optional) debug points
- Bounds controls: position, rotation, scale; `Draw bounds` toggle

Notes
-----

- Surface mode requires an OpenGL 4.3+ capable GPU.
- If surface is invisible, try adjusting `iso level` and check
  `Draw bounds` to verify the simulation domain.

