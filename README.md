# IM_DISPERSER
**[Official Website](https://audio.soout.top/im_disperser)**

A disperser plugin, made with [i_am_dsp](https://github.com/IAMMRGODIE/i_am_dsp), [nih-plug](https://github.com/robbert-vdh/nih-plug) and [vizia](https://github.com/vizia/vizia).

<img width="1193" height="742" alt="screenshot" src="https://github.com/user-attachments/assets/d50d10f3-cde6-4c68-9da9-0f973efd77da" />


## Build

For VST3/CLAP plugins

```bash
cargo xtask bundle im_disperser --release
```

For installer:

```bash
cargo run -p im_installer --release
```

## So MacOS?

see also: https://github.com/JouderMin/im_disperser/releases
