#[macro_use]
extern crate derive_builder;

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate bytes;
extern crate rand;

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate tokio;

extern crate futures;
extern crate native_tls;
extern crate tokio_codec;
extern crate tokio_executor;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate tokio_timer;
extern crate tokio_tls;

#[macro_use]
mod error;

// TODO: Write client handling TCP stream
// TODO: Handle TLS stream
// TODO: Switch parsing to using `nom`

pub use self::error::*;
pub mod codec;
pub mod protocol;

pub mod net;

mod client;
pub use self::client::*;
