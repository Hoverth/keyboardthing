pub mod kde {
    use wayland_client;
    use wayland_scanner;
    // This module hosts a low-level representation of the protocol objects
    // you will not need to interact with it yourself, but the code generated
    // by the generate_client_code! macro will use it
    pub mod __interfaces {
        // import the interfaces from the core protocol if needed
        wayland_scanner::generate_interfaces!("src/protocols/fake-input.xml");
    }
    use self::__interfaces::*;

    // This macro generates the actual types that represent the wayland objects of
    // your custom protocol
    wayland_scanner::generate_client_code!("src/protocols/fake-input.xml");
}

pub mod generic {
    use wayland_client;
    use wayland_scanner;
    // import objects from the core protocol if needed
    use wayland_client::protocol::*;
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("src/protocols/virtual-keyboard-unstable-v1.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("src/protocols/virtual-keyboard-unstable-v1.xml");
}
