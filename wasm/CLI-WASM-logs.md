PS C:\Users\kanal\Documents\Github\SCMessenger> .\target\debug\scmessenger-cli.exe start
2026-05-10T10:25:56.641175Z  INFO scmessenger_cli: SCMessenger CLI starting up...
2026-05-10T10:25:56.641802Z  INFO scmessenger_cli: Log directory: C:\Users\kanal\AppData\Local\scmessenger\logs
2026-05-10T10:25:58.914858Z DEBUG sled::pagecache::iterator: ordering before clearing tears: {0: 0}, max_header_stable_lsn: 0
2026-05-10T10:25:58.915790Z DEBUG sled::pagecache::iterator: in clean_tail_tears, found missing item in tail: None and we'll scan segments {0: 0} above lowest lsn 0
2026-05-10T10:25:58.918042Z DEBUG sled::pagecache::iterator: filtering out segments after detected tear at (lsn, lid) 967
2026-05-10T10:25:58.918895Z DEBUG sled::pagecache::iterator: hit max_lsn 967 in iterator, stopping
2026-05-10T10:25:58.919699Z DEBUG sled::pagecache::snapshot: zeroing the end of the recovered segment at lsn 0 between lids 968 and 524287
2026-05-10T10:25:58.923780Z DEBUG sled::pagecache::blob_io: gc_blobs removing any blob with an lsn above 968
2026-05-10T10:25:58.925497Z DEBUG sled::pagecache::segment: SA starting with tip 524288 stable -1 free {}
2026-05-10T10:25:58.926147Z DEBUG sled::pagecache::iobuf: starting log at recovered active offset 968, recovered lsn 968
2026-05-10T10:25:58.926790Z DEBUG sled::pagecache::iobuf: starting IoBufs with next_lsn: 968 next_lid: 968
2026-05-10T10:25:58.927343Z DEBUG sled::pagecache: load_snapshot loading pages from 0..4
2026-05-10T10:25:58.932361Z DEBUG scmessenger_core::identity: IdentityManager::with_backend: Initializing with persistent storage
2026-05-10T10:25:58.932923Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loading from persistent store
2026-05-10T10:25:58.933355Z DEBUG scmessenger_core::identity::store: IdentityStore::load_nickname: Reading from sled for NICKNAME_KEY
2026-05-10T10:25:58.933844Z DEBUG scmessenger_core::identity::store: IdentityStore::load_nickname: NICKNAME_KEY not found in sled
2026-05-10T10:25:58.934218Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: No nickname found in store
2026-05-10T10:25:58.935533Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loaded keys from store
2026-05-10T10:25:58.936507Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loaded device_metadata=Some(DeviceMetadata { device_id: "ec9967f9-1df9-42ec-b160-9d2cde0a98a0", seniority_timestamp: 1777670931 })
2026-05-10T10:25:58.937569Z  INFO scmessenger_core::iron_core: Consent granted for identity initialization
2026-05-10T10:25:58.938253Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loading from persistent store
2026-05-10T10:25:58.938708Z DEBUG scmessenger_core::identity::store: IdentityStore::load_nickname: Reading from sled for NICKNAME_KEY
2026-05-10T10:25:58.939187Z DEBUG scmessenger_core::identity::store: IdentityStore::load_nickname: NICKNAME_KEY not found in sled
2026-05-10T10:25:58.939555Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: No nickname found in store
2026-05-10T10:25:58.940365Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loaded keys from store
2026-05-10T10:25:58.941098Z DEBUG scmessenger_core::identity: IdentityManager::hydrate_from_store: Loaded device_metadata=Some(DeviceMetadata { device_id: "ec9967f9-1df9-42ec-b160-9d2cde0a98a0", seniority_timestamp: 1777670931 })
2026-05-10T10:25:58.941661Z  INFO scmessenger_core::identity: 🔑 Loaded existing identity
2026-05-10T10:25:58.943647Z  INFO scmessenger_core::iron_core: Identity initialized: Some("2136776c6f5fdbdc1dc0875bd0a87f4005646a1239c5440e3c79c4636e2bcd7c")
[IDENTITY_DIAG] to_libp2p_peer_id OK: 12D3KooWE8DHKCNSrB2NoxvzyCtuoTSBnmikC3vW4u1qXZdhBHaZ
2026-05-10T10:25:58.957186Z DEBUG sled::pagecache::iterator: ordering before clearing tears: {0: 0}, max_header_stable_lsn: 0
2026-05-10T10:25:58.957835Z DEBUG sled::pagecache::iterator: in clean_tail_tears, found missing item in tail: None and we'll scan segments {0: 0} above lowest lsn 0
2026-05-10T10:25:58.960087Z DEBUG sled::pagecache::iterator: filtering out segments after detected tear at (lsn, lid) 95
2026-05-10T10:25:58.961592Z DEBUG sled::pagecache::iterator: hit max_lsn 95 in iterator, stopping
2026-05-10T10:25:58.962659Z DEBUG sled::pagecache::snapshot: zeroing the end of the recovered segment at lsn 0 between lids 96 and 524287
2026-05-10T10:25:58.967838Z DEBUG sled::pagecache::blob_io: gc_blobs removing any blob with an lsn above 96
2026-05-10T10:25:58.968919Z DEBUG sled::pagecache::segment: SA starting with tip 524288 stable -1 free {}
2026-05-10T10:25:58.969590Z DEBUG sled::pagecache::iobuf: starting log at recovered active offset 96, recovered lsn 96
2026-05-10T10:25:58.970150Z DEBUG sled::pagecache::iobuf: starting IoBufs with next_lsn: 96 next_lid: 96
2026-05-10T10:25:58.970899Z DEBUG sled::pagecache: load_snapshot loading pages from 0..4
2026-05-10T10:25:58.975361Z  INFO scmessenger_cli::ledger: 📒 Loaded connection ledger: 0 known peers
SCMessenger — Starting...

Identity: 2136776c6f5fdbdc1dc0875bd0a87f4005646a1239c5440e3c79c4636e2bcd7c
Public Key: 4001276dd2f16774ca67d10244529f8bd1a2a36b716b2a3889da7e58f13adf6a
Landing Page:  http://127.0.0.1:9000
WebSocket:     ws://127.0.0.1:9000/ws
P2P Listener:  /ip4/0.0.0.0/tcp/9001
WASM Bridge:   /ip4/0.0.0.0/tcp/9002/ws
📒 Ledger: 0 peers (0 bootstrap, 0 reachable, 0 in backoff)

✓ Peer ID: 12D3KooWE8DHKCNSrB2NoxvzyCtuoTSBnmikC3vW4u1qXZdhBHaZ

