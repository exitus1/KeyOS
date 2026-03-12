<h1 align="center">micro-ecc-sys</h1>
<div align="center">
 <strong>
   Low-level bindings to micro-ecc
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/micro-ecc-sys">
    <img src="https://img.shields.io/crates/v/micro-ecc-sys.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/micro-ecc-sys">
    <img src="https://img.shields.io/crates/d/micro-ecc-sys.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- API docs -->
  <a href="https://docs.rs/micro-ecc-sys">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="main branch API docs" />
  </a>
</div>

## What is this?

Low-level bindings to the [micro-ecc][micro-ecc] ECDH and ECDSA implementation.

You may also be interested in the high-level, idiomatic Rust library [nisty][nisty].

Upstream release: [v1.0][upstream-release]

[micro-ecc]: https://github.com/kmackay/micro-ecc
[nisty]: https://crates.io/crates/nisty
[upstream-release]: https://github.com/kmackay/micro-ecc/releases/tag/v1.0

## Building / Usage

`bindgen` as build dependency triggers the cargo build bug,
`no_std` platforms need pre-generated bindings.

For Cortex-M4 and Cortex-M33 platforms, these are [packaged][cortex-m4-bindings].
To verify locally, run `cargo build --target thumbv7em-none-eabi --no-default-features`
and compare with the packaged file.

To use on such platforms, include `micro-ecc-sys` without the default features:
```
[dependencies.micro-ecc-sys]
default-features = false
```

[cortex-m4-bindings]: https://github.com/nickray/micro-ecc-sys/blob/main/cortex-m4.bindings.rs


#### License

<sup>micro-ecc is licensed under [BSD-2-Clause][bsd-2-clause], as are these bindings.</sup>

[bsd-2-clause]: https://github.com/kmackay/micro-ecc/blob/master/LICENSE.txt
