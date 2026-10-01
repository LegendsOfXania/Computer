use dioxus::prelude::*;

const _: Asset = asset!("assets/icons", AssetOptions::folder());

#[component]
pub fn Icon(name: &'static str) -> Element {
    let src = format!("/assets/icons/{name}.svg");

    rsx! {
        img { class: "icon", src, alt: "" }
    }
}