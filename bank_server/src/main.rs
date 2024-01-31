use log::LevelFilter;

use crate::network::server::Server;

pub mod network;
pub mod bank;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let bank_server = bank::server::BankServer::new().await?;
    let _ = Server::run_block("[::]:1234", bank_server).await?;


    Ok(())
}