2026-05-10T10:25:58.983753Z  INFO scmessenger_cli::server: Warp HTTP+WS server listening on ws://127.0.0.1:9000
2026-05-10T10:25:59.025670Z  INFO scmessenger_core::transport::behaviour: mDNS LAN discovery: enabled (libp2p-mdns)
2026-05-10T10:25:59.027405Z  INFO libp2p_swarm: local_peer_id=12D3KooWE8DHKCNSrB2NoxvzyCtuoTSBnmikC3vW4u1qXZdhBHaZ
2026-05-10T10:25:59.028532Z DEBUG igd_next::aio::tokio: sending broadcast request to: 239.255.255.250:1900 on interface: Ok(0.0.0.0:52085)
2026-05-10T10:25:59.029063Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::relay::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::relay::RelayPhase<libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>>>>::with_relay_client<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, std::io::error::Error>::{{closure}}> address=/ip4/0.0.0.0/tcp/9001
2026-05-10T10:25:59.033633Z DEBUG libp2p_websocket::framed: Address is not a websocket multiaddr address=/ip4/0.0.0.0/tcp/9001
2026-05-10T10:25:59.034138Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}> address=/ip4/0.0.0.0/tcp/9001
2026-05-10T10:25:59.035982Z DEBUG libp2p_tcp: listening on address address=0.0.0.0:9001
2026-05-10T10:25:59.039023Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::relay::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::relay::RelayPhase<libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>>>>::with_relay_client<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, std::io::error::Error>::{{closure}}> address=/ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.041901Z DEBUG libp2p_websocket::framed: Address is not a websocket multiaddr address=/ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.042639Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}> address=/ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.044502Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}> address=/ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.045653Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}> address=/ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.049604Z  WARN scmessenger_core::transport::swarm: ✗ Failed to bind QUIC listener /ip4/0.0.0.0/udp/0/quic: Multiaddr is not supported: /ip4/0.0.0.0/udp/0/quic
2026-05-10T10:25:59.050230Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::relay::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::relay::RelayPhase<libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>>>>::with_relay_client<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, std::io::error::Error>::{{closure}}> address=/ip4/0.0.0.0/tcp/9002/ws
2026-05-10T10:25:59.053672Z DEBUG libp2p_tcp: listening on address address=0.0.0.0:9002
2026-05-10T10:25:59.055981Z  INFO scmessenger_core::transport::swarm: ✓ Bound WebSocket listener /ip4/0.0.0.0/tcp/9002/ws
2026-05-10T10:25:59.056655Z DEBUG libp2p_gossipsub::behaviour: Subscribing to topic topic=sc-lobby
2026-05-10T10:25:59.057480Z DEBUG libp2p_gossipsub::behaviour: Running JOIN for topic topic=sc-lobby
2026-05-10T10:25:59.058241Z DEBUG libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:25:59.058881Z DEBUG libp2p_gossipsub::behaviour: JOIN: Inserting 0 random peers into the mesh
2026-05-10T10:25:59.060021Z DEBUG libp2p_gossipsub::behaviour: Completed JOIN for topic topic=sc-lobby
2026-05-10T10:25:59.060576Z DEBUG libp2p_gossipsub::behaviour: Subscribed to topic topic=sc-lobby
2026-05-10T10:25:59.061019Z  INFO scmessenger_core::transport::swarm: 📡 Subscribed to lobby topic: sc-lobby
2026-05-10T10:25:59.061439Z DEBUG libp2p_gossipsub::behaviour: Subscribing to topic topic=sc-mesh
2026-05-10T10:25:59.061973Z DEBUG libp2p_gossipsub::behaviour: Running JOIN for topic topic=sc-mesh
2026-05-10T10:25:59.062777Z DEBUG libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:25:59.063257Z DEBUG libp2p_gossipsub::behaviour: JOIN: Inserting 0 random peers into the mesh
2026-05-10T10:25:59.064098Z DEBUG libp2p_gossipsub::behaviour: Completed JOIN for topic topic=sc-mesh
2026-05-10T10:25:59.064863Z DEBUG libp2p_gossipsub::behaviour: Subscribed to topic topic=sc-mesh
2026-05-10T10:25:59.065451Z  INFO scmessenger_core::transport::swarm: 📡 Subscribed to mesh topic: sc-mesh
2026-05-10T10:25:59.066044Z DEBUG libp2p_gossipsub::behaviour: Subscribing to topic topic=sc-receipt-convergence
2026-05-10T10:25:59.066480Z DEBUG libp2p_gossipsub::behaviour: Running JOIN for topic topic=sc-receipt-convergence
2026-05-10T10:25:59.067137Z DEBUG libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:25:59.067729Z DEBUG libp2p_gossipsub::behaviour: JOIN: Inserting 0 random peers into the mesh
2026-05-10T10:25:59.068379Z DEBUG libp2p_gossipsub::behaviour: Completed JOIN for topic topic=sc-receipt-convergence
2026-05-10T10:25:59.068884Z DEBUG libp2p_gossipsub::behaviour: Subscribed to topic topic=sc-receipt-convergence
2026-05-10T10:25:59.069550Z  INFO scmessenger_core::transport::swarm: 📡 Subscribed to delivery convergence topic: sc-receipt-convergence
2026-05-10T10:25:59.070285Z  INFO scmessenger_core::transport::swarm: === OWN_IDENTITY: 12D3KooWE8DHKCNSrB2NoxvzyCtuoTSBnmikC3vW4u1qXZdhBHaZ ===
2026-05-10T10:25:59.083444Z DEBUG sled::pagecache::iterator: ordering before clearing tears: {0: 0}, max_header_stable_lsn: 0
2026-05-10T10:25:59.084063Z DEBUG sled::pagecache::iterator: in clean_tail_tears, found missing item in tail: None and we'll scan segments {0: 0} above lowest lsn 0
2026-05-10T10:25:59.086038Z DEBUG sled::pagecache::iterator: filtering out segments after detected tear at (lsn, lid) 95
2026-05-10T10:25:59.086871Z DEBUG sled::pagecache::iterator: hit max_lsn 95 in iterator, stopping
2026-05-10T10:25:59.087650Z DEBUG sled::pagecache::snapshot: zeroing the end of the recovered segment at lsn 0 between lids 96 and 524287
2026-05-10T10:25:59.091723Z DEBUG sled::pagecache::blob_io: gc_blobs removing any blob with an lsn above 96
2026-05-10T10:25:59.092887Z DEBUG sled::pagecache::segment: SA starting with tip 524288 stable -1 free {}
2026-05-10T10:25:59.093453Z DEBUG sled::pagecache::iobuf: starting log at recovered active offset 96, recovered lsn 96
2026-05-10T10:25:59.093913Z DEBUG sled::pagecache::iobuf: starting IoBufs with next_lsn: 96 next_lid: 96
2026-05-10T10:25:59.094353Z DEBUG sled::pagecache: load_snapshot loading pages from 0..4
2026-05-10T10:25:59.105139Z  INFO Swarm::poll: libp2p_mdns::behaviour::iface: creating instance on iface address address=192.168.0.222
2026-05-10T10:25:59.116452Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/192.168.0.222/tcp/9002
2026-05-10T10:25:59.117481Z DEBUG Swarm::poll: libp2p_websocket::framed: Listening on address address=/ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.118321Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(3) address=/ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.119758Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.120823Z DEBUG libp2p_core::transport::choice: Failed to listen on address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::relay::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::relay::RelayPhase<libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>>>>::with_relay_client<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, std::io::error::Error>::{{closure}}> address=/ip4/0.0.0.0/tcp/9002/ws
2026-05-10T10:25:59.123957Z DEBUG libp2p_tcp: listening on address address=0.0.0.0:9002
✓ WebSocket P2P Bridge started on /ip4/0.0.0.0/tcp/9002/ws
✓ Network started
2026-05-10T10:25:59.126453Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/127.0.0.1/tcp/9002

Commands:
  send <contact> <message>
  contacts
  peers
  status
  quit

2026-05-10T10:25:59.127770Z DEBUG Swarm::poll: libp2p_websocket::framed: Listening on address address=/ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.131584Z  INFO scmessenger_cli::ble_mesh: BLE: GATT advertising stub started for service df0100001000800000805f9b34fb (Awaiting full platform advertising support).

