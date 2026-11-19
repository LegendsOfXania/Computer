wit_bindgen::generate!({
    path: "../../wit",
    world: "cosmos",
    additional_derives: [
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    ],
});
