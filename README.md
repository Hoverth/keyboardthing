# Keyboard Thing

An attempt to create a virtual keyboard on wayland using rust.

## Dev Notes

- Plasma only advertises virtual keyboard interfaces to applications that advertise it (see `keyboardthing.desktop`)
- [wayland-client simple window example](https://github.com/Smithay/wayland-rs/blob/master/wayland-client/examples/simple_window.rs)
- [wayland-cliend docs.rs](https://docs.rs/wayland-client/latest/wayland_client)
- [wayland.app input method v1](https://wayland.app/protocols/input-method-unstable-v1)
