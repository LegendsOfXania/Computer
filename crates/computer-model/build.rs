use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use wit_parser::{Docs, FunctionKind, InterfaceId, Resolve, Type, TypeDefKind, WorldId, WorldItem};

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let wit = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../wit");
    let out = PathBuf::from(env::var("OUT_DIR")?);
    println!("cargo:rerun-if-changed={}", wit.display());

    let mut resolve = Resolve::default();
    let (package, _) = resolve.push_dir(&wit)?;
    let world = resolve.select_world(&[package], Some("cosmos"))?;
    let cbor = env::var_os("CARGO_FEATURE_EXTENSION").is_some();

    fs::write(out.join("types.rs"), types(&resolve, cbor)?)?;
    fs::write(
        out.join("ipc.rs"),
        if cbor { ipc(&resolve, world)? } else { String::new() },
    )?;

    Ok(())
}

fn types(resolve: &Resolve, cbor: bool) -> Res<String> {
    let mut out = String::new();
    let mut seen = HashMap::new();

    for (_, def) in resolve.types.iter() {
        let (Some(name), false) = (&def.name, matches!(def.kind, TypeDefKind::Type(_))) else {
            continue;
        };
        let name = camel(name);

        if seen.insert(name.clone(), ()).is_some() {
            return Err(format!("duplicate WIT type `{name}`").into());
        }

        let n = |i: usize| if cbor { format!("#[n({i})] ") } else { String::new() };
        let derive = |extra: &str, attr: &str| {
            format!(
                "{}#[derive(Debug, Clone, PartialEq{extra}, serde::Serialize, serde::Deserialize{})]\n{}",
                docs(&def.docs, ""),
                if cbor { ", minicbor::Encode, minicbor::Decode" } else { "" },
                if cbor { format!("#[cbor({attr})]\n") } else { String::new() },
            )
        };

        match &def.kind {
            TypeDefKind::Record(record) => {
                out += &derive("", "array");
                writeln!(out, "pub struct {name} {{")?;
                for (i, field) in record.fields.iter().enumerate() {
                    writeln!(
                        out,
                        "{}    {}pub {}: {},",
                        docs(&field.docs, "    "),
                        n(i),
                        ident(&field.name),
                        rust_type(resolve, field.ty)?,
                    )?;
                }
            }
            TypeDefKind::Enum(enum_) => {
                out += &derive(", Copy, Eq", "index_only");
                writeln!(out, "pub enum {name} {{")?;
                for (i, case) in enum_.cases.iter().enumerate() {
                    writeln!(out, "{}    {}{},", docs(&case.docs, "    "), n(i), camel(&case.name))?;
                }
            }
            TypeDefKind::Variant(variant) => {
                out += &derive("", "flat");
                writeln!(out, "pub enum {name} {{")?;
                for (i, case) in variant.cases.iter().enumerate() {
                    let payload = case
                        .ty
                        .map(|ty| Ok::<_, Box<dyn Error>>(format!("({}{})", n(0), rust_type(resolve, ty)?)))
                        .transpose()?
                        .unwrap_or_default();
                    writeln!(out, "{}    {}{}{payload},", docs(&case.docs, "    "), n(i), camel(&case.name))?;
                }
            }
            _ => return Err(format!("unsupported WIT type definition `{name}`").into()),
        }

        out += "}\n\n";
    }

    Ok(out)
}

fn ipc(resolve: &Resolve, world: WorldId) -> Res<String> {
    let world = &resolve.worlds[world];
    let mut ids = HashMap::new();
    let mut out = String::new();

    for item in world.imports.values().chain(world.exports.values()) {
        if let WorldItem::Interface { id, .. } = item {
            out += &interface(resolve, *id, &mut ids)?;
        }
    }

    Ok(out)
}

