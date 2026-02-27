# Interop helpers between `exn` and `anyhow`

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/exn-anyhow.svg
[crates-url]: https://crates.io/crates/exn-anyhow
[docs-badge]: https://docs.rs/exn-anyhow/badge.svg
[docs-url]: https://docs.rs/exn-anyhow
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/exn-anyhow
[license-url]: https://github.com/fast/exn/blob/main/LICENSE
[actions-badge]: https://github.com/fast/exn/workflows/CI/badge.svg
[actions-url]:https://github.com/fast/exn/actions?query=workflow%3ACI

## Overview

`exn-anyhow` provides explicit boundary conversion helpers:

- `exn_anyhow::to_anyhow`: `exn::Exn<E>` -> `anyhow::Error`
- `exn_anyhow::from_anyhow`: `anyhow::Error` -> `exn::Exn<exn_anyhow::AnyhowError>`

Recommended usage is explicit error conversion at API boundaries via `map_err`:

```rust
use exn_anyhow::from_anyhow;
use exn_anyhow::to_anyhow;

let exn_result = anyhow_result.map_err(from_anyhow);
let anyhow_result = exn_result.map_err(to_anyhow);
```

## Documentation

Read the online documents at https://docs.rs/exn-anyhow.

## Examples

```bash
cargo run -p examples --example from-anyhow
cargo run -p examples --example into-anyhow
```

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.85.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor
version updates. For example, if version 1.0 requires Rust 1.60.0, then version 1.0.z for all
values of z will also require Rust 1.60.0 or newer. However, version 1.y for y > 0 may require a
newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](https://github.com/fast/exn/blob/main/LICENSE).
