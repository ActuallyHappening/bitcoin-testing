#![allow(unused_imports)]

mod prelude {
    pub use tracing::{debug, error, info, trace, warn};
}
mod tracing;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    crate::tracing::install_tracing("info,bitcoin-testing=trace")?;
    ::tracing::info!("Started tracing");

    main::main().await?;

    Ok(())
}

mod main {
    use crate::prelude::*;

    pub async fn main() -> color_eyre::Result<()> {
        Ok(())
    }
}
