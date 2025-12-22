//! HoofPrint is a lightweight "this is my membership card" app.
//! It allows users to create codes (barcodes/QR codes) that can be scanned at various sites to prove membership or access rights.

#![deny(warnings)]
#![warn(unused_extern_crates)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]

use std::fmt::Display;

use askama::filters::HtmlSafe;

pub mod cli;
pub mod config;
pub(crate) mod db;
pub mod error;
pub mod logging;
pub mod prelude;
pub mod web;

#[derive(Clone, Debug)]
pub enum Code {
    Barcode(String),
    QRCode(String),
}

impl HtmlSafe for Code {}

impl Code {
    pub fn as_html(&self) -> String {
        match self {
            Code::Barcode(code) => {
                format!(
                    r#"<svg class="code_image" id="barcode" data-value="{}"></svg>"#,
                    code
                )
            }
            Code::QRCode(code) => {
                format!(
                    r#"<div class="code_image" id="qrcode" data-value="{}"></div>"#,
                    code
                )
            }
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Code::Barcode(code) => code,
            Code::QRCode(code) => code,
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_html())
    }
}
