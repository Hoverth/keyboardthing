// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{any::Any, error::Error};

mod protocols;

slint::include_modules!();

use wayland_client::{protocol::wl_registry, Connection, Dispatch, Proxy, QueueHandle};
// This struct represents the state of our app. This simple app does not
// need any state, but this type still supports the `Dispatch` implementations.
struct AppData {
    use_kde: bool,
    use_generic: bool,
}

// Implement `Dispatch<WlRegistry, ()> for our state. This provides the logic
// to be able to process events for the wl_registry interface.
//
// The second type parameter is the user-data of our implementation. It is a
// mechanism that allows you to associate a value to each particular Wayland
// object, and allow different dispatching logic depending on the type of the
// associated value.
//
// In this example, we just use () as we don't have any value to associate. See
// the `Dispatch` documentation for more details about this.
impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        // When receiving events from the wl_registry, we are only interested in the
        // `global` event, which signals a new available global.
        // When receiving this event, we just print its characteristics in this example.
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} (v{})", name, interface, version);
            match interface.as_str() {
                "org_kde_kwin_fake_input" => {
                    println!("KDE found!");
                    state.use_kde = true;
                }
                "zwp_virtual_keyboard_v1" => {
                    println!("Generic Compositor found!");
                    state.use_generic = !state.use_kde; // kde overrides this value
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app_data = AppData {
        use_kde: false,
        use_generic: false,
    };

    // from https://docs.rs/wayland-client/latest/wayland_client/index.html
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    println!("Advertised globals:");
    event_queue.roundtrip(&mut app_data).unwrap();

    use protocols::kde::org_kde_kwin_fake_input as pkde;

    let fake_in = pkde::OrgKdeKwinFakeInput::inert(conn.backend().downgrade());

    println!("{fake_in:?}");

    // either of these panic it
    fake_in.keyboard_key(0, 3);
    fake_in.authenticate(
        String::from("keyboardthing"),
        String::from("Emulate keyboard"),
    );

    let ui = AppWindow::new()?;

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.run()?;

    Ok(())
}
