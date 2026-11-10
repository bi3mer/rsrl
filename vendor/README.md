# Vendored Crates

## specs 0.20.0

A copy of `specs` 0.20.0 from crates.io (MIT or Apache-2.0 license), used as a path dependency in the root `Cargo.toml`.

The only changes silence compiler warnings on current Rust. `cargo fix` added explicit `'_` lifetimes to 26 return types in `src/storage/mod.rs`, `src/storage/restrict.rs`, `src/world/entity.rs`, `src/world/lazy.rs`, and `src/world/world_ext.rs`. `atomic_increment` in `src/world/entity.rs` also drops the deprecated `use std::usize;`, so `usize::MAX` now refers to the associated constant.

## specs-derive 0.4.1

An unmodified copy of `specs-derive` 0.4.1 from crates.io (MIT or Apache-2.0 license), used as a path dependency in the root `Cargo.toml`. It provides `#[derive(Component)]` for `specs`. The crates.io package ships no license files; the licenses match those in `vendor/specs`.
