//! HoofPrint is a lightweight "this is my membership card" app.
//! It allows users to create codes (barcodes/QR codes) that can be scanned at various sites to prove membership or access rights.

pub mod cli;
pub mod config;
pub(crate) mod db;
pub mod logging;
pub mod prelude;
pub mod web;
