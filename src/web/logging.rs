#![allow(dead_code)]

use axum::{
    http::{self},
    response::Response,
};
use tracing::{Level, Span};

#[derive(Clone, Copy)]
pub(crate) struct HoofMakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for HoofMakeSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let method = request.method().to_string();
        let uri = request.uri().path();
        tracing::span!(Level::INFO, "request", %method, %uri, status = tracing::field::Empty, time = tracing::field::Empty)
    }
}

impl<B> tower_http::trace::OnResponse<B> for HoofMakeSpan {
    fn on_response(self, _response: &Response<B>, _duration: std::time::Duration, _span: &Span) {
        // let status = response.status();
        // span.record("status", &tracing::field::display(status.as_u16()));
        // span.record("time", &tracing::field::display(duration.as_millis()));
    }
}
