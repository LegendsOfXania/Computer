use dioxus::prelude::*;
use dioxus_primitives::toast::{self, Toast, ToastProps};
use std::time::Duration;

use crate::components::utils::icon::Icon;

#[component]
fn StyledToast(props: ToastProps) -> Element {
    rsx! {
        Toast {
            id: props.id,
            index: props.index,
            title: props.title,
            description: props.description,
            toast_type: props.toast_type,
            on_close: props.on_close,
            permanent: props.permanent,
            duration: props.duration,
            class: "dx-toast",
            attributes: props.attributes,
            toast::ToastContent { class: "dx-toast-content",
                toast::ToastTitle { class: "dx-toast-title" }
                toast::ToastDescription { class: "dx-toast-description" }
            }
            toast::ToastCloseButton { class: "dx-toast-close",
                Icon { name: "close" }
            }
        }
    }
}

#[component]
pub fn ToastProvider(
    #[props(default = ReadSignal::new(Signal::new(Some(Duration::from_secs(5)))))]
    default_duration: ReadSignal<Option<Duration>>,
    #[props(default = ReadSignal::new(Signal::new(10)))] max_toasts: ReadSignal<usize>,
    #[props(default)] render_toast: Option<Callback<toast::ToastPropsWithOwner, Element>>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let render_toast = render_toast.unwrap_or_else(|| {
        Callback::new(|p: toast::ToastPropsWithOwner| rsx! { StyledToast { ..p } })
    });

    rsx! {
        toast::ToastProvider {
            class: "dx-toast-container",
            default_duration,
            max_toasts,
            render_toast,
            attributes,
            {children}
        }
    }
}
