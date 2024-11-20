# Keyboard Thing

An attempt to create a virtual keyboard on wayland using rust.

## Developing

KDE Plasma will not expose the virtual keyboard APIs to any old application, so you'll have to install `keyboardthing.desktop` to `/usr/share/applications`, with the path corresponding to the binary (usually `<this dir>/target/debug/keyboardthing`), and select it in the virtual keyboards menu in system settings, in order for it to see the virtual keyboard wayland APIs (`input_method` and `fake_input` - these will be commented on when the wayland queue is read)

At the moment, the wayland globals are read, but the virtual keyboard APIs don't work.

### Dev Notes & Links

- Plasma only advertises virtual keyboard interfaces to applications that advertise it (see `keyboardthing.desktop`)
- [wayland-client simple window example](https://github.com/Smithay/wayland-rs/blob/master/wayland-client/examples/simple_window.rs)
- [wayland-cliend docs.rs](https://docs.rs/wayland-client/latest/wayland_client)
- [wayland.app input method v1](https://wayland.app/protocols/input-method-unstable-v1)
