use dioxus::prelude::*;

const ICONS: Asset = asset!("/assets/icons");

#[component]
pub fn Icon(#[props(into, default = "close".to_string())] name: String) -> Element {
    let url = format!("{ICONS}/{name}.svg");

    rsx! {
        span {
            class: "icon",
            style: "-webkit-mask-image: url({url}); mask-image: url({url});",
        }
    }
}
