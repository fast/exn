# A context-aware concrete Error type built on `core::error::Error`

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/exn.svg
[crates-url]: https://crates.io/crates/exn
[docs-badge]: https://docs.rs/exn/badge.svg
[docs-url]: https://docs.rs/exn
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/exn
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/exn/workflows/CI/badge.svg
[actions-url]: https://github.com/fast/exn/actions?query=workflow%3ACI

## Overview

`exn` provides the missing context APIs for `core::error::Error`.

It organizes errors as a tree, preserving typed context at module boundaries while still supporting type erasure where one concrete error type cannot be named.

## Typed and erased boundaries

Prefer `exn::Result<T, E>` with a concrete `E` inside modules and in domain APIs. This keeps the current root error visible in the type system.

Use a bare `Exn` at boundaries that genuinely need one concrete error type for unrelated implementations, such as callbacks, delegates, or heterogeneous collections. Typed exceptions convert into a bare `Exn` through `?` without reallocating their frame tree, and their concrete frame errors remain available for runtime downcasting. Add a typed parent again when the surrounding component incorporates the failure into its own API.

```rust
use core::fmt;
use std::io;

use exn::ErrorExt;
use exn::Exn;
use exn::ResultExt;

fn read_config() -> exn::Result<(), io::Error> {
    Err(io::Error::other("cannot read config").raise())
}

fn callback() -> Result<(), Exn> {
    read_config()?;
    Ok(())
}

fn run_callback(callback: impl FnOnce() -> Result<(), Exn>) -> exn::Result<(), fmt::Error> {
    callback().or_raise(|| fmt::Error)
}
```

## Migrating from 0.3

Import `exn::IteratorExt` and replace `Exn::raise_all(parent, children)` with `children.into_iter().raise(parent)`. Remove references to the unused `ResultExt::Error` associated type; there is no replacement associated type. See the [changelog](https://github.com/fast/exn/blob/main/CHANGELOG.md) for the complete 0.4 release notes.

## Documentation

Read the online documents at https://docs.rs/exn.

## `no_std` crates

This crate is `no_std` compatible, while the `alloc` crate is still required for heap allocations.

It is worth noting that `no_std` support is a nice-to-have feature, and can be dropped if it blocks other important features in the future. Before 1.0, once `exn` APIs settle down, the decision on whether to keep `no_std` as a promise will be finalized.

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.85.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if version 1.0 requires Rust 1.60.0, then version 1.0.z for all values of z will also require Rust 1.60.0 or newer. However, version 1.y for y > 0 may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
