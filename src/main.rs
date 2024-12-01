use flexi_logger::{FileSpec, Logger};
use raw_window_handle::HasWindowHandle;
use slint::ComponentHandle;
use std::error::Error;
use std::sync::{Arc, Mutex};
use wayland_client::{
    event_created_child,
    protocol::{wl_registry, wl_surface},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols::wp::input_method::zv1::client::{
    zwp_input_method_context_v1, zwp_input_method_v1, zwp_input_panel_surface_v1,
    zwp_input_panel_v1,
};

slint::include_modules!();

#[derive(Debug)]
struct AppData {
    // These two bools don't currently do anything.
    use_kde: bool,
    use_generic: bool,
    surface: Option<wl_surface::WlSurface>,
    input_context: Option<zwp_input_method_context_v1::ZwpInputMethodContextV1>,
    input_panel_surface: Option<zwp_input_panel_surface_v1::ZwpInputPanelSurfaceV1>,
}

impl Dispatch<zwp_input_panel_surface_v1::ZwpInputPanelSurfaceV1, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &zwp_input_panel_surface_v1::ZwpInputPanelSurfaceV1,
        event: zwp_input_panel_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        log::info!("Context: {event:?}, State: {state:?}");
    }
}

impl Dispatch<zwp_input_panel_v1::ZwpInputPanelV1, ()> for AppData {
    // Unsure if this is needed.
    event_created_child!(AppData, zwp_input_panel_v1::ZwpInputPanelV1, [
        0 => (zwp_input_panel_surface_v1::ZwpInputPanelSurfaceV1, ())
    ]);

    fn event(
        state: &mut Self,
        panel: &zwp_input_panel_v1::ZwpInputPanelV1,
        event: zwp_input_panel_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        // This is not triggered, as zwp_input_panel doesn't have any events
        // This code is here bc it needs to be moved and implemented yet
        let surface = state
            .surface
            .as_ref()
            .expect("Tried to bind input panel without surface!");

        state.input_panel_surface = Some(
            zwp_input_panel_v1::ZwpInputPanelV1::get_input_panel_surface(panel, surface, qh, ()),
        );
        log::info!("Event: {event:?}\nState: {state:?}");
    }
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
                "zwp_input_panel_v1" => {
                    registry.bind::<zwp_input_panel_v1::ZwpInputPanelV1, _, _>(name, 1, qh, ());
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
        surface: None,
        input_context: None,
        input_panel_surface: None,
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

    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))
        .unwrap();

    let ui = AppWindow::new()?;

    slint::spawn_local({
        let ui_handle = ui.as_weak();
        let appdata = appdata.clone();

        async move {
            //let conn = Connection::connect_to_env().unwrap();
            let ui = ui_handle.unwrap();
            log::info!("Attempting to get window handle for WlSurface from Slint UI...");

            let handle_binding = ui.window().window_handle();
            let handle = handle_binding.window_handle();

            match handle {
                Ok(handle) => match handle.as_raw() {
                    raw_window_handle::RawWindowHandle::Wayland(wayland_window) => {
                        let nn_surface = wayland_window.surface;
                        let wl_surface_obj_id: wayland_client::backend::ObjectId;
                        unsafe {
                            wl_surface_obj_id = wayland_client::backend::ObjectId::from_ptr(
                                wl_surface::WlSurface::interface(),
                                nn_surface.as_ptr().cast(),
                            )
                            .unwrap();
                        }
                        let wl_surface: wl_surface::WlSurface =
                            wl_surface::WlSurface::from_id(&conn, wl_surface_obj_id).unwrap();

                        /*appdata.clone().lock().unwrap().input_panel_surface = Some(
                            zwp_input_panel_v1::ZwpInputPanelV1::get_input_panel_surface(
                                self,
                                &wl_surface,
                                &qh,
                                (),
                            ),
                        );*/

                        appdata.clone().lock().unwrap().surface = Some(wl_surface);
                        log::info!("Got WlSurface from slint UI");
                    }
                    _ => {
                        log::warn!("Not running under wayland!")
                    }
                },
                Err(e) => {
                    log::warn!("Failed to get Sling UI window handle!\n{e:?}, {e}")
                }
            }

            if appdata.clone().lock().unwrap().surface == None {
                log::warn!("Failed to set WlSurface!");
            }
        }
    })
    .unwrap();

    ui.on_request_reload({
        log::info!("Wayland reload requested");
        let appdata = appdata.clone();
        move || match event_queue.roundtrip(&mut appdata.clone().lock().unwrap()) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Requested Reload failed with error: \n{e:?}")
            }
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

    ui.on_key({
        let ui_handle = ui.as_weak();
        let appdata = appdata.clone();
        move |key| {
            log::info!("Trying key!: {key:?}");
            if let Some(context) = &appdata.clone().lock().unwrap().input_context {
                let key = key.as_str();

                match key {
                    "BS" => {
                        context.delete_surrounding_text(1, 1); // TODO: this
                                                               // doesn't work for the last character for some reason
                        context.commit_string(0, String::new());
                    }
                    "TAB" => {
                        context.commit_string(0, String::from("\t")); // slint
                                                                      // doesn't support the '\t' character in it's strings
                    }
                    "LSHFT" => {
                        // need to get modifier map or whatever
                        context.modifiers(0, 0, 0, 0, 0);
                    }
                    _ => {}
                }
            }

            let ui = ui_handle.unwrap();
            ui.invoke_request_reload();
        }
    });

    ui.invoke_request_reload(); // wayland requires a couple of round trips to be fully responsive

    ui.run()?;

    Ok(())
}
