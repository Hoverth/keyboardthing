use flexi_logger::{FileSpec, Logger};
use slint::ComponentHandle;
use std::error::Error;
use std::sync::{Arc, Mutex};
use wayland_client::event_created_child;
use wayland_client::{protocol::wl_registry, Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::wp::input_method::zv1::client::{
    zwp_input_method_context_v1, zwp_input_method_v1,
};

slint::include_modules!();

#[derive(Debug)]
struct AppData {
    // These two bools don't currently do anything.
    use_kde: bool,
    use_generic: bool,
    input_context: Option<zwp_input_method_context_v1::ZwpInputMethodContextV1>,
}

impl Dispatch<zwp_input_method_context_v1::ZwpInputMethodContextV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &zwp_input_method_context_v1::ZwpInputMethodContextV1,
        event: zwp_input_method_context_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        log::info!("Context: {event:?}, State: {state:?}");
    }
}

impl Dispatch<zwp_input_method_v1::ZwpInputMethodV1, ()> for AppData {
    event_created_child!(AppData, zwp_input_method_v1::ZwpInputMethodV1, [
        0 => (zwp_input_method_context_v1::ZwpInputMethodContextV1, ())
    ]);

    fn event(
        state: &mut Self,
        _: &zwp_input_method_v1::ZwpInputMethodV1,
        event: zwp_input_method_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        log::info!("Event: {event:?}");
        if let zwp_input_method_v1::Event::Activate { id } = event {
            log::info!("Activated: {id:?}");
            state.input_context = Some(id);
        } else if let zwp_input_method_v1::Event::Deactivate { context } = event {
            log::info!("Deactivated {context:?}");
            if let Some(input_context) = &state.input_context {
                if context == *input_context {
                    state.input_context = None;
                }
            }
            context.destroy();
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            log::info!("[{}] {} (v{})", name, interface, version);
            match interface.as_str() {
                "zwp_input_method_v1" => {
                    log::info!("Found string support (zwp_input_method_v1)");
                    registry.bind::<zwp_input_method_v1::ZwpInputMethodV1, _, _>(name, 1, qh, ());
                }
                "org_kde_kwin_fake_input" => {
                    println!("KDE found!");
                    log::info!("Found KDE fake input (org_kde_kwin_fake_input)");
                    state.use_kde = true;
                }
                "zwp_virtual_keyboard_v1" => {
                    state.use_generic = !state.use_kde; // kde overrides this value
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    log_panics::init();
    Logger::try_with_str("info")?
        .log_to_file(FileSpec::try_from("/tmp/keyboardthing/keyboardthing.log")?)
        .append()
        .format(flexi_logger::detailed_format)
        .start()?;
    log::info!("Started logging!\n\n");

    let app_data = AppData {
        use_kde: false,
        use_generic: false,
        input_context: None,
    };

    let appdata = Arc::new(Mutex::new(app_data));

    // from https://docs.rs/wayland-client/latest/wayland_client/index.html
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();
    let mut event_queue: EventQueue<AppData> = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());

    event_queue
        .roundtrip(&mut appdata.clone().lock().unwrap())
        .unwrap();

    let ui = AppWindow::new()?;

    ui.on_request_reload({
        log::info!("Wayland reload requested");
        let appdata = appdata.clone();
        move || {
            event_queue
                .roundtrip(&mut appdata.clone().lock().unwrap())
                .unwrap();
        }
    });

    ui.on_add_text({
        let ui_handle = ui.as_weak();
        let appdata = appdata.clone();
        move |text| {
            log::info!("Trying to add text!: {text:?}");

            if let Some(context) = &appdata.clone().lock().unwrap().input_context {
                context.commit_string(0, text.to_string());
            }

            let ui = ui_handle.unwrap();
            ui.invoke_request_reload();
        }
    });

    ui.invoke_request_reload();

    ui.run()?;

    Ok(())
}
