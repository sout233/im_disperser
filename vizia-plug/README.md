# Vizia Plug

A replacement for `nih-plug-vizia` which updates it to the latest version of `vizia`.

This crate allows for the use of the Vizia GUI library to be used with the nih-plug plugin framework.

## Building Examples

```
cargo +nightly xtask bundle gain_gui --release
```

The outputs will be placed in the `target\bundled` directory.