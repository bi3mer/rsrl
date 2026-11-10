# Vendored Crates

## bracket-terminal 0.8.7

A copy of `bracket-terminal` 0.8.7 from crates.io (MIT license), used by `rltk` through `[patch.crates-io]` in the root `Cargo.toml`.

The fix is in `src/hal/scaler.rs`, in `change_physical_size_smooth` and `recalculate_coordinates`. Upstream sets only one of the two smooth gutters on each resize and keeps the other from the previous resize. When a tiling window manager resizes the window twice with different aspect ratios, the stale gutter can exceed the new window size, and `physical_size - gutter` underflows a `u32` (panic at line 199). The patch resets both gutters on every resize and uses saturating subtraction. The bug is also present on the upstream `master` branch as of 2026-11-10.

Remove this directory and the `[patch.crates-io]` entry once a fixed release ships.

`src/hal/gl_common/backing/shared_main_loop.rs` also drops four unneeded `mut` bindings (lines 67, 88, 109, and 130) to silence `unused_mut` warnings on current Rust.

## specs 0.20.0

A copy of `specs` 0.20.0 from crates.io (MIT or Apache-2.0 license), used as a path dependency in the root `Cargo.toml`.

The only changes silence compiler warnings on current Rust. `cargo fix` added explicit `'_` lifetimes to 26 return types in `src/storage/mod.rs`, `src/storage/restrict.rs`, `src/world/entity.rs`, `src/world/lazy.rs`, and `src/world/world_ext.rs`. `atomic_increment` in `src/world/entity.rs` also drops the deprecated `use std::usize;`, so `usize::MAX` now refers to the associated constant.

## specs-derive 0.4.1

An unmodified copy of `specs-derive` 0.4.1 from crates.io (MIT or Apache-2.0 license), used as a path dependency in the root `Cargo.toml`. It provides `#[derive(Component)]` for `specs`. The crates.io package ships no license files; the licenses match those in `vendor/specs`.