fn interface(resolve: &Resolve, id: InterfaceId, ids: &mut HashMap<u32, String>) -> Res<String> {
    let interface = &resolve.interfaces[id];
    let name = interface.name.as_deref().ok_or("unnamed WIT interface")?;
    let qualified = resolve.id_of(id).ok_or("WIT interface has no qualified name")?;

    let (mut methods, mut clients, mut arms, mut constants) =
        (String::new(), String::new(), String::new(), String::new());

    for (function_name, function) in &interface.functions {
        if !matches!(function.kind, FunctionKind::Freestanding | FunctionKind::AsyncFreestanding) {
            return Err(format!("`{qualified}#{function_name}`: only free functions are supported").into());
        }

        let path = format!("{qualified}#{function_name}");
        let call_id = fnv1a_32(path.as_bytes());

        if let Some(other) = ids.insert(call_id, path.clone()) {
            return Err(format!("WIT function ID collision: {other} and {path}").into());
        }

        let ok = fallible(resolve, function.result)
            .map_err(|reason| format!("`{path}`: {reason}"))?;

        let constant = constant(function_name);
        let method = ident(function_name);
        let doc = docs(&function.docs, "    ");

        let (mut owned, mut borrowed, mut names) = (Vec::new(), Vec::new(), Vec::new());
        for param in &function.params {
            let param_name = ident(&param.name);
            owned.push(format!("{param_name}: {}", rust_type(resolve, param.ty)?));
            borrowed.push(format!("{param_name}: {}", borrowed_type(resolve, param.ty)?));
            names.push(param_name);
        }

        let tuple = |items: &[String]| match items {
            [one] => format!("({one},)"),
            _ => format!("({})", items.join(", ")),
        };
        let (self_owned, self_borrowed) = (
            owned.iter().map(|p| format!(", {p}")).collect::<String>(),
            borrowed.iter().map(|p| format!(", {p}")).collect::<String>(),
        );
        let args = tuple(&names);

        writeln!(constants, "    pub const {constant}: u32 = {call_id:#010x};")?;
        writeln!(methods, "{doc}    fn {method}(&self{self_owned}) -> impl Future<Output = Result<{ok}, String>>;\n")?;
        writeln!(
            clients,
            "{doc}    pub async fn {method}<T: Transport>(transport: &T{self_borrowed}) -> Result<{ok}, String> {{\n        call(transport, {constant}, {args}).await\n    }}\n"
        )?;
        writeln!(
            arms,
            "            {constant} => {{\n                let {args} = args(&mut decoder)?;\n                reply(&handler.{method}({}).await)\n            }}",
            names.join(", "),
        )?;
    }

    let trait_name = camel(name);

    Ok(format!(
        "{}pub mod {module} {{\n    #[allow(unused_imports)]\n    use crate::wit::*;\n    use super::{{Transport, args, call, open, reply}};\n\n{constants}\n    /// Implemented by the side that serves `{qualified}`; the other side calls it with the functions below.\n    pub trait {trait_name} {{\n{methods}    }}\n\n{clients}    /// Decodes a request, runs the matching handler method and encodes its response.\n    pub async fn serve<H: {trait_name}>(handler: &H, request: &[u8]) -> Result<Vec<u8>, String> {{\n        let mut decoder = minicbor::Decoder::new(request);\n\n        match open(&mut decoder)? {{\n{arms}            id => Err(format!(\"unknown function {{id:#010x}} in `{qualified}`\")),\n        }}\n    }}\n}}\n\n",
        docs(&interface.docs, ""),
        module = ident(name),
    ))
}

fn fallible(resolve: &Resolve, result: Option<Type>) -> Result<String, String> {
    let Some(Type::Id(id)) = result.map(|ty| unalias(resolve, ty)) else {
        return Err("must return `result<_, string>`".into());
    };

    match &resolve.types[id].kind {
        TypeDefKind::Result(result) if result.err.map(|ty| unalias(resolve, ty)) == Some(Type::String) => result
            .ok
            .map_or_else(|| Ok("()".to_owned()), |ty| rust_type(resolve, ty).map_err(|e| e.to_string())),
        _ => Err("must return `result<_, string>`".into()),
    }
}