⚙ Aggressive Discovery — dialing known peers...
2026-05-10T10:25:59.127563Z  INFO scmessenger_cli::ble_daemon: btleplug: Bluetooth manager created successfully
2026-05-10T10:25:59.131730Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(3) address=/ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.131541Z  INFO scmessenger_cli::ble_mesh: BLE: CLI GATT central for service df0100001000800000805f9b34fb (peripheral advertising via btleplug not enabled).
✓ Control API: http://127.0.0.1:9876
2026-05-10T10:25:59.134417Z DEBUG Swarm::poll: libp2p_upnp::behaviour: multiaddress not supported for UPnP /ip4/127.0.0.1/tcp/9002/ws
✓ Listening on /ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.135709Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/127.0.0.1/tcp/9002/ws
✓ Listening on /ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.136612Z  INFO scmessenger_cli::api: Control API listening on 127.0.0.1:9876
2026-05-10T10:25:59.144563Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/192.168.0.222/tcp/9002
2026-05-10T10:25:59.145207Z DEBUG Swarm::poll: libp2p_websocket::framed: Listening on address address=/ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.145924Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(4) address=/ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.146818Z DEBUG Swarm::poll: libp2p_upnp::behaviour: port from multiaddress is already being mapped multiaddress=/ip4/192.168.0.222/tcp/9002/ws mapped_multiaddress=/ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.147496Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/192.168.0.222/tcp/9002/ws
✓ Listening on /ip4/192.168.0.222/tcp/9002/ws
2026-05-10T10:25:59.149804Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:25:59.150349Z DEBUG Swarm::poll: libp2p_websocket::framed: Listening on address address=/ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.150915Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(4) address=/ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.151446Z DEBUG Swarm::poll: libp2p_upnp::behaviour: multiaddress not supported for UPnP /ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.152105Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/127.0.0.1/tcp/9002/ws
✓ Listening on /ip4/127.0.0.1/tcp/9002/ws
2026-05-10T10:25:59.152809Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:25:59.163009Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/192.168.0.222/tcp/9001
2026-05-10T10:25:59.163700Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(1) address=/ip4/192.168.0.222/tcp/9001
2026-05-10T10:25:59.164592Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/192.168.0.222/tcp/9001
✓ Listening on /ip4/192.168.0.222/tcp/9001
2026-05-10T10:25:59.165480Z DEBUG Swarm::poll: libp2p_tcp: New listen address address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:25:59.166291Z DEBUG Swarm::poll: libp2p_swarm: New listener address listener=ListenerId(1) address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:25:59.166910Z DEBUG Swarm::poll: libp2p_upnp::behaviour: multiaddress not supported for UPnP /ip4/127.0.0.1/tcp/9001
2026-05-10T10:25:59.167511Z  INFO scmessenger_core::transport::swarm: Listening on /ip4/127.0.0.1/tcp/9001
✓ Listening on /ip4/127.0.0.1/tcp/9001
2026-05-10T10:25:59.227171Z  INFO scmessenger_cli::ble_daemon: btleplug: acquired Bluetooth manager; 1 adapter(s) visible
2026-05-10T10:25:59.227725Z DEBUG scmessenger_cli::ble_daemon: btleplug adapter: Adapter { manager: AdapterManager { peripherals: {}, events_channel: broadcast::Sender } }
2026-05-10T10:25:59.268631Z  INFO scmessenger_cli::ble_mesh: BLE scan active (wide open, manually filtering to SCM service 0000df01-0000-1000-8000-00805f9b34fb)
2026-05-10T10:25:59.319142Z  INFO Swarm::poll: libp2p_mdns::behaviour: discovered peer on address peer=12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB address=/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:25:59.320084Z  INFO scmessenger_core::transport::swarm: mDNS discovered peer: 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB at /ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:25:59.321320Z DEBUG scmessenger_core::transport::swarm: 🔄 Activated SyncSession for peer: 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB

