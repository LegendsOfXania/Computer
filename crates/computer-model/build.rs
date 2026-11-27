use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use wit_parser::{Handle, Resolve, Type, TypeDefKind, TypeOwner, WorldItem};

type BuildResult<T> = Result<T, Box<dyn Error>>;
type AttributeMap = BTreeMap<String, String>;

#[derive(Default)]
struct CborGeneration {
    type_attributes: AttributeMap,
    member_attributes: AttributeMap,
    ignored_derives: BTreeSet<String>,
    variant_impls: String,
}

struct FunctionMetadata {
    constant_name: String,
    parameters: Vec<(String, String)>,
    result: Option<String>,
    is_async: bool,
    imported: bool,
}

fn main() -> BuildResult<()> {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_EXTENSION");

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| io::Error::other("CARGO_MANIFEST_DIR is not set"))?,
    );
    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").ok_or_else(|| io::Error::other("OUT_DIR is not set"))?,
    );

    let mut wit_bindings = String::from(
        "wit_bindgen::generate!({\n\
         path: \"../../wit\",\n\
         world: \"cosmos\",\n\
         additional_derives: [\n\
             Clone,\n\
             PartialEq,\n\
             serde::Serialize,\n\
             serde::Deserialize,\n",
    );

    let (function_ids, variant_impls) =
        if env::var_os("CARGO_FEATURE_EXTENSION").is_some() {
            let wit_dir = manifest_dir.join("../../wit");
            println!("cargo:rerun-if-changed={}", wit_dir.display());

            let mut resolve = Resolve::default();
            let (package_id, sources) = resolve.push_dir(&wit_dir)?;

            for path in sources.paths() {
                println!("cargo:rerun-if-changed={}", path.display());
            }

            let world_id = resolve.select_world(&[package_id], Some("cosmos"))?;
            let generated = generate_cbor_attributes(&resolve, package_id, world_id)?;

            wit_bindings.push_str(
                "minicbor::Encode,\n\
                 minicbor::Decode,\n\
                 minicbor::CborLen,\n\
             ],\n",
            );

            append_derive_ignore(&mut wit_bindings, &generated.ignored_derives);
            append_attribute_map(
                &mut wit_bindings,
                "additional_type_attributes",
                &generated.type_attributes,
            );
            append_attribute_map(
                &mut wit_bindings,
                "additional_member_attributes",
                &generated.member_attributes,
            );

            (
                generate_function_ids(&resolve, world_id)?,
                generated.variant_impls,
            )
        } else {
            wit_bindings.push_str("         ],\n");
            (String::new(), String::new())
        };

    wit_bindings.push_str("});\n");

    fs::write(out_dir.join("wit_bindings.rs"), wit_bindings)?;
    fs::write(out_dir.join("variant_impls.rs"), variant_impls)?;
    fs::write(out_dir.join("function_ids.rs"), function_ids)?;

    Ok(())
}

fn generate_cbor_attributes(
    resolve: &Resolve,
    package_id: wit_parser::PackageId,
    world_id: wit_parser::WorldId,
) -> BuildResult<CborGeneration> {
    let mut generated = CborGeneration::default();

    let package = resolve
        .packages
        .get(package_id)
        .ok_or_else(|| io::Error::other("Cosmos WIT package was not found"))?;

    let world = resolve
        .worlds
        .get(world_id)
        .ok_or_else(|| io::Error::other("Cosmos WIT world was not found"))?;

    let package_path = format!(
        "{}::{}",
        rust_module_name(&package.name.namespace),
        rust_module_name(&package.name.name),
    );

    for (interface_name, interface_id) in &package.interfaces {
        let interface = resolve
            .interfaces
            .get(*interface_id)
            .ok_or_else(|| io::Error::other("Cosmos WIT interface was not found"))?;

        let interface_selector = resolve
            .id_of(*interface_id)
            .ok_or_else(|| io::Error::other("Cosmos WIT interface has no qualified name"))?;

        let interface_is_exported = world.exports.values().any(
            |item| matches!(item, WorldItem::Interface { id, .. } if id == interface_id),
        );

        let interface_path = format!(
            "{}{}::{}",
            if interface_is_exported {
                "exports::"
            } else {
                ""
            },
            package_path,
            rust_module_name(interface_name),
        );

        for (name, type_id) in &interface.types {
            let type_def = resolve
                .types
                .get(*type_id)
                .ok_or_else(|| io::Error::other("Cosmos WIT type was not found"))?;

            let selector = format!("{interface_selector}/{name}");

            match &type_def.kind {
                TypeDefKind::Record(record) => {
                    generated
                        .type_attributes
                        .insert(selector.clone(), "#[cbor(array)]".to_owned());

                    for (index, field) in record.fields.iter().enumerate() {
                        generated.member_attributes.insert(
                            format!("{selector}.{}", field.name),
                            format!("#[n({index})]"),
                        );
                    }
                }

                TypeDefKind::Enum(enum_) => {
                    generated
                        .type_attributes
                        .insert(selector.clone(), "#[cbor(index_only)]".to_owned());

                    for (index, case) in enum_.cases.iter().enumerate() {
                        generated.member_attributes.insert(
                            format!("{selector}.{}", case.name),
                            format!("#[n({index})]"),
                        );
                    }
                }

                TypeDefKind::Variant(variant) => {
                    // wit-bindgen already implements Clone for these variants.
                    // Their CBOR implementations are generated separately below.
                    generated.ignored_derives.insert(name.clone());

                    generated.type_attributes.insert(
                        selector,
                        "#[derive(PartialEq, serde::Serialize, serde::Deserialize)]"
                            .to_owned(),
                    );

                    let rust_type_path =
                        format!("{interface_path}::{}", rust_type_name(name));

                    generate_variant_impls(
                        &mut generated.variant_impls,
                        &rust_type_path,
                        name,
                        variant,
                    )?;
                }

                _ => {}
            }
        }
    }

    Ok(generated)
}

