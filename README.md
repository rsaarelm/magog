# calx-ecs

Calx-ecs is a serializable entity-component system for Rust.

It is based on a macro which generates a local ECS structure with serialization
implemented.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
calx-ecs = "0.3"
```

and this to your crate root:

```rust
extern crate calx_ecs;
```
