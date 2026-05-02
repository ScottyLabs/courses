//! Client-side search index for the CMU course catalog. Compiles to a native
//! binary for the weekly build step that produces `catalog.bin`, and to wasm
//! for the runtime query engine loaded by the browser.

pub mod binary;
pub mod doc;
pub mod index;
pub mod load;
