use ironrdp::connector::{ClientConnector, ConnectionResult, Credentials, DesktopSize};
use ironrdp::pdu::gcc;
use ironrdp::pdu::rdp::capability_sets::MajorPlatformType;
use ironrdp::pdu::rdp::client_info::{PerformanceFlags, TimezoneInfo};
use ironrdp_tokio::TokioFramed;
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tracing::info;

use crate::config::ConnectionProfile;
use crate::error::{RdpError, Result};

pub async fn establish_connection(
    profile: &ConnectionProfile,
) -> Result<(TokioFramed<TlsStream<TcpStream>>, ConnectionResult)> {
    let server_addr = profile.server_addr();
    info!("Connecting to proxy at {}", server_addr);

    let tcp_stream = TcpStream::connect(&server_addr)
        .await
        .map_err(|e| RdpError::Connection(format!("TCP connect failed: {e}")))?;

    let mut framed: TokioFramed<TcpStream> = TokioFramed::new(tcp_stream);

    let config = ironrdp::connector::Config {
        desktop_size: DesktopSize {
            width: profile.width,
            height: profile.height,
        },
        desktop_scale_factor: 0,
        enable_tls: true,
        enable_credssp: true,
        credentials: Credentials::UsernamePassword {
            username: profile.username.clone(),
            password: profile.password.clone(),
        },
        domain: None,
        client_build: 0,
        client_name: "RustRDP".to_string(),
        keyboard_type: gcc::KeyboardType::IbmEnhanced,
        keyboard_subtype: 0,
        keyboard_functional_keys_count: 12,
        keyboard_layout: 0x0409,
        ime_file_name: String::new(),
        bitmap: None,
        dig_product_id: String::new(),
        client_dir: String::new(),
        platform: MajorPlatformType::WINDOWS,
        hardware_id: None,
        request_data: None,
        autologon: false,
        enable_audio_playback: false,
        performance_flags: PerformanceFlags::default(),
        license_cache: None,
        timezone_info: TimezoneInfo::default(),
        enable_server_pointer: true,
        pointer_software_rendering: false,
    };

    let local_addr = tcp_stream_local_addr(&server_addr)?;
    let server_name = profile.hostname.clone();
    let mut connector = ClientConnector::new(config, local_addr);

    let should_upgrade = ironrdp_tokio::connect_begin(&mut framed, &mut connector)
        .await
        .map_err(|e| RdpError::Connection(format!("RDP negotiation failed: {e}")))?;

    info!("TLS upgrade required, upgrading...");

    let initial_stream: TcpStream = framed.into_inner_no_leftover();

    let native_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| RdpError::Tls(format!("TLS connector build failed: {e}")))?;

    let tls_connector = tokio_native_tls::TlsConnector::from(native_connector);

    let tls_stream: TlsStream<TcpStream> = tls_connector
        .connect(&server_name, initial_stream)
        .await
        .map_err(|e| RdpError::Tls(format!("TLS handshake failed: {e}")))?;

    let server_public_key = extract_server_public_key(&tls_stream);

    let mut upgraded_framed: TokioFramed<TlsStream<TcpStream>> = TokioFramed::new(tls_stream);

    let upgraded = ironrdp_tokio::mark_as_upgraded(should_upgrade, &mut connector);

    info!("TLS upgrade complete, finalizing connection...");

    let mut network_client = ironrdp_tokio::reqwest::ReqwestNetworkClient::new();

    let connection_result = ironrdp_tokio::connect_finalize(
        upgraded,
        connector,
        &mut upgraded_framed,
        &mut network_client,
        ironrdp::connector::ServerName::new(server_name),
        server_public_key,
        None,
    )
    .await
    .map_err(|e| RdpError::Authentication(format!("Connection finalization failed: {e}")))?;

    info!("RDP connection established successfully");

    Ok((upgraded_framed, connection_result))
}

fn extract_server_public_key(tls_stream: &TlsStream<TcpStream>) -> Vec<u8> {
    tls_stream
        .get_ref()
        .peer_certificate()
        .ok()
        .flatten()
        .map(|cert| cert.to_der().unwrap_or_default())
        .unwrap_or_default()
}

fn tcp_stream_local_addr(server_addr: &str) -> Result<std::net::SocketAddr> {
    use std::net::ToSocketAddrs;
    let addr = server_addr
        .to_socket_addrs()
        .map_err(|e| RdpError::Connection(format!("Invalid server address: {e}")))?
        .next()
        .ok_or_else(|| RdpError::Connection("Could not resolve server address".to_string()))?;
    let local = if addr.is_ipv4() {
        std::net::SocketAddr::from(([0, 0, 0, 0], 0))
    } else {
        std::net::SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 0))
    };
    Ok(local)
}
