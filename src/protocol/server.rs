#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// The unique identifier of the NATS server
    server_id: String,
    /// The version of the NATS server
    version: String,
    /// The version of golang the NATS server was built with
    proto: u8,
    /// The IP address used to start the NATS server, by default this will be 0.0.0.0 and can be configured with `-client_advertise host:port`
    go: String,
    /// The port number the NATS server is configured to listen on
    host: String,
    /// Maximum payload size, in bytes, that the server will accept from the client.
    port: u32,
    /// An integer indicating the protocol version of the server. The server version 1.2.0 sets this to 1 to indicate that it supports the “Echo” feature.
    max_payload: u32,
    /// An optional unsigned integer (64 bits) representing the internal client identifier in the server. This can be used to filter client connections in monitoring, correlate with error logs, etc…
    client_id: Option<u64>,
    /// If this is set, then the client should try to authenticate upon connect.
    auth_required: Option<bool>,
    /// If this is set, then the client must perform the TLS/1.2 handshake. Note, this used to be ssl_required and has been updated along with the protocol from SSL to TLS.
    tls_required: Option<bool>,
    /// If this is set, the client must provide a valid certificate during the TLS handshake.
    tls_verify: Option<bool>,
    /// An optional list of server urls that a client can connect to.
    connect_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connect {
    /// Turns on +OK protocol acknowledgements.
    verbose: bool,
    /// Turns on additional strict format checking, e.g. for properly formed subjects
    pedantic: bool,
    /// Indicates whether the client requires an SSL connection.
    tls_required: bool,
    /// Client authorization token (if auth_required is set)
    auth_token: Option<String>,
    /// Connection username (if auth_required is set)
    user: Option<String>,
    /// Connection password (if auth_required is set)
    pass: Option<String>,
    /// Optional client name
    name: Option<String>,
    /// The implementation language of the client.
    lang: String,
    /// The version of the client.
    version: String,
    /// optional int. Sending 0 (or absent) indicates client supports original protocol. Sending 1 indicates that the client supports dynamic reconfiguration of cluster topology changes by asynchronously receiving INFO messages with known servers it can reconnect to.
    protocol: Option<u8>,
    /// Optional boolean. If set to true, the server (version 1.2.0+) will not send originating messages from this connection to its own subscriptions. Clients should set this to true only for server supporting this feature, which is when proto in the INFO protocol is set to at least 1.
    echo: Option<bool>,
}
