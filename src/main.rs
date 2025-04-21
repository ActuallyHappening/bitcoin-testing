#![allow(unused_imports)]

mod prelude {
  pub use tracing::{debug, error, info, trace, warn};

  pub use color_eyre::eyre::WrapErr as _;
  pub use color_eyre::eyre::eyre;
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
  use std::net::IpAddr;

  use tokio::net::TcpStream;

  use crate::{conn, prelude::*, resolver};

  pub async fn main() -> color_eyre::Result<()> {
    let ips = resolver::get_ips().await?;
    debug!("Found IPs {:?}", ips);

    let ip = ips.get(0).cloned().ok_or(eyre!("No IPs found"))?;
    let conn = conn::BitcoinNodeConn::connect(ip)?;

    Ok(())
  }
}

mod conn {
  use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
  };

  use bitcoin::{
    consensus::{Decodable as _, Encodable},
    hex::{self, DisplayHex},
    p2p::{Address, ServiceFlags, message_network::VersionMessage},
  };

  use crate::prelude::*;

  pub struct BitcoinNodeConn {
    them: SocketAddr,
    stream: TcpStream,
  }

  impl BitcoinNodeConn {
    pub fn connect(ip: IpAddr) -> color_eyre::Result<Self> {
      let port = 8333;
      let addr = SocketAddr::new(ip, port);
      let stream = TcpStream::connect(addr).wrap_err("Couldn't connect to main bitcoin node")?;
      trace!(?ip, ?port, "Connected to node");
      let mut conn = Self { stream, them: addr };

      conn.send_version()?;
      let received = conn.receive_version()?;

      Ok(conn)
    }

    fn send_version(&mut self) -> color_eyre::Result<()> {
      let me_ip: SocketAddr = (Ipv4Addr::LOCALHOST, 8333).into();
      let me: Address = Address::new(&me_ip, ServiceFlags::NONE);

      let them = Address::new(&self.them, ServiceFlags::NETWORK);

      let message = bitcoin::p2p::message_network::VersionMessage {
        version: bitcoin::p2p::PROTOCOL_VERSION,
        services: ServiceFlags::NONE,
        timestamp: std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)?
          .as_secs()
          .try_into()?,
        receiver: them,
        sender: me,
        nonce: 0,
        user_agent: "Rust bitcoin testing".into(),
        start_height: 0,
        relay: false,
      };

      message.consensus_encode(&mut self.stream)?;
      Ok(())
    }

    fn receive_version(&mut self) -> color_eyre::Result<VersionMessage> {
      let mut buffer = [0; 64];
      self.stream.read_exact(&mut buffer)?;
      let dump = buffer.to_lower_hex_string();
      let str = String::from_utf8_lossy(&buffer);
      trace!(?buffer, ?dump, %str, "Debug read buffer");

      todo!()
      // let message =
      //   bitcoin::p2p::message_network::VersionMessage::consensus_decode(&mut self.stream)
      //     .wrap_err("Failed to decode version message")?;
      // trace!(?message, "Received version message");
      // Ok(message)
    }
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
