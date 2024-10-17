#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use nbtsniper;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);
    
    let mc_bin = nbtsniper::NbtFile::read("../../tests/files/bigtest.nbt".to_string());
    let hex_dump = use_signal(|| mc_bin.hex_dump());

    rsx! {
        Link {
            to: Route::Blog {
                id: count()
            },
            "Go to blog"
        }
        div {
            h1 { "Hex Dump" },
            pre { 
                onmouseenter: move |_event| { let on_text = true; },
                onmouseleave: move |_event| { let on_text = false; },
                style: "color: red ; font-weight: bold",
                //style: "color: {} ; font-weight: {}",
                //if on_text { "red" } else { "black" },
                //if on_text { "bold" } else { "normal" },

                "{hex_dump}"    
            }
            
        }
    }
}
