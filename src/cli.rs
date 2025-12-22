use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliOpts {
    #[clap(long)]
    pub debug: bool,

    #[clap(long, env = "HOOFPRINT_DB_FILE", default_value = "./hoofprint.sqlite")]
    pub database_file: String,

    #[clap(long, env = "HOOFPRINT_HOST", default_value = "127.0.0.1")]
    pub host: String,

    #[clap(long, env = "HOOFPRINT_PORT", default_value_t = 3000)]
    pub port: u16,
}
