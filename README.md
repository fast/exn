# A context-aware concrete Error type built on `std::error::Error`

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
[actions-badge]: https://github.com/cratesland/exn/workflows/CI/badge.svg
[actions-url]:https://github.com/cratesland/exn/actions?query=workflow%3ACI

## Overview

`exn` provides the missing context APIs for `std::error::Error`.

It organizes errors as a tree structure, allowing you to easily access the root cause and all related errors with their context, by traversing the tree with a visitor.

## Documentation

Read the online documents at https://docs.rs/exn.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
