pub(crate) const GENERIC_SITE: &str = "Generic Site";

pub(crate) enum Urls {
    Home,
    Login,
    Logout,
    Scan,
    Create,
    Manifest,
    Static,
    CspReportOnly,
    Register,
}

impl AsRef<str> for Urls {
    fn as_ref(&self) -> &str {
        match self {
            Urls::Home => "/",
            Urls::Login => "/login",
            Urls::Logout => "/logout",
            Urls::Scan => "/scan",
            Urls::Create => "/create",
            Urls::Manifest => "/manifest.webmanifest",
            Urls::Static => "/static/",
            Urls::CspReportOnly => "/csp/reportOnly",
            Urls::Register => "/register",
        }
    }
}
