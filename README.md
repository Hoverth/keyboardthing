# Keyboard Thing

An attempt to create a virtual keyboard on wayland using rust.

## Developing

KDE Plasma will not expose the virtual keyboard APIs to any old application,
so you'll have to install `keyboardthing.desktop` to `/usr/share/applications`,
with the path corresponding to the binary (usually
`<this dir>/target/debug/keyboardthing`), and select it in the virtual
keyboards menu in system settings, in order for it to see the virtual keyboard
wayland APIs (`input_method` and `fake_input` - these will be commented on
when the wayland queue is read)

### Dev Notes & Links

- Plasma only advertises virtual keyboard interfaces to applications that
advertise it (see `keyboardthing.desktop`)
- [wayland-client simple window example](https://github.com/Smithay/wayland-rs/blob/master/wayland-client/examples/simple_window.rs)
- [wayland-cliend docs.rs](https://docs.rs/wayland-client/latest/wayland_client)
- [wayland.app input method v1](https://wayland.app/protocols/input-method-unstable-v1)
- needed to include macro (`main.rs`, lines 35-37)
- slint currently doesn't have a working always-on-top for wayland
- a keyboard working with `zwp_input_method_v1` needs to use the
`zwp_input_panel_surface_v1` to allow focus to remain on the text input instead
of the keyboard application
- don't use `conn.get_object_data(<id>)` to get a known struct, use
`struct::from_id(&conn, <id>)`
- `raw_window_handle` only works for slint's winit backend

### Next Steps (TODOs)

- get wayland handles from slint / switch ui framework (Qt rust bindings?)
  - need to access surface
- Modifier mapping, and more analysis of the input method context
