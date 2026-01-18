# A context-aware concrete Error type built on `core::error::Error`

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/exn.svg
[crates-url]: https://crates.io/crates/exn
[docs-badge]: https://docs.rs/exn/badge.svg
[docs-url]: https://docs.rs/exn
[license-badge]: https://img.shields.io/crates/l/exn
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/exn/workflows/CI/badge.svg
[actions-url]:https://github.com/fast/exn/actions?query=workflow%3ACI

## Overview

`exn` provides the missing context APIs for `core::error::Error`.

It organizes errors as a tree structure, allowing you to easily access the root cause and all related errors with their context.

## Documentation

Read the online documents at https://docs.rs/exn.

## `no_std` crates

This crate is `no_std` compatible, while the `alloc` crate is still required for heap allocations.

It is worth noting that `no_std` support is a nice-to-have feature, and can be dropped if it blocks other important features in the future. Before 1.0, once `exn` APIs settle down, the decision on whether to keep `no_std` as a promise will be finalized.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
