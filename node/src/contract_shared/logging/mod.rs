//! A logger implementation which outputs log messages from CasperLabs crates to the terminal.

mod settings;
mod structured_message;
mod terminal_logger;

use std::{collections::BTreeMap, time::Duration};

use log::{self, Level, LevelFilter, Log, SetLoggerError};

pub use self::terminal_logger::TerminalLogger;
use crate::contract_shared::newtypes::CorrelationId;
pub use settings::{Settings, Style};

#[doc(hidden)]
pub const PAYLOAD_KEY: &str = "payload=";
pub(crate) const METRIC_METADATA_TARGET: &str = "METRIC";
pub(crate) const CASPERLABS_METADATA_TARGET: &str = "casperlabs_";
pub(crate) const MESSAGE_TEMPLATE_KEY: &str = "message_template";
pub(crate) const DEFAULT_MESSAGE_TEMPLATE: &str = "{message}";
pub(crate) const DEFAULT_MESSAGE_KEY: &str = "message";

/// Initializes the global logger using the given settings.
///
/// The logger will write all log messages from crates prefixed with "casperlabs_" to stdout, and
/// can also log internal metrics generated by the Execution Engine.
///
/// Returns an error if the global logger has already been set in this process.
pub fn initialize(settings: Settings) -> Result<(), SetLoggerError> {
    let logger = Box::new(TerminalLogger::new(&settings));
    initialize_with_logger(logger, settings)
}

/// This and the `TerminalLogger` are public but undocumented to allow functional testing of this
/// crate, e.g. by passing a logger composed of a `TerminalLogger`.
#[doc(hidden)]
pub fn initialize_with_logger(
    logger: Box<dyn Log>,
    settings: Settings,
) -> Result<(), SetLoggerError> {
    if settings.max_level() == LevelFilter::Off && !settings.enable_metrics() {
        // No logging required
        return Ok(());
    }

    log::set_boxed_logger(logger)?;
    log::set_max_level(settings.max_level());
    Ok(())
}

/// Logs a message using the given format and properties.
///
/// # Arguments
///
/// * `log_level` - log level of the message to be logged
/// * `message_format` - a message template to apply over properties by key
/// * `properties` - a collection of machine readable key / value properties which will be logged
#[inline]
pub fn log_details(
    _log_level: Level,
    _message_format: String,
    _properties: BTreeMap<&str, String>,
) {
    // TODO: Metrics story https://casperlabs.atlassian.net/browse/NDRS-120
}

/// Logs the duration of a specific operation.
///
/// # Arguments
///
/// * `correlation_id` - a shared identifier used to group metrics
/// * `metric` - the name of the metric
/// * `tag` - a grouping tag for the metric
/// * `duration` - in seconds
#[inline]
pub fn log_duration(correlation_id: CorrelationId, metric: &str, tag: &str, duration: Duration) {
    let duration_in_seconds: f64 = duration.as_secs_f64();

    log_metric(
        correlation_id,
        metric,
        tag,
        "duration_in_seconds",
        duration_in_seconds,
    )
}

/// Logs the details of the specified metric.
///
/// # Arguments
///
/// * `correlation_id` - a shared identifier used to group metrics
/// * `metric` - the name of the metric
/// * `tag` - a grouping tag for the metric
/// * `metric_key` - property key for metric's value
/// * `metric_value` - numeric value of metric
#[inline]
pub fn log_metric(
    _correlation_id: CorrelationId,
    _metric: &str,
    _tag: &str,
    _metric_key: &str,
    _metric_value: f64,
) {
    // TODO: Metrics story https://casperlabs.atlassian.net/browse/NDRS-120
}

/// Logs the metrics associated with the specified host function.
pub fn log_host_function_metrics(_host_function: &str, _properties: BTreeMap<&str, String>) {
    // TODO: Metrics story https://casperlabs.atlassian.net/browse/NDRS-120
}
