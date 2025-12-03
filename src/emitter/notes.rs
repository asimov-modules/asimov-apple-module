// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-apple-notes-emitter requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use html2text::from_read;
use serde_json::json;
use std::{
    error::Error as StdError,
    fmt, io,
    process::{Command, ExitStatus},
};

type CoreResult<T> = Result<T, NotesError>;

#[derive(Debug)]
enum NotesError {
    Io {
        context: &'static str,
        source: io::Error,
    },
    OsaScriptFailed {
        status: ExitStatus,
        stderr: String,
    },
    NotesParse {
        context: &'static str,
        message: String,
    },
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
}

impl fmt::Display for NotesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotesError::Io { context, .. } => {
                write!(f, "I/O error while {context}")
            }
            NotesError::OsaScriptFailed { .. } => {
                write!(f, "failed to talk to Apple Notes (osascript)")
            }
            NotesError::NotesParse { context, .. } => {
                write!(f, "failed to parse Apple Notes output while {context}")
            }
            NotesError::Json { context, .. } => {
                write!(f, "failed to serialize JSON while {context}")
            }
        }
    }
}

impl StdError for NotesError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            NotesError::Io { source, .. } => Some(source),
            NotesError::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for NotesError {
    fn from(source: io::Error) -> Self {
        NotesError::Io {
            context: "performing I/O",
            source,
        }
    }
}

impl From<html2text::Error> for NotesError {
    fn from(e: html2text::Error) -> Self {
        NotesError::NotesParse {
            context: "converting note body from HTML to text",
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for NotesError {
    fn from(e: serde_json::Error) -> Self {
        NotesError::Json {
            context: "writing JSON to stdout",
            source: e,
        }
    }
}

fn handle_error(err: &NotesError, _flags: &StandardOptions) -> SysexitsError {
    eprintln!("Error: {err}");

    #[cfg(feature = "tracing")]
    match err {
        NotesError::Io { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::notes_emitter",
                %context,
                error = %source,
                "I/O error details"
            );
        }
        NotesError::OsaScriptFailed { status, stderr } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::notes_emitter",
                ?status,
                stderr = %stderr,
                "osascript failure details"
            );
        }
        NotesError::NotesParse { context, message } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::notes_emitter",
                %context,
                %message,
                "parse failure details"
            );
        }
        NotesError::Json { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::notes_emitter",
                %context,
                error = %source,
                "JSON serialization failure details"
            );
        }
    }

    match err {
        NotesError::Io { .. } => EX_IOERR,
        NotesError::OsaScriptFailed { .. } => EX_UNAVAILABLE,
        NotesError::NotesParse { .. } => EX_DATAERR,
        NotesError::Json { .. } => EX_DATAERR,
    }
}

/// asimov-apple-notes-emitter
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// Wrap width for plain-text conversion from HTML
    #[arg(
        short = 'w',
        long = "wrap-width",
        value_name = "WIDTH",
        default_value = "80"
    )]
    wrap_width: usize,
}

pub fn main() -> Result<SysexitsError, Box<dyn StdError>> {
    // Load environment variables from `.env`:
    asimov_module::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = asimov_module::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    // Handle the `--version` flag:
    if options.flags.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(EX_OK);
    }

    // Handle the `--license` flag:
    if options.flags.license {
        print!("{}", include_str!("../../UNLICENSE"));
        return Ok(EX_OK);
    }

    // Configure logging & tracing:
    #[cfg(feature = "tracing")]
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    let exit_code = match run_emitter(&options) {
        Ok(()) => EX_OK,
        Err(err) => handle_error(&err, &options.flags),
    };

    Ok(exit_code)
}

fn run_emitter(opts: &Options) -> CoreResult<()> {
    use std::io::{self, BufWriter, Write};

    const APPLESCRIPT: &str = r#"
        set output to ""
        tell application "Notes"
            set theAccounts to every account
            repeat with acc in theAccounts
                set accName to the name of acc
                set foldersList to every folder of acc
                repeat with f in foldersList
                    set folderName to the name of f
                    set notesList to every note of f
                    repeat with n in notesList
                        set noteId to the id of n
                        set noteName to the name of n
                        set noteBody to the body of n
                        set noteCreated to the creation date of n
                        set noteModified to the modification date of n
                        set output to output & noteId & "|||"
                        set output to output & noteName & "|||"
                        set output to output & noteBody & "|||"
                        set output to output & noteCreated & "|||"
                        set output to output & noteModified & "|||"
                        set output to output & folderName & "|||"
                        set output to output & accName & "~~~"
                    end repeat
                end repeat
            end repeat
        end tell
        return output
    "#;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::notes_emitter",
        "starting apple notes emitter"
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(APPLESCRIPT)
        .output()
        .map_err(|e| NotesError::Io {
            context: "invoking osascript",
            source: e,
        })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::debug!(
        target: "asimov_apple_module::notes_emitter",
        status = ?output.status,
        stdout_len = output.stdout.len(),
        stderr_len = output.stderr.len(),
        "osascript completed"
    );

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(NotesError::OsaScriptFailed {
            status: output.status,
            stderr,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        #[cfg(feature = "tracing")]
        asimov_module::tracing::info!(
            target: "asimov_apple_module::notes_emitter",
            "no notes returned from Apple Notes"
        );
        return Ok(());
    }

    let locked = io::stdout().lock();
    let mut writer = BufWriter::new(locked);

    let mut count = 0usize;

    for chunk in stdout.split("~~~").filter(|c| !c.trim().is_empty()) {
        let mut parts = chunk.split("|||");

        let id = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading note id",
                message: "missing id field".to_string(),
            })?
            .trim();

        let name = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading note name",
                message: "missing name field".to_string(),
            })?
            .trim()
            .to_string();

        let body_html = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading note body",
                message: "missing body field".to_string(),
            })?
            .trim();

        let created = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading creation date",
                message: "missing creation date field".to_string(),
            })?
            .trim()
            .to_string();

        let modified = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading modification date",
                message: "missing modification date field".to_string(),
            })?
            .trim()
            .to_string();

        let folder = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading folder name",
                message: "missing folder field".to_string(),
            })?
            .trim()
            .to_string();

        let account = parts
            .next()
            .ok_or_else(|| NotesError::NotesParse {
                context: "reading account name",
                message: "missing account field".to_string(),
            })?
            .trim()
            .to_string();

        let text = from_read(body_html.as_bytes(), opts.wrap_width)?
            .trim()
            .to_string();

        #[cfg(feature = "tracing")]
        asimov_module::tracing::debug!(
            target: "asimov_apple_module::notes_emitter",
            note_id = %id,
            account = %account,
            folder = %folder,
            name = %name,
            "emitting note"
        );

        let node = json!({
            "@type": "CreativeWork",
            "@id": format!("urn:apple:notes:note:{id}"),
            "name": name,
            "text": text,
            "dateCreated": created,
            "dateModified": modified,
            "isPartOf": folder,
            "account": account,
            "source": "apple-notes",
        });

        serde_json::to_writer(&mut writer, &node)?;
        writer.write_all(b"\n").map_err(|e| NotesError::Io {
            context: "writing newline to stdout",
            source: e,
        })?;

        count += 1;
    }

    writer.flush().map_err(|e| NotesError::Io {
        context: "flushing stdout",
        source: e,
    })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::notes_emitter",
        notes = count,
        "finished apple notes emitter"
    );

    Ok(())
}
