use baizekit_gen::cmd::Commands;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(cargo_generate::log_formatter)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false)
        .format_level(false)
        .target(env_logger::Target::Stdout)
        .init();

    dotenvy::dotenv().ok();
    Commands::parse().run()
}
