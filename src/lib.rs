extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate bytes;
extern crate futures;

#[macro_use]
extern crate log;

#[cfg(test)]
extern crate tokio;

extern crate tokio_codec;
extern crate tokio_executor;
extern crate tokio_io;
extern crate tokio_tcp;
extern crate tokio_timer;

pub mod protocol;