✓ Peer: 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
> 2026-05-10T10:25:59.323546Z  INFO scmessenger_cli::transport_bridge: Registered peer 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB with capabilities: [Internet, Local]
2026-05-10T10:25:59.324378Z  INFO scmessenger_cli: Registered transport capabilities for 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB: [Internet, Local], reachable=true
2026-05-10T10:25:59.325349Z  INFO scmessenger_core::transport::swarm: 📒 Sharing ledger with 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB (0 entries)
2026-05-10T10:25:59.326905Z DEBUG Swarm::poll: libp2p_core::transport::choice: Failed to dial address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_relay::priv_client::transport::Transport, libp2p_core::transport::upgrade::Builder<libp2p_relay::priv_client::transport::Transport>::authenticate<libp2p_relay::priv_client::Connection, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::relay::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::relay::RelayPhase<libp2p_core::transport::map::Map<libp2p_core::transport::choice::OrTransport<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>, libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}>>>>::with_relay_client<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_relay::priv_client::Connection>>>>, std::io::error::Error>::{{closure}}> address=/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:25:59.330756Z DEBUG Swarm::poll: libp2p_core::transport::choice: Failed to dial address using libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>, libp2p_core::transport::upgrade::Builder<libp2p_websocket::WsConfig<libp2p_dns::Transport<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, hickory_resolver::async_resolver::AsyncResolver<hickory_resolver::name_server::connection_provider::GenericConnector<hickory_resolver::name_server::connection_provider::tokio_runtime::TokioRuntimeProvider>>>>>::authenticate<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::websocket::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::websocket::WebsocketPhase<libp2p_core::transport::map::Map<libp2p_core::transport::upgrade::Multiplexed<libp2p_core::transport::and_then::AndThen<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>, libp2p_core::transport::upgrade::Authenticated<libp2p_core::transport::and_then::AndThen<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>, libp2p_core::transport::upgrade::Builder<libp2p_tcp::Transport<libp2p_tcp::provider::tokio::Tcp>>::authenticate<libp2p_tcp::provider::tokio::TcpStream, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Config, libp2p_noise::Error>::{{closure}}>>::multiplex<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_yamux::Config, std::io::error::Error>::{{closure}}>>, libp2p::builder::phase::tcp::<impl libp2p::builder::SwarmBuilder<libp2p::builder::phase::provider::Tokio, libp2p::builder::phase::tcp::TcpPhase>>::with_tcp<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<libp2p_tcp::provider::tokio::TcpStream>>>>, std::io::error::Error>::{{closure}}>>>>::with_websocket<libp2p_noise::Config::new, libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>, libp2p_noise::Error, <libp2p_yamux::Config as core::default::Default>::default, libp2p_yamux::Muxer<multistream_select::negotiated::Negotiated<libp2p_noise::io::Output<multistream_select::negotiated::Negotiated<rw_stream_sink::RwStreamSink<libp2p_websocket::BytesConnection<libp2p_tcp::provider::tokio::TcpStream>>>>>>, std::io::error::Error>::{{closure}}::{{closure}}> address=/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:25:59.332305Z DEBUG Swarm::poll: libp2p_tcp: dialing address address=192.168.0.110:9001
2026-05-10T10:25:59.431889Z DEBUG sled::pagecache::iobuf: advancing offset within the current segment from 968 to 1303
2026-05-10T10:25:59.437548Z DEBUG sled::pagecache::iobuf: wrote lsns 968-1302 to disk at offsets 968-1302, maxed false complete_len 335
2026-05-10T10:25:59.438000Z DEBUG sled::pagecache::iobuf: mark_interval(968, 335)
2026-05-10T10:25:59.438534Z DEBUG sled::pagecache::iobuf: new highest interval: 968 - 1302
2026-05-10T10:26:00.004551Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:26:00.005271Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:00.009285Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:00.269083Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:00.270079Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:00.274276Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:01.090892Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:01.091852Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:01.096251Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:01.638456Z DEBUG Swarm::poll: libp2p_kad::behaviour: Last remaining address of peer is unreachable. peer=12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB address=/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:26:01.639610Z DEBUG Swarm::poll: libp2p_swarm: Connection attempt to peer failed with Transport([(/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB, Other(Custom { kind: Other, error: Other(Right(Right(Left(Left(Os { code: 10061, kind: ConnectionRefused, message: "No connection could be made because the target machine actively refused it." }))))) }))]). peer=12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB
2026-05-10T10:26:01.641084Z DEBUG scmessenger_core::transport::swarm: ⚠ Outgoing connection error to 12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB: Failed to negotiate transport protocol(s): [(/ip4/192.168.0.110/tcp/9001/p2p/12D3KooWEoerqxtKdVcP8RToASVzbAk8ou9hYzbH9jcuY8v3KoBB: : No connection could be made because the target machine actively refused it. (os error 10061): No connection could be made because the target machine actively refused it. (os error 10061))]
2026-05-10T10:26:01.769063Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:01.769799Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:01.773762Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:02.970606Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:02.971466Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ServicesAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), services: [0000fddf-0000-1000-8000-00805f9b34fb] }
2026-05-10T10:26:02.972032Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:02.975766Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:04.032104Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:04.033079Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:04.033777Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:04.034274Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:04.034837Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:04.035333Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:04.035992Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:04.036526Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:04.037092Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:04.037451Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:04.038322Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:05.099194Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:05.100182Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:05.104532Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:07.103614Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:07.104568Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:07.108246Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:08.748041Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:08.748675Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:08.751987Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:09.023234Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:09.024059Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:09.024525Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:09.025045Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:09.025652Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:09.026290Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:09.026818Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:09.027410Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:09.027940Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:09.028459Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:09.029009Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:09.039372Z DEBUG Swarm::poll: libp2p_upnp::behaviour: could not find gateway: IO error: search timed out
2026-05-10T10:26:09.106328Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:09.107231Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:09.110926Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:11.119875Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:11.120690Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:11.124166Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:11.352614Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:26:11.353316Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:11.357265Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:12.408655Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:26:12.409532Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:12.412929Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:12.769193Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:12.770070Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:12.773774Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:13.126436Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:13.127243Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:13.131313Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:13.357674Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:26:13.358608Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:13.363864Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:14.023844Z DEBUG Swarm::poll: libp2p_autonat::behaviour::as_client: Outbound dial-back request aborted: No qualified server
2026-05-10T10:26:14.024981Z DEBUG scmessenger_core::transport::swarm: AutoNAT outbound probe: Error { probe_id: ProbeId(0), peer: None, error: NoServer }
2026-05-10T10:26:14.025540Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:14.026030Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:14.026708Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:14.027300Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:14.027784Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:14.028453Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:14.029044Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:14.029531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:14.030179Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:14.030683Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:14.031320Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:14.415047Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:26:14.415840Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:14.419829Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:15.123754Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:15.124513Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:15.128290Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:16.416508Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:26:16.417214Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:16.421910Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:26:16.547407Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:16.548886Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:16.553187Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:17.127177Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:17.128139Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:17.132552Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:17.253093Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:17.253832Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:17.259004Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:19.035105Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:19.035897Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:19.036403Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:19.037139Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:19.037793Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:19.038298Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:19.038705Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:19.039106Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:19.039554Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:19.039930Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:19.040332Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:19.137657Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:26:19.138370Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:19.142065Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:26:22.579505Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:22.582322Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:22.585444Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:24.021243Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:24.022084Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:24.022514Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:24.023169Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:24.023594Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:24.023975Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:24.024356Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:24.024780Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:24.025270Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:24.025839Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:24.026372Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:25.394201Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:26:25.395007Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:25.399458Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:26:25.758728Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:25.759556Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:25.764010Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:29.034385Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:29.035170Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:29.035663Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:29.036317Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:29.036854Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:29.037324Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:29.037802Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:29.038367Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:29.038868Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:29.039259Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:29.039678Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:31.774910Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:31.775621Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:31.780123Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:32.371621Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:32.372756Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:32.376539Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:32.738677Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:32.739534Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:32.743965Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:34.029305Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:34.030240Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:34.030716Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:34.031134Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:34.031552Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:34.032009Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:34.032480Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:34.032975Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:34.033451Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:34.033753Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:34.034060Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:37.249869Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:37.257698Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:37.262148Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:38.409869Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:38.410934Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:38.414835Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:38.754928Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:38.755813Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:38.760112Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:39.023984Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:39.025034Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:39.026117Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:39.027175Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:39.032160Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:39.034178Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:39.034524Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:39.035255Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:39.036628Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:39.037141Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:39.037553Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:39.942611Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:26:39.943329Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceDiscovered(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:39.948031Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:41.950719Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:26:41.951521Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:41.955320Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:43.953159Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:26:43.953979Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:43.958322Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:44.028510Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:44.029181Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:44.029672Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:44.030171Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:44.030812Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:44.031361Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:44.031890Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:44.032650Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:44.033199Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:44.033848Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:44.034521Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:47.250280Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:47.250959Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:47.255069Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:48.214562Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:48.215329Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:48.219585Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:49.030359Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:49.031005Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:49.031545Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:49.032066Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:49.032568Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:49.033108Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:49.033700Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:49.034064Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:49.034517Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:49.034917Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:49.035317Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:49.733358Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:49.734058Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:49.738345Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:49.965221Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:26:49.966061Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:49.970272Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:51.975435Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:26:51.976590Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:51.980518Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:26:52.003759Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:52.006865Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:52.009187Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:52.108715Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:26:52.109334Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:52.113822Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:54.027582Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:54.028213Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:54.028567Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:54.029056Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:54.029478Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:54.029860Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:54.030231Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:54.030826Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:54.031224Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:54.031627Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:54.032395Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:55.766498Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:26:55.773991Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:55.778848Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:26:55.780773Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:55.781935Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:55.784059Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:56.114135Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:26:56.114883Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:56.119640Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:58.025247Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:26:58.026782Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:58.030468Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:26:58.119731Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:26:58.120453Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:58.125882Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:26:59.026943Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:26:59.027520Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:26:59.028445Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:59.029036Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:59.029769Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:26:59.030286Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:59.030699Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:59.031056Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:26:59.031417Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:26:59.031880Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:26:59.032297Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:26:59.109139Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:26:59.137936Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:27:00.119970Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:27:00.120813Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:27:00.125030Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:27:00.280407Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:00.281540Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:00.284936Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:01.795791Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:01.796484Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:01.797801Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:02.747437Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:02.748364Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:02.754150Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:04.032574Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:04.033242Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:04.033687Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:04.034130Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:04.034480Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:04.035093Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:04.035524Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:04.035877Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:04.036387Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:04.037034Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:04.037459Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:07.479530Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:27:07.480250Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:27:07.484599Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:27:07.825710Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:07.826421Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:07.830597Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:08.760435Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:08.761226Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:08.765632Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:09.027115Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:09.027976Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:09.028493Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:09.028931Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:09.029549Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:09.029923Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:09.030317Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:09.030882Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:09.031243Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:09.031777Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:09.032276Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:10.544402Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:10.545182Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:10.549266Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:12.548572Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:12.549651Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:12.553945Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:14.022261Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:14.022898Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:14.023402Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:14.024104Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:14.024676Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:14.025155Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:14.025693Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:14.026178Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:14.026701Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:14.027329Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:14.027946Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:15.379733Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:15.380452Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:15.385491Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:16.557265Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:16.558330Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:16.562493Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:18.556783Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:18.558256Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:18.561648Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:19.023087Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:19.024321Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:19.024937Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:19.025532Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:19.026001Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:19.026501Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:19.026970Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:19.027583Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:19.028045Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:19.028561Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:19.029115Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:19.137373Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:19.149331Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:19.152027Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:19.155465Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:20.567492Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:20.568565Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:20.572792Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:21.764052Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:21.764681Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:21.768636Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:22.572576Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:27:22.573600Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:22.578281Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:27:24.028601Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:24.029296Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:24.029984Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:24.030405Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:24.030797Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:24.031202Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:24.031564Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:24.031927Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:24.032471Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:24.032939Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:24.033362Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:25.528298Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:27:25.529139Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:27:25.533224Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:27:25.758869Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:25.759512Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:25.764012Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:27.315935Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:27:27.317511Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:27.321584Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:28.960228Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:28.963667Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:28.964778Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:29.034981Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:29.035726Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:29.036412Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:29.037112Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:29.037636Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:29.038184Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:29.038791Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:29.039435Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:29.040161Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:29.040696Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:29.041109Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:32.795817Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:32.813765Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:32.814367Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:34.034848Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:34.035662Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:34.036232Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:34.037230Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:34.037885Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:34.038448Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:34.039048Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:34.039558Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:34.042164Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:34.042677Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:34.043247Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:34.987012Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:34.987776Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:34.991989Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:35.345403Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:27:35.346343Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:35.349199Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:39.032514Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:39.033374Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:39.034098Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:39.036163Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:39.036958Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:39.037943Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:39.038313Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:39.038763Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:39.039759Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:39.040155Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:39.040729Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:41.361251Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:27:41.362062Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:41.365622Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:43.366584Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:27:43.367598Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:43.370571Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:44.024241Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:44.025517Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:44.025891Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:44.026667Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:44.027249Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:44.027660Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:44.028050Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:44.028396Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:44.028814Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:44.029186Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:44.029753Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:44.030657Z DEBUG Swarm::poll: libp2p_autonat::behaviour::as_client: Outbound dial-back request aborted: No qualified server
2026-05-10T10:27:44.031201Z DEBUG scmessenger_core::transport::swarm: AutoNAT outbound probe: Error { probe_id: ProbeId(1), peer: None, error: NoServer }
2026-05-10T10:27:44.798181Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:44.803832Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:44.804452Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:47.055047Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:47.057177Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:47.059974Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:47.372664Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:27:47.373548Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:47.376759Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:27:49.030650Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:49.031505Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:49.032172Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:49.032853Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:49.033393Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:49.034009Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:49.034547Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:49.035144Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:49.035797Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:49.036451Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:49.037107Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:50.127048Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:27:50.127835Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:50.130886Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:50.827619Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:50.829272Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:50.831274Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:52.338892Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:27:52.339497Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:52.343142Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:27:54.029447Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:54.030938Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:54.031447Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:54.032048Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:54.032767Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:54.033271Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:54.033739Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:54.034435Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:54.035138Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:54.035670Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:54.036392Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:54.127376Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:27:54.128045Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:54.131479Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:54.243769Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:27:54.244677Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:54.248413Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:27:56.132163Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:27:56.133010Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:56.138202Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:58.138998Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:27:58.140053Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:58.143145Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:27:59.027580Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:27:59.028243Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:27:59.028801Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:59.029316Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:59.029802Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:27:59.030386Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:59.030880Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:59.031375Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:27:59.031859Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:27:59.032527Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:27:59.032937Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:27:59.106890Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:27:59.154526Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:28:00.122639Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:00.621485Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:00.622108Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:00.625710Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:02.143875Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:28:02.144690Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:02.148052Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:02.755041Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:02.756059Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:02.759803Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:04.024511Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:04.025249Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:04.025843Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:04.026497Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:04.026973Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:04.027457Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:04.027835Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:04.028321Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:04.028689Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:04.029083Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:04.029473Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:04.299684Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:04.300618Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:04.304133Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:06.156977Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:28:06.157803Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:06.161675Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:06.645730Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:06.646838Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:06.650177Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:08.772619Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:08.773606Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:08.776753Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:09.023134Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:09.024025Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:09.024687Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:09.025223Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:09.025743Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:09.026206Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:09.026684Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:09.027281Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:09.027680Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:09.028052Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:09.028638Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:10.425468Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:10.428961Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:10.429805Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:12.314305Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:12.315152Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:12.318586Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:12.680398Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:12.682636Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:12.685864Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:14.024092Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:14.024804Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:14.025433Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:14.025893Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:14.026517Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:14.027084Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:14.027669Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:14.028455Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:14.029038Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:14.029598Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:14.030237Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:14.324798Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:14.325602Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:14.329539Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:16.330270Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:16.331063Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:16.334526Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:17.286240Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:17.286981Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:17.290675Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:18.333222Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:18.333926Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:18.337123Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:19.027318Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:19.028437Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:19.028937Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:19.029434Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:19.029989Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:19.030766Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:19.031287Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:19.031937Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:19.032575Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:19.032944Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:19.033409Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:20.335604Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:20.336770Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:20.340122Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:21.649271Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:28:21.649967Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:28:21.654067Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:28:22.585448Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:22.727696Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:22.728533Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:22.731987Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:24.024951Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:24.026050Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:24.026432Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:24.026868Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:24.027290Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:24.027794Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:24.028199Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:24.028724Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:24.029156Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:24.029764Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:24.030280Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:24.256909Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:24.258612Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:24.261247Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:26.359303Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:28:26.361448Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:26.364590Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:28:29.024321Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:29.025192Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:29.025658Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:29.026082Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:29.026618Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:29.027093Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:29.027523Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:29.027998Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:29.028529Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:29.028977Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:29.029461Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:30.757777Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:30.758768Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:30.761402Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:32.273315Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:32.275693Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:32.276505Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:32.759236Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:32.760065Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:32.763243Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:32.768376Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:32.769013Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:32.772476Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:34.035729Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:34.036641Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:34.037408Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:34.038037Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:34.038617Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:34.039150Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:34.039716Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:34.040244Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:34.040774Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:34.041167Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:34.041658Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:34.544017Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:34.550687Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:34.556918Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:34.763529Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:34.764414Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:34.767867Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:38.777970Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:38.778751Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:38.781883Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:39.033294Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:39.034388Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:39.035962Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:39.036661Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:39.037303Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:39.039186Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:39.039610Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:39.040045Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:39.040527Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:39.041093Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:39.041615Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:39.836593Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:39.837544Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:39.840912Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:40.787758Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:40.788761Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:40.791998Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:43.506406Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:28:43.507163Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:43.510559Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:43.612123Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:43.612732Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:43.616983Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:44.019811Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:44.020656Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:44.021309Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:44.021821Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:44.022339Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:44.022959Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:44.023561Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:44.024090Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:44.024679Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:44.025246Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:44.025842Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:44.793253Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 33]} }
2026-05-10T10:28:44.794248Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:44.797256Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:28:45.514685Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:28:45.515506Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:45.519706Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:45.889557Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:45.893481Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:45.895192Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:47.279652Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:47.280494Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:47.283971Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:47.516735Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:28:47.517572Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:47.520889Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:49.033188Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:49.033981Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:49.034599Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:49.035113Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:49.035597Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:49.036160Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:49.036665Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:49.037405Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:49.038257Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:49.039386Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:49.039924Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:49.519187Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:28:49.519934Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:49.523648Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:28:52.269892Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:28:52.271010Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:52.274103Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:28:53.434714Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:53.442947Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:53.443736Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:54.023386Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:54.024185Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:54.024652Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:54.025196Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:54.025754Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:54.026347Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:54.026821Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:54.027502Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:54.028032Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:54.028534Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:54.028986Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:54.251048Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:28:54.251780Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:54.256737Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:28:55.712971Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:28:55.713777Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:28:55.716952Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:28:59.031607Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:28:59.032540Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:28:59.033047Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:59.033735Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:59.034234Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:28:59.034829Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:59.035328Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:59.035718Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:28:59.036132Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:28:59.036621Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:28:59.037236Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:28:59.110498Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:28:59.158688Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:28:59.454979Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:28:59.455962Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:28:59.459256Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:03.228820Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:03.230722Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:03.234767Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:04.022790Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:04.023402Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:04.024078Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:04.024662Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:04.025217Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:04.025907Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:04.026451Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:04.027087Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:04.027683Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:04.028171Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:04.028711Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:06.304857Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:06.305709Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:06.309840Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:07.252449Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:07.253330Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:07.256911Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:08.310051Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:08.310902Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:08.314113Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:09.024286Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:09.024863Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:09.025560Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:09.026114Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:09.026589Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:09.027109Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:09.027556Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:09.028037Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:09.028581Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:09.029023Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:09.029567Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:10.321426Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:10.322672Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:10.324866Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:12.327771Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:12.328617Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:12.331811Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:14.026163Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:14.026825Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:14.027450Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:14.027918Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:14.028424Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:14.028925Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:14.029449Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:14.029995Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:14.030573Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:14.031159Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:14.031730Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:14.041868Z DEBUG Swarm::poll: libp2p_autonat::behaviour::as_client: Outbound dial-back request aborted: No qualified server
2026-05-10T10:29:14.042888Z DEBUG scmessenger_core::transport::swarm: AutoNAT outbound probe: Error { probe_id: ProbeId(2), peer: None, error: NoServer }
2026-05-10T10:29:14.243399Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:14.244264Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:14.247542Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:14.336175Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:14.338137Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:14.341348Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:14.470507Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:14.471501Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:14.475365Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:15.755831Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:15.759484Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:15.761455Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:15.767981Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:29:15.768840Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:15.774150Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:16.339438Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:16.340287Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:16.344175Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:17.547563Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:17.548536Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:17.552142Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:19.023903Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:19.024642Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:19.025145Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:19.025618Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:19.026076Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:19.026850Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:19.027439Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:19.027964Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:19.028471Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:19.028890Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:19.029533Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:19.058646Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:19.059331Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:19.063490Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:21.766454Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:21.767275Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:21.771072Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:22.492192Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:22.493014Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:22.496396Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:22.834899Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:22.839697Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:22.840402Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:23.771666Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:29:23.772339Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:23.776069Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:24.035358Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:24.036069Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:24.036601Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:24.037142Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:24.037823Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:24.038362Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:24.038875Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:24.039258Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:24.039904Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:24.040337Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:24.040931Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:24.366167Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:24.366930Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:24.370697Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:24.499556Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:24.500431Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:24.504622Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:25.777461Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:29:25.778325Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:25.781644Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:26.378856Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:26.380030Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:26.383483Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:28.381339Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 219]} }
2026-05-10T10:29:28.382153Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:28.385575Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:29:29.030695Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:29.031599Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:29.032199Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:29.032738Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:29.033272Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:29.033871Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:29.034450Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:29.035015Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:29.035454Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:29.035924Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:29.037856Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:29.793127Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:29:29.793743Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:29.796847Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:30.269020Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:30.269904Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:30.273255Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:30.526749Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:30.527298Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:30.534601Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:31.601928Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 62]} }
2026-05-10T10:29:31.602881Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:29:31.606755Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:29:31.806315Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 66]} }
2026-05-10T10:29:31.807908Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:31.811660Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:32.526196Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:32.527008Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:32.530171Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:32.657610Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:32.659616Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:32.660165Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:34.020909Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:34.021504Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:34.021956Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:34.022392Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:34.022848Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:34.023534Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:34.024065Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:34.024712Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:34.025155Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:34.025583Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:34.026067Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:34.535771Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:34.537748Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:34.540448Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:36.423932Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:36.426028Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:36.427750Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:39.030931Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:39.031853Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:39.032399Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:39.033159Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:39.033660Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:39.034274Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:39.034841Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:39.035495Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:39.035982Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:39.036482Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:39.037235Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:40.550336Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:40.551100Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:40.554760Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:42.447542Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:42.448483Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:42.452173Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:42.550343Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 33]} }
2026-05-10T10:29:42.551231Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:42.555037Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:43.627196Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:29:43.627805Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:43.631073Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:29:44.021608Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:44.022409Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:44.022954Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:44.023458Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:44.024014Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:44.024667Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:44.025323Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:44.025857Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:44.026532Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:44.027113Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:44.027718Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:44.228613Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:44.229718Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:44.233323Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:44.722902Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:44.723650Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:44.727233Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:44.795250Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:29:48.476881Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:29:48.478904Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:48.480617Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:48.488836Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:48.489459Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:48.492639Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:49.031131Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:49.031818Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:49.032779Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:49.033304Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:49.033785Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:49.034239Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:49.034730Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:49.035223Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:49.035614Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:49.036216Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:49.036633Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:49.998989Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:50.000025Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:50.002734Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:50.250556Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:50.251327Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:50.254807Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:50.485625Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:29:50.486436Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:50.490113Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:52.264775Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:29:52.267349Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:52.268760Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:29:54.035005Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:54.035839Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:54.036312Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:54.036758Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:54.037221Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:54.037738Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:54.038194Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:54.038754Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:54.039238Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:54.039720Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:54.040180Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:56.505637Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:29:56.506298Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:56.509761Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:58.502461Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:29:58.504838Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:58.507145Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:29:58.754506Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:29:58.755260Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:58.758821Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:29:59.035572Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:29:59.036459Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:29:59.036894Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:59.037431Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:59.038039Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:29:59.038547Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:59.039069Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:59.039520Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:29:59.040006Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:29:59.040420Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:29:59.040987Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:29:59.100240Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:29:59.163168Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:30:00.281490Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:00.282334Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:00.286355Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:00.513034Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:30:00.513750Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:30:00.517663Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:30:02.287453Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:02.288253Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:02.292375Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:03.591627Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:03.593612Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:03.598079Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:04.029719Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:04.030182Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:04.030824Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:04.031557Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:04.032054Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:04.032481Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:04.033104Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:04.033605Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:04.034098Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:04.034427Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:04.034766Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:04.292525Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:04.293266Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:04.297896Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:05.854348Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:05.857271Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:05.857955Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:06.299166Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:06.300029Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:06.303507Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:08.303525Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:08.304483Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:08.307908Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:09.030941Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:09.031591Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:09.032223Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:09.032803Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:09.033281Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:09.033828Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:09.034369Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:09.035051Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:09.035577Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:09.036264Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:09.036767Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:09.616856Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:09.619576Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:09.624862Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:13.394563Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:13.395592Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:13.398721Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:14.032608Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:14.033318Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:14.033913Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:14.034554Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:14.035136Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:14.035701Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:14.036277Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:14.036994Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:14.037514Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:14.038049Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:14.038567Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:14.225848Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:30:14.226701Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:14.230601Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:15.658391Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:15.659430Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:15.662838Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:19.035574Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:19.036499Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:19.037114Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:19.037643Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:19.038243Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:19.038857Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:19.039454Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:19.039942Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:19.040392Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:19.040862Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:19.041353Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:24.023288Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:24.024072Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:24.024738Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:24.025289Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:24.025843Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:24.026345Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:24.026930Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:24.027541Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:24.028236Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:24.028733Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:24.029486Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:26.987032Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:26.987969Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:26.992164Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:27.224663Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:30:27.225440Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:27.229446Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:29.024542Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:29.025250Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:29.025872Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:29.026440Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:29.026972Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:29.027542Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:29.028082Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:29.028552Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:29.028993Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:29.029492Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:29.029961Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:29.253899Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:29.255879Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:29.257757Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:31.604935Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:34.033391Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:34.034478Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:34.035188Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:34.035754Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:34.036348Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:34.037106Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:34.037617Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:34.038020Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:34.038627Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:34.039137Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:34.039686Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:35.274997Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:35.276184Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:35.279951Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:38.929474Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:38.930229Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:38.934010Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:39.025791Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:39.029872Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:39.031829Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:39.033820Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:39.034320Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:39.034670Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:39.034840Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:39.035034Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:39.037268Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:39.038224Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:39.038577Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:39.040943Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:39.041585Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:39.045535Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:41.752635Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:30:41.753488Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:41.757053Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:42.813773Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:42.818280Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:42.839829Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:43.635156Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:30:44.020470Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:44.021225Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:44.021753Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:44.022328Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:44.022857Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:44.023405Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:44.024112Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:44.024538Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:44.025206Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:44.025841Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:44.026535Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:44.052300Z DEBUG Swarm::poll: libp2p_autonat::behaviour::as_client: Outbound dial-back request aborted: No qualified server
2026-05-10T10:30:44.053382Z DEBUG scmessenger_core::transport::swarm: AutoNAT outbound probe: Error { probe_id: ProbeId(3), peer: None, error: NoServer }
2026-05-10T10:30:44.944566Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:44.945342Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:44.948808Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:46.948972Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:46.949707Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:46.953411Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:48.961259Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:48.962034Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:48.966506Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:49.034301Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:49.034966Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:49.035596Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:49.036132Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:49.036682Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:49.037245Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:49.037688Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:49.038145Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:49.038709Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:49.039336Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:49.039929Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:49.668811Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:30:49.669744Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:49.672938Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:50.244802Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:30:50.245627Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:50.248991Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:50.965826Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:50.966842Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:50.970853Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:51.100263Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:30:51.101007Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:51.105326Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:30:53.678706Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:30:53.679684Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:53.683442Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:54.031501Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:54.032369Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:54.033033Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:54.033663Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:54.034310Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:54.035085Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:54.035625Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:54.036204Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:54.036603Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:54.037115Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:54.037669Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:54.982693Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:54.983478Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:54.987349Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:55.680901Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:30:55.681791Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:55.685606Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:56.402301Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:30:56.403230Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:56.407468Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:30:56.651896Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:30:56.652735Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:30:56.656316Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:30:56.981491Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:56.982486Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:56.986151Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:57.232565Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:30:57.233368Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:57.238270Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:30:57.681615Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:30:57.682458Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:57.686138Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:58.993050Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:30:58.993759Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:58.998131Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:30:59.022800Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:30:59.023512Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:30:59.024002Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:59.024552Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:59.025262Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:30:59.026095Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:59.026699Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:59.027336Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:30:59.027888Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:30:59.028458Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:30:59.029028Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:30:59.100079Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:30:59.179705Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:30:59.691439Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:30:59.692313Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:30:59.695566Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:00.414754Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:31:00.415601Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:00.419783Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:01.698420Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:31:01.699293Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:01.702962Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:01.735662Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:01.736591Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:01.740831Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:01.837139Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:01.838200Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:01.841553Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:02.416390Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:02.418841Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:02.421165Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:02.660225Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:31:02.660962Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:31:02.665324Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:31:03.002215Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:31:03.003261Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:03.006300Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:04.032655Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:04.033977Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:04.035002Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:04.035533Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:04.036146Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:04.036848Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:04.037287Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:04.037895Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:04.038360Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:04.038855Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:04.039565Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:04.417341Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:31:04.418897Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:04.423147Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:05.005370Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:31:05.006053Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:05.009782Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:05.844835Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:05.845542Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:05.849600Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:07.016341Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:31:07.017373Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:07.022043Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:08.450764Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:08.451815Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:08.455369Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:09.023764Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:09.024479Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:09.025009Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:09.025531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:09.026081Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:09.026557Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:09.026906Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:09.027294Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:09.027802Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:09.028386Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:09.028852Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:09.860581Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:09.861399Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:09.865411Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:10.218408Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:10.219356Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:10.223400Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:10.678273Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:31:10.679264Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:31:10.682800Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:31:11.862681Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:11.863551Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:11.867526Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:12.217857Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:12.218609Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:12.224549Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:14.028579Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:14.029379Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:14.029911Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:14.030486Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:14.030942Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:14.031386Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:14.031766Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:14.032285Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:14.032787Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:14.033494Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:14.034089Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:15.878778Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:15.879567Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:15.882055Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:18.249029Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:18.249820Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:18.253675Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:18.710602Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:18.711259Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:18.714773Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:19.032319Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:19.033127Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:19.033521Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:19.034235Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:19.034785Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:19.035277Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:19.035686Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:19.036185Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:19.036695Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:19.037173Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:19.037729Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:19.892712Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:31:19.893664Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:19.896711Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:31:20.506333Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:20.506916Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:20.510437Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:21.965992Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/64866 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:21.977357Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:21.979670Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/64866 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:21.981527Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/50419 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:21.985121Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/61675 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:23.068149Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:23.074100Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/50419 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:23.079252Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/53374 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:23.080186Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:23.080803Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/61675 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:23.085742Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:23.087284Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/53374 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:23.090286Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/61969 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:23.092746Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:23.093268Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/61969 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:24.034798Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:24.035398Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:24.035880Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:24.036207Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:24.036682Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:24.037280Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:24.037618Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:24.037965Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:24.038340Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:24.038683Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:24.039425Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:28.046147Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:28.046890Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:28.051301Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:28.129143Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/59633 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:28.133002Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/53015 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:31:28.136241Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:31:28.137150Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/59633 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:29.026771Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:29.027498Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:29.028101Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:29.028842Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:29.029449Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:29.030071Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:29.030508Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:29.030996Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:29.031446Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:29.031835Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:29.032398Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:32.564627Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:32.565632Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:32.570205Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:34.025117Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:34.025677Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:34.026200Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:34.026871Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:34.027488Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:34.028118Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:34.028643Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:34.029355Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:34.029815Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:34.030238Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:34.030681Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:34.202120Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:34.202873Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:34.207314Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:38.146645Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Timeout }))
2026-05-10T10:31:38.147523Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/53015 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:38.742346Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/55502 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:38.747455Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Left(Left(Left(Handshake(HeaderNotFound("Upgrade"))))))) }))
2026-05-10T10:31:38.748363Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/55502/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:38.755989Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/63672 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:38.757822Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/50709 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:39.028689Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:39.029544Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:39.030222Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:39.030958Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:39.031574Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:39.032251Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:39.033046Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:39.033907Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:39.034467Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:39.035221Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:39.035728Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:39.841600Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Left(Left(Left(Handshake(HeaderNotFound("Upgrade"))))))) }))
2026-05-10T10:31:39.842839Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/63672/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:39.857777Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Left(Left(Left(Handshake(HeaderNotFound("Upgrade"))))))) }))
2026-05-10T10:31:39.858370Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/50709/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:39.860164Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/55190 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:39.864096Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Left(Left(Left(Handshake(HeaderNotFound("Upgrade"))))))) }))
2026-05-10T10:31:39.864574Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/55190/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:40.105566Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:40.106497Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:40.110307Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:42.706637Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:42.707327Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:42.711009Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:43.883938Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:43.884944Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:43.888919Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:44.023867Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:44.024835Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:44.025697Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:44.026238Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:44.026817Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:44.027442Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:44.028083Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:44.028761Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:44.029392Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:44.029986Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:44.030880Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:44.911924Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/55481 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:44.920101Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Left(Left(Left(Handshake(HeaderNotFound("Upgrade"))))))) }))
2026-05-10T10:31:44.921898Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/55481/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:44.923879Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/63978 local_address=/ip4/127.0.0.1/tcp/9002
2026-05-10T10:31:48.726947Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:31:48.727708Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:48.731557Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:31:49.034311Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:49.034952Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:49.035602Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:49.036240Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:49.036774Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:49.037315Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:49.038509Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:49.039060Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:49.039620Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:49.040096Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:49.040831Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:50.517592Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:31:50.518977Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:50.523673Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:53.694518Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:53.695170Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:53.699379Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:53.814814Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:31:53.816085Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:53.820130Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:54.029455Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:54.030294Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:54.030956Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:54.031488Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:54.032023Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:54.032683Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:54.033392Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:54.034021Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:54.034532Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:54.035000Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:54.035495Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:54.526465Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:31:54.527206Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:54.530626Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:31:54.933689Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Timeout }))
2026-05-10T10:31:54.934907Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/63978/ws -> /ip4/127.0.0.1/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:31:55.115647Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:31:55.116512Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:55.120343Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:55.823964Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:31:55.825171Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:55.828060Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:57.130991Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:31:57.132369Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:57.138296Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:31:57.470158Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:31:57.470953Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:57.475468Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:31:57.833790Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:31:57.841169Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:57.844181Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:31:59.030146Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:31:59.031054Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:31:59.031577Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:59.032266Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:59.032897Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:31:59.033531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:59.034088Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:59.034519Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:31:59.035065Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:31:59.035531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:31:59.036303Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:31:59.107812Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:31:59.186939Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:32:01.135499Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:32:01.136396Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:32:01.140913Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:32:02.553543Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:32:02.554504Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:02.559314Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:03.134427Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:32:03.135364Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:32:03.140085Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:32:03.499040Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:03.499994Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:03.504110Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:03.849394Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:32:03.850721Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:03.855134Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:04.026844Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:04.027567Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:04.028275Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:04.028727Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:04.029414Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:04.030055Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:04.030632Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:04.031152Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:04.031711Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:04.032482Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:04.032927Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:04.201872Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:04.202684Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:04.207522Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:04.563133Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:32:04.563990Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:04.568587Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:05.850495Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:32:05.851404Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:05.855934Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:06.563893Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:32:06.564713Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:06.569880Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:07.275768Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:07.276499Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:07.280567Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:07.853783Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:32:07.854546Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:07.859807Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:08.573654Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:32:08.574506Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:08.579089Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:32:08.809590Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:32:08.810317Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:08.815341Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:09.028277Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:09.029088Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:09.029885Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:09.030605Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:09.031369Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:09.031970Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:09.032647Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:09.033214Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:09.033748Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:09.034210Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:09.034856Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:09.862654Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:32:09.863556Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:09.868217Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:32:10.233614Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:10.234474Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:10.239350Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:10.818258Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:32:10.819085Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:10.823778Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:11.052072Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:11.052785Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:11.057539Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:12.702860Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20418: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:12.703734Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:12.708765Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:14.025182Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:14.025833Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:14.026483Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:14.026998Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:14.027633Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:14.028234Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:14.028773Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:14.029322Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:14.029898Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:14.030348Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:14.031009Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:14.056178Z DEBUG Swarm::poll: libp2p_autonat::behaviour::as_client: Outbound dial-back request aborted: No qualified server
2026-05-10T10:32:14.056971Z DEBUG scmessenger_core::transport::swarm: AutoNAT outbound probe: Error { probe_id: ProbeId(4), peer: None, error: NoServer }
2026-05-10T10:32:14.829886Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:32:14.830808Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:14.837228Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:32:14.846794Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:14.847710Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:14.855346Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:18.724822Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20418: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:18.725557Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:18.729690Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:19.024397Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:19.025531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:19.026113Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:19.026715Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:19.027352Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:19.027878Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:19.028325Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:19.028630Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:19.029046Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:19.029462Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:19.029853Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:19.900948Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:32:24.032248Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:24.033040Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:24.033534Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:24.034230Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:24.034715Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:24.035224Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:24.035696Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:24.036287Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:24.036872Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:24.037357Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:24.037875Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:24.640574Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:24.641738Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:24.645140Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:25.704203Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:25.705047Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:25.709321Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:26.896089Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:26.896777Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:26.902137Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:29.026708Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:29.027378Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:29.028016Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:29.028553Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:29.028975Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:29.029730Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:29.030238Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:29.030991Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:29.031571Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:29.032302Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:29.033092Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:30.083252Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(F2:B8:BB:F6:BE:D0), manufacturer_data: {76: [18, 25, 16, 83, 32, 76, 133, 135, 57, 0, 142, 73, 124, 45, 92, 191, 177, 240, 34, 53, 250, 50, 62, 233, 78, 3, 179]} }
2026-05-10T10:32:30.084081Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:32:30.088949Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(F2:B8:BB:F6:BE:D0))
2026-05-10T10:32:31.728897Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:31.729714Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:31.734489Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:34.021043Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:34.021782Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:34.022378Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:34.022851Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:34.023297Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:34.023780Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:34.024120Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:34.024645Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:34.025006Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:34.025447Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:34.025979Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:38.215898Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:38.216884Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:38.221256Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:38.719727Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:38.720595Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:38.725099Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:39.033933Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:39.034685Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:39.035223Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:39.035955Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:39.036496Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:39.037095Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:39.037681Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:39.038340Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:39.038801Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:39.039241Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:39.039755Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:39.613102Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/56111 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:32:39.619125Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/56112 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:32:39.621013Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:32:39.621721Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/56111 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:32:39.623507Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/63042 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:32:39.630273Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:32:39.631228Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/63042 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:32:39.641783Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:32:39.642527Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/56112 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:32:40.223017Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:40.223910Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:40.228451Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:40.616279Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/56920 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:32:40.619750Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:32:40.620411Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/56920 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:32:40.621903Z DEBUG Swarm::poll: libp2p_tcp: Incoming connection from remote at local remote_address=/ip4/127.0.0.1/tcp/59623 local_address=/ip4/127.0.0.1/tcp/9001
2026-05-10T10:32:40.625419Z DEBUG Swarm::poll: libp2p_swarm: Incoming connection failed: Transport(Other(Custom { kind: Other, error: Other(Right(Right(Left(Right(Select(ProtocolError(InvalidMessage))))))) }))
2026-05-10T10:32:40.626247Z  WARN scmessenger_core::transport::swarm: ⚠ Incoming connection error from /ip4/127.0.0.1/tcp/59623 -> /ip4/127.0.0.1/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
2026-05-10T10:32:44.025430Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:44.026681Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:44.027203Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:44.027800Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:44.028375Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:44.029040Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:44.029531Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:44.030168Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:44.030861Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:44.031489Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:44.032115Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:44.239446Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:44.240193Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:44.245603Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:48.727584Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:DA:12:93:BA:25), manufacturer_data: {20162: [0, 15, 33, 51, 1]} }
2026-05-10T10:32:48.728305Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:48.733336Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:DA:12:93:BA:25))
2026-05-10T10:32:49.020884Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:49.021579Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:49.022280Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:49.023018Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:49.023459Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:49.023966Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:49.024467Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:49.025037Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:49.025814Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:49.026334Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:49.027055Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:50.281216Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:50.282067Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:50.286890Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:54.032700Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:54.033413Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:54.034269Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:54.034888Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:54.035818Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:54.036422Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:54.036996Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:54.037671Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:54.038289Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:54.038783Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:54.039459Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:56.315098Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:32:56.316381Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:56.320315Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:32:59.033438Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:32:59.034307Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:32:59.034975Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:59.035494Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:59.036226Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:32:59.036929Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:59.037503Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:59.038181Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:32:59.038856Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:32:59.039475Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:32:59.039884Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:32:59.110733Z  INFO scmessenger_core::transport::swarm: 📊 Relay custody audit log count: 0
2026-05-10T10:32:59.206201Z DEBUG scmessenger_cli::ledger: 📒 Saved ledger (0 entries)
2026-05-10T10:32:59.258463Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:32:59.259220Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:32:59.265009Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:01.263684Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:33:01.264845Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:01.269829Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:04.026860Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:33:04.027509Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:33:04.028347Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:04.028795Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:04.029285Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:33:04.029778Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:04.030509Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:04.031413Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:33:04.031888Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:04.032855Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:04.033424Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:33:05.268553Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:33:05.269511Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:05.274274Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:07.275686Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E1:56:70:4E:94:85), manufacturer_data: {76: [18, 25, 16, 208, 22, 7, 41, 80, 232, 90, 213, 138, 133, 209, 21, 115, 97, 110, 52, 135, 98, 224, 21, 73, 171, 2, 179]} }
2026-05-10T10:33:07.276435Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:07.280793Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E1:56:70:4E:94:85))
2026-05-10T10:33:08.580324Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:33:09.023707Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:33:09.024813Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:33:09.025358Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:09.025873Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:09.026418Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:33:09.026953Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:09.027901Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:09.028454Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:33:09.028965Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:09.029836Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:09.030503Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
2026-05-10T10:33:09.868148Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:33:09.888368Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(54:B7:E5:CF:42:46), manufacturer_data: {4096: []} }
2026-05-10T10:33:09.888942Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:33:09.893903Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(54:B7:E5:CF:42:46))
2026-05-10T10:33:10.959078Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:33:10.959776Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:33:10.965213Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:33:12.726912Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(CC:70:A4:07:32:F8), manufacturer_data: {76: [18, 25, 162, 1, 83, 20, 118, 217, 12, 209, 175, 142, 165, 200, 225, 16, 141, 6, 64, 29, 192, 7, 51, 242, 195, 0, 109]} }
2026-05-10T10:33:12.727587Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:33:12.732282Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(CC:70:A4:07:32:F8))
2026-05-10T10:33:12.963436Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(E8:C9:54:FF:BF:D3), manufacturer_data: {76: [18, 25, 16, 113, 92, 174, 9, 50, 154, 21, 244, 248, 233, 31, 143, 181, 19, 87, 109, 99, 235, 71, 16, 209, 243, 0, 34]} }
2026-05-10T10:33:12.964339Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:33:12.969290Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(E8:C9:54:FF:BF:D3))
2026-05-10T10:33:14.029057Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: ManufacturerDataAdvertisement { id: PeripheralId(C4:55:60:F9:21:C7), manufacturer_data: {76: [18, 25, 16, 29, 61, 48, 101, 175, 137, 103, 117, 57, 92, 64, 28, 33, 244, 218, 138, 25, 255, 46, 76, 69, 228, 1, 105]} }
2026-05-10T10:33:14.029930Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:33:14.032034Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Starting heartbeat
2026-05-10T10:33:14.032970Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-mesh
2026-05-10T10:33:14.034987Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:14.035482Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:14.036081Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-receipt-convergence
2026-05-10T10:33:14.036629Z DEBUG scmessenger_cli::ble_mesh: BLE central event received: DeviceUpdated(PeripheralId(C4:55:60:F9:21:C7))
2026-05-10T10:33:14.036648Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:14.037463Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:14.037932Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: HEARTBEAT: Mesh low. Topic contains: 0 needs: 1 topic=sc-lobby
2026-05-10T10:33:14.038460Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: RANDOM PEERS: Got 0 peers
2026-05-10T10:33:14.038860Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Updating mesh, new mesh: {}
2026-05-10T10:33:14.039402Z DEBUG Swarm::poll: libp2p_gossipsub::behaviour: Completed Heartbeat
