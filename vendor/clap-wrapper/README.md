# clap-wrapper-rs

[![Validate](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml/badge.svg)](https://github.com/blepfx/clap-wrapper-rs/actions/workflows/validate.yml)
[![Crates](https://img.shields.io/crates/v/clap-wrapper)](https://crates.io/crates/clap-wrapper)

An easy way to use [clap-wrapper](https://github.com/free-audio/clap-wrapper) in your Rust plugins!

## Usecases
- Adding VST3 or AUv2 support to existing Rust plugin frameworks that do not support them (e.g. [clack](https://github.com/prokopyl/clack))
- Making your own audio plugin framework without dealing with VST3 and AUv2 directly

## Features
- Provides a simple way to export Rust-based CLAP plugins as VST3 and AUv2 plugins.
- Builds "fat", self-contained binaries for VST3 and AUv2 plugins.
- Does not use `cmake`. Instead it uses the `cc` crate to compile the `clap-wrapper` code.
- Tested on Linux (Ubuntu 22.04), MacOS (13.7) and Windows (10). In theory the minimum supported OSX version is 10.12, but I have no way to test that.

## Limitations
- Currently only supports VST3 and AUv2 plugins. Standalone builds are not supported yet.
- AUv2 wrapper can only export up to 4 plugins per binary for now.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
clap-wrapper = { version = "0.3.1", features = ["vst3", "auv2", "parallel"] } # these features are enabled by default
```
    
Then, in your `lib.rs`:
```rust
// exports `GetPluginFactoryAUV2` symbol.
clap_wrapper::export_auv2!(); 
// exports `GetPluginFactory` symbol and extra VST3 symbols.
clap_wrapper::export_vst3!(); 
```

This will export VST3 and AUv2 entrypoints that use the `clap_entry` symbol exported from your crate (as an example, `nih_plug::nih_export_clap` exports it).

Keep in mind, that `clap-wrapper-rs` only adds the necessary entrypoints that reexport the CLAP plugin you already have. You'd still have to use a crate like `nih-plug` to actually create the plugin.


After building, you have to "bundle" your plugin. This means setting up the correct directory structure and copying the necessary files. See [VST 3 Developer Portal: Plug-in Format Structure](https://steinbergmedia.github.io/vst3_dev_portal/pages/Technical+Documentation/Locations+Format/Plugin+Format.html) for more info about VST3 directory structure. For AUv2, the directory structure is similar. 


**clap-wrapper-rs** ships with a `bundler` tool that can be run as a separate step after
building the plugin library. Click [here](bundler/README.md) for more info about how to use it.


See [validate.yml](.github/workflows/validate.yml) for a complete example of how to build, bundle and validate a plugin.

## Changelog

- 0.3.1:
    - Added unique suffix to generated ObjC classes as a temporary workaround
- 0.3.0:
    - Updated `clap-wrapper` to latest (0.14.0). See [clap-wrapper changelog](https://github.com/free-audio/clap-wrapper/wiki/ChangeLog#0140-march-2026).
    - Added an experimental bundler tool (see [bundler](bundler) folder) that can be used to automate the bundling process.
- 0.2.1:
    - Added documentation
- 0.2.0:
    - Embedded VST3 and AUv2 SDKs directly into the crate, removing the need to download them separately. This is possible thanks to VST3 SDK's new MIT license. 
    - Added `vst3` and `auv2` features to enable/disable building those wrappers.
    - Simplified build.rs by a lot.
- 0.1.2:
    - Updated `clap-wrapper` to latest.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.