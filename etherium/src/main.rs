#![allow(unused_imports)]

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
  etherium_testing::tracing::install_tracing("info,bitcoin_testing=trace")?;
  ::tracing::info!("Started tracing");

  etherium_testing::main::main().await?;

  ::tracing::info!("Finished");
  Ok(())
}

pub mod main {}
