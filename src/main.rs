use std::sync::Arc;

use telegram_bot::{
    build_app_with_provisioning,
    domain::chat_provisioning_gateway::ChatProvisioningGateway,
    infrastructure::{
        disabled_chat_provisioning_gateway::DisabledChatProvisioningGateway,
        mtproto_chat_gateway::MtprotoChatGateway,
    },
    Config,
};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Configuration error: {err}");
            std::process::exit(1);
        }
    };

    if std::env::args().any(|arg| arg == "--mtproto-login") {
        let Some((api_id, api_hash)) = config.mtproto_credentials() else {
            eprintln!("TELEGRAM_API_ID and TELEGRAM_API_HASH must be set for --mtproto-login");
            std::process::exit(1);
        };
        if let Err(err) =
            MtprotoChatGateway::login_interactive(api_id, api_hash, &config.telegram_session_path)
                .await
        {
            eprintln!("MTProto login failed: {err}");
            std::process::exit(1);
        }
        return;
    }

    if config.api_key.is_none() {
        warn!("API_KEY is not set; /api/v1/* endpoints are running without authentication");
    } else {
        info!("API key authentication enabled");
    }

    let provisioning_gateway: Arc<dyn ChatProvisioningGateway> =
        if let Some((api_id, _)) = config.mtproto_credentials() {
            match MtprotoChatGateway::connect(api_id, &config.telegram_session_path).await {
                Ok(gw) => {
                    info!(session = %config.telegram_session_path, "MTProto provisioning enabled");
                    Arc::new(gw)
                }
                Err(err) => {
                    warn!("Failed to initialize MTProto gateway: {err}");
                    Arc::new(DisabledChatProvisioningGateway)
                }
            }
        } else {
            info!("MTProto credentials not set; channel/group creation endpoints disabled");
            Arc::new(DisabledChatProvisioningGateway)
        };

    let app = build_app_with_provisioning(&config, provisioning_gateway);
    let listener = TcpListener::bind(config.listen_addr)
        .await
        .expect("failed to bind listen address");

    info!(addr = %config.listen_addr, "telegram-bot listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