struct VariantCase {
    index: u32,
    rust_name: String,
    has_payload: bool,
}

fn generate_variant_impls(
    output: &mut String,
    rust_type_path: &str,
    type_name: &str,
    variant: &wit_parser::Variant,
) -> BuildResult<()> {
    let cases = variant
        .cases
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let index = u32::try_from(index).map_err(|_| {
                io::Error::other(format!("too many cases in WIT variant `{type_name}`"))
            })?;

            Ok(VariantCase {
                index,
                rust_name: rust_type_name(&case.name),
                has_payload: case.ty.is_some(),
            })
        })
        .collect::<BuildResult<Vec<_>>>()?;

    generate_variant_encode(output, rust_type_path, &cases);
    generate_variant_decode(output, rust_type_path, &cases);
    generate_variant_cbor_len(output, rust_type_path, &cases);

    Ok(())
}

fn generate_variant_encode(
    output: &mut String,
    rust_type_path: &str,
    cases: &[VariantCase],
) {
    output.push_str(&format!(
        "\n// CBOR Encode implementation generated from WIT.\n\
         impl<C> minicbor::Encode<C> for {rust_type_path} {{\n\
             fn encode<W: minicbor::encode::Write>(\n\
                 &self,\n\
                 encoder: &mut minicbor::Encoder<W>,\n\
                 context: &mut C,\n\
             ) -> Result<(), minicbor::encode::Error<W::Error>> {{\n\
                 match self {{\n"
    ));

    for case in cases {
        let VariantCase {
            index,
            rust_name,
            has_payload,
        } = case;

        if *has_payload {
            output.push_str(&format!(
                "            Self::{rust_name}(value) => {{\n\
                     encoder.array(2)?.u32({index})?;\n\
                     minicbor::Encode::encode(value, encoder, context)?;\n\
                 }},\n"
            ));
        } else {
            output.push_str(&format!(
                "            Self::{rust_name} => {{\n\
                     encoder.array(1)?.u32({index})?;\n\
                 }},\n"
            ));
        }
    }

    output.push_str(
        "        }\n\
             Ok(())\n\
         }\n\
         }\n",
    );
}

fn generate_variant_decode(
    output: &mut String,
    rust_type_path: &str,
    cases: &[VariantCase],
) {
    output.push_str(&format!(
        "\nimpl<'bytes, C> minicbor::Decode<'bytes, C> for {rust_type_path} {{\n\
             fn decode(\n\
                 decoder: &mut minicbor::Decoder<'bytes>,\n\
                 context: &mut C,\n\
             ) -> Result<Self, minicbor::decode::Error> {{\n\
                 let array_len = decoder.array()?.ok_or_else(|| {{\n\
                     minicbor::decode::Error::message(\n\
                         \"indefinite WIT variant arrays are not supported\",\n\
                     )\n\
                 }})?;\n\
                 let case_index = decoder.u32()?;\n\
                 match case_index {{\n"
    ));

    for case in cases {
        let VariantCase {
            index,
            rust_name,
            has_payload,
        } = case;

        if *has_payload {
            output.push_str(&format!(
                "            {index} => {{\n\
                     if array_len != 2 {{\n\
                         return Err(minicbor::decode::Error::message(\n\
                             \"invalid WIT variant array length\",\n\
                         ));\n\
                     }}\n\
                     Ok(Self::{rust_name}(\n\
                         minicbor::Decode::decode(decoder, context)?,\n\
                     ))\n\
                 }},\n"
            ));
        } else {
            output.push_str(&format!(
                "            {index} => {{\n\
                     if array_len != 1 {{\n\
                         return Err(minicbor::decode::Error::message(\n\
                             \"invalid WIT variant array length\",\n\
                         ));\n\
                     }}\n\
                     Ok(Self::{rust_name})\n\
                 }},\n"
            ));
        }
    }

    output.push_str(
        "            _ => Err(minicbor::decode::Error::message(\n\
                 \"unknown WIT variant discriminant\",\n\
             )),\n\
         }\n\
         }\n\
         }\n",
    );
}

