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
    let mut handles = Vec::with_capacity(ips.len());
    // for ip in ips {
    // let conn = conn::BitcoinNodeConn::connect(ip)?;
    handles.push(std::thread::spawn(move || -> color_eyre::Result<()> {
      let _ = conn::BitcoinNodeConn::connect(ip)?;

      Ok(())
    }));
    // }

    for handle in handles {
      handle.join().unwrap()?;
    }

    Ok(())
  }
}

mod conn {
  use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
  };

  use bitcoin::{
    consensus::{Decodable as _, Encodable},
    hex::{self, DisplayHex, FromHex},
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
      debug!(?ip, ?port, "Connected to node");
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

      trace!("Sending version message");
      message.consensus_encode(&mut self.stream)?;
      trace!("Sent version message");

      Ok(())
    }

    fn receive_version(&mut self) -> color_eyre::Result<VersionMessage> {
      // let mut buffer = [0; 85 + 1];
      // self.stream.read_exact(&mut buffer)?;
      // let dump = buffer.to_lower_hex_string();
      // let str = String::from_utf8_lossy(&buffer);
      // trace!(?buffer, ?dump, %str, "Debug read buffer");

      // use bitcoin::consensus::ReadExt as _;
      // let version = buffer[..].read_u32();

      // todo!()
      // let stream = &mut self.stream;
      let stream = &mut VecDeque::from(Vec::from_hex(
        "1f08b58a4d59bf0c127bfeb143420b46ba474e76e3bd7f5c811991b80ea18c1da545d1cbfee58e89bc90db5496b5aa704b25522f4d37322e39c3a5021c7f806abc74cafb4c5fe40decb8f7729544ea2b106cf6035972",
      )?);
      let message = bitcoin::p2p::message_network::VersionMessage::consensus_decode(stream)
        .wrap_err("Failed to decode version message")?;
      trace!(?message, "Received version message");
      Ok(message)
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
