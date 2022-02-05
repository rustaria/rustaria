extern crate core;

use opt::Verbosity;
use std::env;
use time::macros::format_description;
use tracing_error::ErrorLayer;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use crate::network::server::ServerNetwork;

pub const KERNEL_VERSION: (u8, u8, u8) = (0, 0, 1);

pub mod api;
pub mod chunk;
pub mod comps;
pub mod entity;
pub mod opt;
pub mod registry;
pub mod types;
pub mod world;
pub mod network;
mod blake3;

/// Common initialization code for both Rustaria client and dedicated server.
/// This currently sets up [`color_eyre`] and [`tracing`].
pub fn init(verbosity: Verbosity) -> eyre::Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    color_eyre::install()?;

    let timer = UtcTime::new(format_description!(
        "[hour]:[minute]:[second].[subsecond digits:3]"
    ));
    let format = tracing_subscriber::fmt::format()
        .with_timer(timer)
        .compact();
    let fmt_layer = tracing_subscriber::fmt::layer().event_format(format);

    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| {
        EnvFilter::try_new(match verbosity {
            Verbosity::Normal => "info,wgpu_hal=warn,wgpu_core=warn",
            Verbosity::Verbose => "debug,wgpu_hal=warn,wgpu_core=warn,naga=info",
            Verbosity::VeryVerbose => {
                "trace,wgpu_core::present=info,wgpu_core::device=info,wgpu_hal=info,naga=info"
            }
            Verbosity::VeryVeryVerbose => "trace",
        })
    })?;

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

pub struct Server {
    pub network: ServerNetwork,
}
