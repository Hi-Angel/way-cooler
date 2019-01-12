/// Generated modules from the XML protocol spec.
pub use self::generated::server::*;

mod generated {
    // Generated code generally doesn't follow standards
    #![allow(dead_code, non_camel_case_types, unused_unsafe, unused_variables)]
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, unused_qualifications)]

    pub mod c_interfaces {
        use wayland_client::sys::{common::*, protocol_interfaces::*};
        #[doc(hidden)]
        include!(concat!(env!("OUT_DIR"), "/noop_interface.rs"));
    }

    pub mod server {
        #[doc(hidden)]
        use super::c_interfaces;
        #[doc(hidden)]
        use wayland_server::protocol::*;
        #[doc(hidden)]
        use wayland_server::*;
        use wayland_client::commons::*;
        #[doc(hidden)]
        use wayland_client::protocol::*;
        #[doc(hidden)]
        use wayland_client::*;
        use wayland_sys as sys;
        include!(concat!(env!("OUT_DIR"), "/noop_api.rs"));
    }
}
