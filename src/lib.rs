//! SOCKS proxy clients
#![doc(html_root_url="https://sfackler.github.io/rust-socks/doc/v0.2.1")]
#![warn(missing_docs)]

extern crate byteorder;
extern crate net2;

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

pub use v4::{Socks4Stream, Socks4Listener};
pub use v5::{Socks5Stream, Socks5Listener, Socks5Datagram};

mod v4;
mod v5;

/// A description of a connection target.
#[derive(Debug, Clone)]
pub enum TargetAddr {
    /// Connect to an IP address.
    Ip(SocketAddr),
    /// Connect to a fully qualified domain name.
    ///
    /// The domain name will be passed along to the proxy server and DNS lookup
    /// will happen there.
    Domain(String, u16),
}

/// A trait for objects that can be converted to `TargetAddr`.
pub trait ToTargetAddr {
    /// Converts the value of `self` to a a `TargetAddr`.
    fn to_target_addr(&self) -> io::Result<TargetAddr>;
}

impl ToTargetAddr for TargetAddr {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        Ok(self.clone())
    }
}

impl ToTargetAddr for SocketAddr {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        Ok(TargetAddr::Ip(*self))
    }
}

impl ToTargetAddr for SocketAddrV4 {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddr::V4(*self).to_target_addr()
    }
}

impl ToTargetAddr for SocketAddrV6 {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddr::V6(*self).to_target_addr()
    }
}

impl ToTargetAddr for (Ipv4Addr, u16) {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddrV4::new(self.0, self.1).to_target_addr()
    }
}

impl ToTargetAddr for (Ipv6Addr, u16) {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        SocketAddrV6::new(self.0, self.1, 0, 0).to_target_addr()
    }
}

impl<'a> ToTargetAddr for (&'a str, u16) {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        // try to parse as an IP first
        if let Ok(addr) = self.0.parse::<Ipv4Addr>() {
            return (addr, self.1).to_target_addr();
        }

        if let Ok(addr) = self.0.parse::<Ipv6Addr>() {
            return (addr, self.1).to_target_addr();
        }

        Ok(TargetAddr::Domain(self.0.to_owned(), self.1))
    }
}

impl<'a> ToTargetAddr for &'a str {
    fn to_target_addr(&self) -> io::Result<TargetAddr> {
        // try to parse as an IP first
        if let Ok(addr) = self.parse::<SocketAddrV4>() {
            return addr.to_target_addr();
        }

        if let Ok(addr) = self.parse::<SocketAddrV6>() {
            return addr.to_target_addr();
        }

        // split the string by ':' and convert the second part to u16
        let mut parts_iter = self.rsplitn(2, ':');
        let port_str = match parts_iter.next() {
            Some(s) => s,
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))
            }
        };

        let host = match parts_iter.next() {
            Some(s) => s,
            None => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))
            }
        };

        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid port value")),
        };

        (host, port).to_target_addr()
    }
}
