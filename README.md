<!--
SPDX-FileCopyrightText: 2023 Foundation Devices, Inc. <hello@foundation.xyz>
SPDX-License-Identifier: GPL-3.0-or-later
-->

# KeyOS

## Code Structure

- [`apps`](apps): Built-in KeyOS apps that are started on demand from the launcher. Each is a Rust binary crate, including a `src` directory, and a `ui` directory that defines the Slint UI. Each app also includes localization dictionaries under `i18n`.
  - [`bitcoin`](apps/gui-app-bitcoin)
    - [`src`](apps/gui-app-bitcoin/src)
    - [`ui`](apps/gui-app-bitcoin/ui)
    - [`i18n`](apps/gui-app-bitcoin/i18n)
- [`os`](os): KeyOS system services that run persistently. These typically include a binary and a library crate, and the latter provides a simple interface for sending KeyOS messages to the service. Most typical cross-app interactions are coordinated through [gui navigation](os/gui-server-api/src/navigation), including file selection and QR scanning.
  - [Filesystem](os/fs)
  - [GUI](os/gui-server-api)
  - [Camera](os/camera)
- [`xtask`](xtask): The image builder for the KeyOS system and apps. All options can be viewed with `cargo xtask help`, but most commonly used options are found in the [`Justfile`](Justfile).
- [`xous`](xous): The kernel KeyOS is built on.
  - [Timers](xous/api/ticktimer)
  - [Logging](xous/api/log)
  - [Server Names](xous/api/names)
- [`server`](server): The interface definitions for KeyOS server messages. These can be synchronous or asynchronous requests, or subscriptions. Servers will typically define a Message, implement one of the message traits for it, and a handler, then include the message in the array returned by its `messages` method.
  - [Archive](server/src/archive.rs): Allows processes to send allocated memory sized as multiples of 4096 to other processes. This is useful from sending small structs with various parameters to sending large pieces of data for processing.
  - [Scalar](server/src/scalar.rs): Allows processes to send up to 4 usize values.
  - [LendMut](server/src/lend_mut.rs): Allows processes to lend others allocated memory in multiples of 4096 to other processes for processing and modification.
  - [Event](server/src/event): Allows processes to subscribe to event messages from other processes.

## `cosign2` tool

The [`cosign2`] tool is required to be installed in order to sign the OS image.
Refer to the [`cosign2`](imports/cosign2) folder for information on how to install the tool and prepare the keys.

```bash
cargo install --path imports/cosign2/cosign2-bin
```

## Simulator

Run the simulator from the root of the repo with `just sim`. See [DEVELOPMENT.md](DEVELOPMENT.md) for info about dependencies.
