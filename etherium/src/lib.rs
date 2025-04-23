#![allow(unused_imports)]

mod prelude {
  pub use tracing::{debug, error, info, trace, warn};

  pub use color_eyre::eyre::WrapErr as _;
  pub use color_eyre::eyre::eyre;
}
pub mod tracing;

pub mod main {
  use crate::prelude::*;

  pub async fn main() -> color_eyre::Result<()> {
    todo!()
  }
}
