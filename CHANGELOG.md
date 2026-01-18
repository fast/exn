# CHANGELOG

All significant changes to this project will be documented in this file.

## Unreleased

### Breaking Changes

* `Exn::from_iter` has been renamed to `Exn::raise_all`
* `exn::Error` trait bound has been removed in favor of inlined `StdError + Send + Sync + 'static` bounds.
* `err.raise()` has been moved to the `exn::ErrorExt` extension trait.

### New Features

* This crate is now `no_std` compatible, while the crate still depends on the `alloc` crate for heap allocations. It is worth noting that `no_std` support is a nice-to-have feature now, and can be dropped if it blocks other important features in the future. Before 1.0, once the exn APIs settle down, the decision on whether to keep `no_std` as a promise will be finalized.
