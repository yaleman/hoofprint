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

use askama::filters::HtmlSafe;
use serde::{Deserialize, Serialize};

use crate::{db::entities, error::HoofprintError};

pub mod cli;
pub mod config;
pub(crate) mod db;
pub mod error;
pub mod logging;
pub mod prelude;
pub mod web;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum Code {
    Bar,
    QR,
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Code::Bar => write!(f, "barcode"),
            Code::QR => write!(f, "qrcode"),
        }
    }
}

impl HtmlSafe for Code {}

impl Code {
    pub fn as_html(&self, value: &str) -> String {
        match self {
            Code::Bar => {
                format!(
                    r#"<svg class="code_image" id="barcode" data-value="{}"></svg>"#,
                    value
                )
            }
            Code::QR => {
                format!(
                    r#"<div class="code_image" id="qrcode" data-value="{}"></div>"#,
                    value
                )
            }
        }
    }
}

impl TryFrom<&entities::code::Model> for Code {
    type Error = HoofprintError;

    fn try_from(value: &entities::code::Model) -> Result<Self, Self::Error> {
        match value.type_.as_str() {
            "barcode" => Ok(Code::Bar),
            "qrcode" => Ok(Code::QR),
            _ => Err(HoofprintError::InvalidCodeType(value.type_.clone())),
        }
    }
}
