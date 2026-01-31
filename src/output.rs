/// Macro for outputting data to stdout (machine-readable output)
///
/// This macro is used for actual data/results that should go to stdout,
/// as opposed to diagnostic messages which should use info!(), error!(), etc.
/// and go to stderr.
///
/// # Examples
///
/// ```
/// output!("binary: repo version");
/// output!("{}\t{}", repo, version);
/// ```
///
/// # Unix Philosophy
///
/// - stdout: data/results (use output!())
/// - stderr: diagnostics/messages (use info!(), error!(), warn!(), debug!())
#[macro_export]
macro_rules! output {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}
