use crate::{Code, prelude::*};

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage {}

/// Homepage handler that returns a simple HTML response
#[instrument(level = "info")]
pub(crate) async fn homepage() -> Result<Html<String>, HoofprintError> {
    let homepage = HomePage {};
    Ok(Html(homepage.render()?))
}

#[derive(Template)]
#[template(path = "view_code.html")]
struct ViewCode {
    pub code: Code,
}

#[instrument(level = "info")]
pub(crate) async fn view_code(Path(code): Path<String>) -> Result<Html<String>, HoofprintError> {
    let test_code = Code::QRCode(code.to_string());

    let code_page = ViewCode { code: test_code };
    Ok(Html(code_page.render()?))
}