fn unalias(resolve: &Resolve, ty: Type) -> Type {
    match ty {
        Type::Id(id) => match resolve.types[id].kind {
            TypeDefKind::Type(inner) => unalias(resolve, inner),
            _ => ty,
        },
        _ => ty,
    }
}

fn rust_type(resolve: &Resolve, ty: Type) -> Res<String> {
    let id = match ty {
        Type::Bool => return Ok("bool".into()),
        Type::U8 => return Ok("u8".into()),
        Type::U16 => return Ok("u16".into()),
        Type::U32 => return Ok("u32".into()),
        Type::U64 => return Ok("u64".into()),
        Type::S8 => return Ok("i8".into()),
        Type::S16 => return Ok("i16".into()),
        Type::S32 => return Ok("i32".into()),
        Type::S64 => return Ok("i64".into()),
        Type::F32 => return Ok("f32".into()),
        Type::F64 => return Ok("f64".into()),
        Type::Char => return Ok("char".into()),
        Type::String => return Ok("String".into()),
        Type::Id(id) => id,
        Type::ErrorContext => return Err("unsupported WIT type `error-context`".into()),
    };

    let def = &resolve.types[id];
    let mut inner = |ty| rust_type(resolve, ty);

    Ok(match &def.kind {
        TypeDefKind::Record(_) | TypeDefKind::Variant(_) | TypeDefKind::Enum(_) => {
            camel(def.name.as_deref().ok_or("unnamed WIT type")?)
        }
        TypeDefKind::Type(ty) => inner(*ty)?,
        TypeDefKind::List(ty) => format!("Vec<{}>", inner(*ty)?),
        TypeDefKind::Option(ty) => format!("Option<{}>", inner(*ty)?),
        TypeDefKind::Map(key, value) => {
            format!("std::collections::BTreeMap<{}, {}>", inner(*key)?, inner(*value)?)
        }
        TypeDefKind::Tuple(tuple) => {
            let types = tuple.types.iter().map(|ty| inner(*ty)).collect::<Res<Vec<_>>>()?;
            format!("({}{})", types.join(", "), if types.len() == 1 { "," } else { "" })
        }
        TypeDefKind::Result(result) => {
            let mut side = |ty: Option<Type>| ty.map_or_else(|| Ok("()".to_owned()), &mut inner);
            format!("Result<{}, {}>", side(result.ok)?, side(result.err)?)
        }
        other => return Err(format!("unsupported WIT type `{other:?}`").into()),
    })
}

fn borrowed_type(resolve: &Resolve, ty: Type) -> Res<String> {
    let ty = unalias(resolve, ty);
    let owned = rust_type(resolve, ty)?;

    Ok(match ty {
        Type::String => "&str".to_owned(),
        Type::Id(id) => match &resolve.types[id].kind {
            TypeDefKind::List(item) => format!("&[{}]", rust_type(resolve, *item)?),
            TypeDefKind::Enum(_) => owned,
            _ => format!("&{owned}"),
        },
        _ => owned,
    })
}

fn docs(docs: &Docs, indent: &str) -> String {
    docs.contents
        .as_deref()
        .unwrap_or_default()
        .lines()
        .map(|line| format!("{indent}/// {line}\n"))
        .collect()
}

fn camel(name: &str) -> String {
    name.split('-')
        .flat_map(|word| {
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().chain(chars))
        })
        .flatten()
        .collect()
}

fn ident(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern",
        "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "static", "struct", "trait", "true", "type", "unsafe", "use",
        "where", "while",
    ];

    let name = name.replace('-', "_");

    if KEYWORDS.contains(&name.as_str()) { format!("r#{name}") } else { name }
}

fn constant(name: &str) -> String {
    name.replace('-', "_").to_uppercase()
}

fn fnv1a_32(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}