fn generate_variant_cbor_len(
    output: &mut String,
    rust_type_path: &str,
    cases: &[VariantCase],
) {
    output.push_str(&format!(
        "\nimpl<C> minicbor::CborLen<C> for {rust_type_path} {{\n\
             fn cbor_len(&self, context: &mut C) -> usize {{\n\
                 match self {{\n"
    ));

    for case in cases {
        let VariantCase {
            index,
            rust_name,
            has_payload,
        } = case;

        if *has_payload {
            output.push_str(&format!(
                "            Self::{rust_name}(value) => {{\n\
                     1 + minicbor::CborLen::cbor_len(&{index}u32, context)\n\
                         + minicbor::CborLen::cbor_len(value, context)\n\
                 }},\n"
            ));
        } else {
            output.push_str(&format!(
                "            Self::{rust_name} => {{\n\
                     1 + minicbor::CborLen::cbor_len(&{index}u32, context)\n\
                 }},\n"
            ));
        }
    }

    output.push_str(
        "        }\n\
             }\n\
         }\n",
    );
}

fn generate_function_ids(
    resolve: &Resolve,
    world_id: wit_parser::WorldId,
) -> BuildResult<String> {
    let world = resolve
        .worlds
        .get(world_id)
        .ok_or_else(|| io::Error::other("Cosmos WIT world was not found"))?;

    let mut functions = BTreeMap::<String, FunctionMetadata>::new();

    for (items, imported) in [(&world.imports, true), (&world.exports, false)] {
        for item in items.values() {
            let WorldItem::Interface { id, .. } = item else {
                continue;
            };

            let interface = resolve
                .interfaces
                .get(*id)
                .ok_or_else(|| io::Error::other("Cosmos WIT interface was not found"))?;

            let interface_name = interface
                .name
                .as_deref()
                .ok_or_else(|| io::Error::other("unnamed Cosmos interface"))?;

            let qualified_interface = resolve
                .id_of(*id)
                .ok_or_else(|| io::Error::other("Cosmos interface has no qualified name"))?;

            for function in interface.functions.values() {
                let qualified_name =
                    format!("{qualified_interface}#{}", function.name);

                let constant_name = format!(
                    "{}_{}",
                    rust_constant_name(interface_name),
                    rust_constant_name(&function.name),
                );

                let parameters = function
                    .params
                    .iter()
                    .map(|parameter| {
                        Ok((
                            parameter.name.clone(),
                            wit_type_name(resolve, parameter.ty, 0)?,
                        ))
                    })
                    .collect::<BuildResult<Vec<_>>>()?;

                let result = function
                    .result
                    .map(|ty| wit_type_name(resolve, ty, 0))
                    .transpose()?;

                let metadata = FunctionMetadata {
                    constant_name,
                    parameters,
                    result,
                    is_async: function.kind.is_async(),
                    imported,
                };

                if functions
                    .insert(qualified_name.clone(), metadata)
                    .is_some()
                {
                    return Err(io::Error::other(format!(
                        "duplicate WIT function: {qualified_name}"
                    ))
                        .into());
                }
            }
        }
    }

    let mut seen_ids = BTreeMap::<u32, String>::new();
    let mut seen_names = BTreeMap::<String, String>::new();

    for (qualified_name, metadata) in &functions {
        let id = fnv1a_32(qualified_name.as_bytes());

        if seen_ids
            .insert(id, qualified_name.clone())
            .is_some_and(|previous| previous != *qualified_name)
        {
            return Err(io::Error::other(format!(
                "WIT function ID collision: {qualified_name} maps to {id:#010x}"
            ))
                .into());
        }

        if seen_names
            .insert(metadata.constant_name.clone(), qualified_name.clone())
            .is_some_and(|previous| previous != *qualified_name)
        {
            return Err(io::Error::other(format!(
                "WIT function constant collision: {}",
                metadata.constant_name
            ))
                .into());
        }
    }

    let mut output = String::from(
        "// Generated from the Cosmos WIT world.\n\n"
    );

    for (qualified_name, metadata) in &functions {
        let id = fnv1a_32(qualified_name.as_bytes());

        output.push_str(&format!(
            "/// Function `{qualified_name}`.\n\
             pub const {}: u32 = {id:#010x};\n\n",
            metadata.constant_name,
        ));
    }

    for (qualified_name, metadata) in &functions {
        let parameters = metadata
            .parameters
            .iter()
            .map(|(name, ty)| format!("({name:?}, {ty:?})"))
            .collect::<Vec<_>>()
            .join(", ");

        let result = metadata.result.as_ref().map_or_else(
            || "None".to_owned(),
            |ty| format!("Some({ty:?})"),
        );

        let constant_name = &metadata.constant_name;
        let is_async = metadata.is_async;
        let imported = metadata.imported;

        output.push_str(&format!(
             "/// Signature of `{qualified_name}`.\n\
              pub const {constant_name}_SIGNATURE: Signature = Signature {{\n\
                  id: {constant_name},\n\
                  name: {qualified_name:?},\n\
                  parameters: &[{parameters}],\n\
                  result: {result},\n\
                  is_async: {is_async},\n\
                  imported: {imported},\n\
              }};\n\n",
        ));
    }

    output.push_str("pub const SIGNATURES: &[Signature] = &[\n");

    for metadata in functions.values() {
        output.push_str(&format!(
            "    {}_SIGNATURE,\n",
            metadata.constant_name,
        ));
    }

    output.push_str(
        "];\n\n\
         /// Finds a WIT function signature by its ID.\n\
         #[must_use]\n\
         pub fn signature(id: u32) -> Option<&'static Signature> {\n\
             match id {\n",
    );

    for metadata in functions.values() {
        output.push_str(&format!(
            "        {} => Some(&{}_SIGNATURE),\n",
            metadata.constant_name,
            metadata.constant_name,
        ));
    }

    output.push_str(
        "        _ => None,\n\
             }\n\
         }\n",
    );

    Ok(output)
}

