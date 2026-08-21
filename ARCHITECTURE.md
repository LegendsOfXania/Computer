# Architecture

The workspace is intentionally split by responsibility rather than by implementation detail.

## model

Portable, dependency-free data definitions:

- `Value` and `Number`
- `Schema`, `Field`, and `StructSchema`
- `EntryData`
- `PageData` and `PageType`

`model` does not know about registries, runtime storage, extensions, serialization, or Pumpkin. It is the crate intended to be compiled independently for WebAssembly.

## serialization

Transforms `model` values and page documents to and from external formats. It currently provides KDL support. Serialization-specific transport types such as `RawEntry` live here because an entry identifier is document/runtime context, not part of `EntryData`.

## runtime

Owns live state and services:

- typed runtime `Entry` values
- pages and the library
- typed `Ref<E>` resolution
- extension type and tag registration
- validation against registered definitions and the current library

This is also the public API an extension uses when it declares entry types and tags. `engine` was removed because it had no coherent responsibility yet. A future Pumpkin integration can introduce an engine crate that depends on these three crates rather than becoming their owner.

Dependency graph:

```text
model
  ↑      ↑
  │      │
serialization  runtime
                 ↑
          future Pumpkin engine
```
