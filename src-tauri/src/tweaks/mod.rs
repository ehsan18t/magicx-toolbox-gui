//! The redesigned tweak engine. `model` is the one typed representation (spec §5/§6); later
//! tasks add authoring-surface parsers, kind implementations, the engine, and storage on top.

pub mod model;

pub use model::*;
