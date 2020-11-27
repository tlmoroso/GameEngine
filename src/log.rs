use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{Registry, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;

use thiserror::Error;
use anyhow::Result;
use crate::log::LogError::{InitLogTracerError, SetGlobalDefaultError};

pub fn init_logger() -> Result<()>{
    LogTracer::init().map_err(|e| {
        InitLogTracerError {
            source: e
        }
    })?;

    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();
    let file_appender = tracing_appender::rolling::daily("/", "game_engine.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking_writer);
    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);
    tracing::subscriber::set_global_default(subscriber).map_err(|e| {
        SetGlobalDefaultError {
            source: e
        }
    })?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum LogError {
    #[error("Error initializing log-forwarder for tracing")]
    InitLogTracerError {
        source: tracing_log::log_tracer::SetLoggerError
    },
    #[error("Error setting global default subscriber for tracing")]
    SetGlobalDefaultError {
        source: tracing::subscriber::SetGlobalDefaultError
    }
}