fn wit_type_name(
    resolve: &Resolve,
    ty: Type,
    depth: usize,
) -> BuildResult<String> {
    if depth > 64 {
        return Err(io::Error::other("WIT type nesting is too deep").into());
    }

    let primitive = match ty {
        Type::Bool => Some("bool"),
        Type::U8 => Some("u8"),
        Type::U16 => Some("u16"),
        Type::U32 => Some("u32"),
        Type::U64 => Some("u64"),
        Type::S8 => Some("s8"),
        Type::S16 => Some("s16"),
        Type::S32 => Some("s32"),
        Type::S64 => Some("s64"),
        Type::F32 => Some("f32"),
        Type::F64 => Some("f64"),
        Type::Char => Some("char"),
        Type::String => Some("string"),
        Type::ErrorContext => Some("error-context"),
        Type::Id(_) => None,
    };

    if let Some(name) = primitive {
        return Ok(name.to_owned());
    }

    let Type::Id(type_id) = ty else {
        return Err(io::Error::other("unsupported WIT type").into());
    };

    let type_def = resolve
        .types
        .get(type_id)
        .ok_or_else(|| io::Error::other("WIT type was not found"))?;

    if let Some(name) = &type_def.name {
        return match type_def.owner {
            TypeOwner::Interface(interface_id) => {
                let interface_name = resolve
                    .id_of(interface_id)
                    .ok_or_else(|| io::Error::other("WIT interface has no qualified name"))?;

                Ok(format!("{interface_name}/{name}"))
            }
            TypeOwner::World(_) | TypeOwner::None => Ok(name.clone()),
        };
    }

    match &type_def.kind {
        TypeDefKind::Type(inner) => {
            wit_type_name(resolve, *inner, depth + 1)
        }

        TypeDefKind::List(inner) => Ok(format!(
            "list<{}>",
            wit_type_name(resolve, *inner, depth + 1)?,
        )),

        TypeDefKind::Map(key, value) => Ok(format!(
            "map<{}, {}>",
            wit_type_name(resolve, *key, depth + 1)?,
            wit_type_name(resolve, *value, depth + 1)?,
        )),

        TypeDefKind::Tuple(tuple) => {
            let types = tuple
                .types
                .iter()
                .map(|ty| wit_type_name(resolve, *ty, depth + 1))
                .collect::<BuildResult<Vec<_>>>()?;

            Ok(format!("tuple<{}>", types.join(", ")))
        }

        TypeDefKind::Option(inner) => Ok(format!(
            "option<{}>",
            wit_type_name(resolve, *inner, depth + 1)?,
        )),

        TypeDefKind::Result(result) => {
            let ok = optional_wit_type_name(resolve, result.ok, depth + 1)?;
            let err = optional_wit_type_name(resolve, result.err, depth + 1)?;

            Ok(format!("result<{ok}, {err}>"))
        }

        TypeDefKind::FixedLengthList(inner, length) => Ok(format!(
            "fixed-length-list<{}, {length}>",
            wit_type_name(resolve, *inner, depth + 1)?,
        )),

        TypeDefKind::Future(inner) => Ok(format!(
            "future<{}>",
            optional_wit_type_name(resolve, *inner, depth + 1)?,
        )),

        TypeDefKind::Stream(inner) => Ok(format!(
            "stream<{}>",
            optional_wit_type_name(resolve, *inner, depth + 1)?,
        )),

        TypeDefKind::Handle(handle) => match handle {
            Handle::Own(resource) => Ok(format!(
                "own<{}>",
                wit_type_name(resolve, Type::Id(*resource), depth + 1)?,
            )),
            Handle::Borrow(resource) => Ok(format!(
                "borrow<{}>",
                wit_type_name(resolve, Type::Id(*resource), depth + 1)?,
            )),
        },

        TypeDefKind::Record(record) => {
            let fields = record
                .fields
                .iter()
                .map(|field| {
                    Ok(format!(
                        "{}: {}",
                        field.name,
                        wit_type_name(resolve, field.ty, depth + 1)?,
                    ))
                })
                .collect::<BuildResult<Vec<_>>>()?;

            Ok(format!("record{{{}}}", fields.join(", ")))
        }

        TypeDefKind::Variant(variant) => {
            let cases = variant
                .cases
                .iter()
                .map(|case| {
                    let payload = case
                        .ty
                        .map(|ty| wit_type_name(resolve, ty, depth + 1))
                        .transpose()?;

                    Ok(match payload {
                        Some(payload) => format!("{}({payload})", case.name),
                        None => case.name.clone(),
                    })
                })
                .collect::<BuildResult<Vec<_>>>()?;

            Ok(format!("variant{{{}}}", cases.join(", ")))
        }

        TypeDefKind::Enum(enum_) => Ok(format!(
            "enum{{{}}}",
            enum_
                .cases
                .iter()
                .map(|case| case.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )),

        TypeDefKind::Flags(flags) => Ok(format!(
            "flags{{{}}}",
            flags
                .flags
                .iter()
                .map(|flag| flag.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )),

        TypeDefKind::Resource => Ok("resource".to_owned()),

        TypeDefKind::Unknown => {
            Err(io::Error::other("unknown WIT type").into())
        }
    }
}

fn optional_wit_type_name(
    resolve: &Resolve,
    ty: Option<Type>,
    depth: usize,
) -> BuildResult<String> {
    match ty {
        Some(ty) => wit_type_name(resolve, ty, depth),
        None => Ok("_".to_owned()),
    }
}

fn append_derive_ignore(output: &mut String, names: &BTreeSet<String>) {
    if names.is_empty() {
        return;
    }

    output.push_str("additional_derives_ignore: [");

    for name in names {
        output.push_str(&format!("{name:?}, "));
    }

    output.push_str("],\n");
}

fn append_attribute_map(
    output: &mut String,
    option: &str,
    attributes: &AttributeMap,
) {
    if attributes.is_empty() {
        return;
    }

    output.push_str(option);
    output.push_str(": {\n");

    for (selector, attribute) in attributes {
        output.push_str("    ");
        output.push_str(&format!("{selector:?}"));
        output.push_str(": [");
        output.push_str(attribute);
        output.push_str("],\n");
    }

    output.push_str("},\n");
}

fn rust_module_name(name: &str) -> String {
    name.replace('-', "_")
}

fn rust_type_name(name: &str) -> String {
    let mut result = String::new();
    let mut uppercase_next = true;

    for character in name.chars() {
        if character == '-' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            result.extend(character.to_uppercase());
            uppercase_next = false;
        } else {
            result.push(character);
        }
    }

    result
}

fn rust_constant_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_uppercase());
        } else if !result.ends_with('_') {
            result.push('_');
        }
    }

    result.trim_matches('_').to_owned()
}

fn fnv1a_32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}