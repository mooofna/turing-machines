//! The errors of the command-line tool and their exit codes.

use std::io;
use std::process::ExitCode;

/// Why a command failed.
///
/// Diagnostics of invalid machines are printed where they are found; the error only carries the
/// closing summary.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// A file could not be read or written.
    #[error("cannot {action} `{path}`: {source}")]
    File {
        /// `read` or `write`.
        action: &'static str,
        /// The path as given on the command line.
        path: String,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
    /// Writing to standard output or standard error failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// The arguments ask for something that cannot be done.
    #[error("{0}")]
    Usage(String),
    /// The machine description is invalid; its diagnostics have already been printed.
    #[error("could not load `{origin}` due to {summary}")]
    Invalid {
        /// Where the description came from.
        origin: String,
        /// For example `2 errors and 1 warning`.
        summary: String,
    },
    /// A check failed or a run produced no result.
    #[error("{0}")]
    Failed(String),
}

impl Error {
    /// A usage error with `message`.
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    /// A failure with `message`.
    pub(crate) fn failed(message: impl Into<String>) -> Self {
        Self::Failed(message.into())
    }

    /// Exit code 1 for invalid machines and failed checks, 2 for usage and I/O errors.
    pub(crate) fn exit_code(&self) -> ExitCode {
        match self {
            Self::Invalid { .. } | Self::Failed(_) => ExitCode::from(1),
            Self::File { .. } | Self::Io(_) | Self::Usage(_) => ExitCode::from(2),
        }
    }

    /// Whether the error is a closed pipe on standard output, as with `tm ... | head`.
    pub(crate) fn is_broken_pipe(&self) -> bool {
        matches!(self, Self::Io(e) if e.kind() == io::ErrorKind::BrokenPipe)
    }
}

/// The result type of every command.
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
