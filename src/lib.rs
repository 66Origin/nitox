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

#[macro_use]
extern crate log;

extern crate futures;
extern crate native_tls;
extern crate tokio_codec;
extern crate tokio_executor;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate tokio_timer;
extern crate tokio_tls;
extern crate url;

#[macro_use]
mod error;

// TODO: Add backoff mechanism for retry
// TODO: Find a way to integration test the reconnection mechanism
// TODO: Auto-pruning of unsub
// TODO: Handle verbose mode
// TODO: Handle pedantic mode
// TODO: Switch parsing to using `nom`

pub use self::error::*;
pub mod codec;
pub mod protocol;

pub mod net;

mod client;
pub use self::client::*;
