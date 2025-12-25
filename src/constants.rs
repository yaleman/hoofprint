pub(crate) enum Urls {
    Home,
    Login,
    Logout,
    Scan,
    Create,
    Manifest,
}

impl AsRef<str> for Urls {
    fn as_ref(&self) -> &str {
        match self {
            Urls::Home => "/",
            Urls::Login => "/login",
            Urls::Logout => "/logout",
            Urls::Scan => "/scan",
            Urls::Create => "/create",
            Urls::Manifest => "/manifest.json",
        }
    }
}
