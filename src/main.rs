#![allow(unused_imports)]

mod prelude {
  pub use tracing::{debug, error, info, trace, warn};

  pub use color_eyre::eyre::WrapErr as _;
}
mod tracing;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
  crate::tracing::install_tracing("info,bitcoin_testing=trace")?;
  ::tracing::info!("Started tracing");

  main::main().await?;

  ::tracing::info!("Finished");
  Ok(())
}

mod main {
  use crate::{prelude::*, resolver};

  pub async fn main() -> color_eyre::Result<()> {
    let ips = resolver::get_ips().await?;
    debug!("Found IPs {:?}", ips);
    
    

    Ok(())
  }
}

mod resolver {
  use std::net::IpAddr;

  use hickory_resolver::{Resolver, config::ResolverConfig, name_server::TokioConnectionProvider};

  use crate::prelude::*;

  pub async fn get_ips() -> color_eyre::Result<Vec<IpAddr>> {
    let resolver = Resolver::builder_with_config(
      ResolverConfig::default(),
      TokioConnectionProvider::default(),
    )
    .build();

    let lookup = resolver
      .lookup_ip("seed.bitcoin.sipa.be")
      .await
      .wrap_err("Couldn't lookup default DNS")?;

    Ok(lookup.iter().collect())
  }
}
