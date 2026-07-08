//! Browser-facing wrapper around [`courses_index::Index`]. Exposes a single
//! [`WasmIndex`] handle to JavaScript that owns the loaded index plus a
//! reusable [`Searcher`] workspace, so the JS side can construct it once
//! from a `catalog.bin` byte buffer and then run many queries against it
//! without paying repeated allocation costs.
//!
//! Query input and result output cross the JS/wasm boundary as plain
//! `JsValue` trees through `serde-wasm-bindgen`, which keeps the surface
//! ergonomic from JS while staying typed in Rust.

use courses_index::binary;
use courses_index::index::{Index, Query, Searcher};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmIndex {
    index: Index,
    searcher: Searcher,
}

#[wasm_bindgen]
impl WasmIndex {
    /// Build the in-memory index from a `catalog.bin` payload. Decompresses
    /// the body, decodes the corpus, and constructs all auxiliary indexes.
    /// When the catalog ships a prebuilt text bundle the heaviest build phase
    /// (FST + posting arena construction, normally ~200 ms) is skipped.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8]) -> Result<WasmIndex, JsError> {
        console_error_panic_hook::set_once();

        let payload = binary::read_catalog_minimal_from_slice(bytes)
            .map_err(|e| JsError::new(&format!("decode catalog: {e}")))?;
        let index = match payload.prebuilt_text {
            Some(p) => Index::build_with_prebuilt_text(payload.corpus, p)
                .map_err(|e| JsError::new(&format!("rebuild from prebuilt: {e}")))?,
            None => Index::build(payload.corpus),
        };
        let searcher = Searcher::new(index.n_docs as usize);
        Ok(WasmIndex { index, searcher })
    }

    /// Run a query encoded as a JS object matching the [`Query`] shape and
    /// return the [`courses_index::index::QueryResult`] as a JS object.
    pub fn query(&mut self, q: JsValue) -> Result<JsValue, JsError> {
        let query: Query = serde_wasm_bindgen::from_value(q)
            .map_err(|e| JsError::new(&format!("decode query: {e}")))?;
        let result = self.searcher.query(&self.index, &query);
        let serializer = serde_wasm_bindgen::Serializer::new()
            .serialize_maps_as_objects(true)
            .serialize_large_number_types_as_bigints(false);
        result
            .serialize(&serializer)
            .map_err(|e| JsError::new(&format!("encode result: {e}")))
    }

    pub fn course_count(&self) -> u32 {
        self.index.n_docs
    }

    /// Look up a single course by its dashed code (e.g. `"15-122"`). Returns
    /// `null` when no such course exists.
    pub fn course_by_code(&self, code: &str) -> Result<JsValue, JsError> {
        let Some(&id) = self.index.code_to_id.get(code) else {
            return Ok(JsValue::NULL);
        };
        self.serialize_course(id)
    }

    /// Look up a single course by its dense numeric id (the value inside
    /// each [`Hit`]). Returns `null` when the id is out of range.
    pub fn course_by_id(&self, id: u32) -> Result<JsValue, JsError> {
        if (id as usize) >= self.index.courses.len() {
            return Ok(JsValue::NULL);
        }
        self.serialize_course(id)
    }

    fn serialize_course(&self, id: u32) -> Result<JsValue, JsError> {
        let course = &self.index.courses[id as usize];
        let serializer = serde_wasm_bindgen::Serializer::new()
            .serialize_maps_as_objects(true)
            .serialize_large_number_types_as_bigints(false);
        course
            .serialize(&serializer)
            .map_err(|e| JsError::new(&format!("encode course: {e}")))
    }
}
