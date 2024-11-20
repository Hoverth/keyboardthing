use slint::ComponentHandle;
use std::error::Error;
use std::sync::{Arc, Mutex};
use wayland_client::{protocol::wl_registry, Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::wp::input_method::zv1::client::zwp_input_method_v1;

slint::include_modules!();

struct AppData {
    output: String,
    use_kde: bool,
    use_generic: bool,
}

impl Dispatch<zwp_input_method_v1::ZwpInputMethodV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &zwp_input_method_v1::ZwpInputMethodV1,
        event: zwp_input_method_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        state.output += format!("{event:?}").as_str();
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            state.output += format!("[{}] {} (v{})\n", name, interface, version).as_str();
            match interface.as_str() {
                "zwp_input_method_v1" => {
                    state.output += "Found string support!\n";
                }
                "org_kde_kwin_fake_input" => {
                    state.output += "Found KDE!\n";
                    println!("KDE found!");
                    state.use_kde = true;
                }
                "zwp_virtual_keyboard_v1" => {
                    state.output += "Found generic!\n";
                    state.use_generic = !state.use_kde; // kde overrides this value
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app_data = AppData {
        output: String::new(),
        use_kde: false,
        use_generic: false,
    };

    let appdata = Arc::new(Mutex::new(app_data));

    // from https://docs.rs/wayland-client/latest/wayland_client/index.html
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();
    let mut event_queue: EventQueue<AppData> = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    let ui = AppWindow::new()?;

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.on_request_reload({
        let ui_handle = ui.as_weak();
        let appdata = appdata.clone();
        move || {
            appdata.clone().lock().unwrap().output += "---\n\n";
            event_queue
                .roundtrip(&mut appdata.clone().lock().unwrap())
                .unwrap();

            let ui = ui_handle.unwrap();
            ui.set_output(appdata.clone().lock().unwrap().output.clone().into());
        }
    });

    ui.run()?;

    Ok(())
}
