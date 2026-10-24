//! The catalog builder. `courses-index build` reads the raw inputs (the scraper's `exported/` tree and the
//! repository's `data/`) and writes `catalog.sqlite` (decision 15): the JSON documents the website loads, in
//! Courseweave's formats (decision 16), and relational tables of public fields.
//!
//! The modules port Courseweave's data scripts; each says which. [`js`] reproduces the JavaScript behavior they rely
//! on, so the documents match the files those scripts write.

pub mod catalog;
pub mod config;
pub mod finals;
pub mod js;
pub mod merge;
pub mod model;
pub mod pipeline;
pub mod requisite;
pub mod soc;
pub mod sources;
pub mod syllabi;
pub mod terms;
pub mod text;
pub mod titles;
