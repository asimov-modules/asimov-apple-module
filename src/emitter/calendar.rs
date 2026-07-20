// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-apple-calendar-emitter requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use std::{
    error::Error as StdError,
    fmt, io,
    process::{Command, ExitStatus},
};

type CoreResult<T> = Result<T, CalendarError>;

#[derive(Debug)]
enum CalendarError {
    Io {
        context: &'static str,
        source: io::Error,
    },
    OsaScriptFailed {
        status: ExitStatus,
        stderr: String,
    },
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
    Jq {
        context: &'static str,
        source: jq::JsonFilterError,
    },
}

impl fmt::Display for CalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalendarError::Io { context, .. } => {
                write!(f, "I/O error while {context}")
            }
            CalendarError::OsaScriptFailed { .. } => {
                write!(f, "failed to talk to Apple Calendar (osascript)")
            }
            CalendarError::Json { context, .. } => {
                write!(f, "failed to serialize JSON while {context}")
            }
            CalendarError::Jq { context, .. } => {
                write!(f, "failed to filter JSON while {context}")
            }
        }
    }
}

impl StdError for CalendarError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CalendarError::Io { source, .. } => Some(source),
            CalendarError::Json { source, .. } => Some(source),
            CalendarError::Jq { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for CalendarError {
    fn from(source: io::Error) -> Self {
        CalendarError::Io {
            context: "performing I/O",
            source,
        }
    }
}

impl From<serde_json::Error> for CalendarError {
    fn from(e: serde_json::Error) -> Self {
        CalendarError::Json {
            context: "writing JSON to stdout",
            source: e,
        }
    }
}

fn handle_error(err: &CalendarError, _flags: &StandardOptions) -> SysexitsError {
    eprintln!("Error: {err}");

    #[cfg(feature = "tracing")]
    match err {
        CalendarError::Io { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::calendar_emitter",
                %context,
                error = %source,
                "I/O error details"
            );
        }
        CalendarError::OsaScriptFailed { status, stderr } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::calendar_emitter",
                ?status,
                stderr = %stderr,
                "osascript failure details"
            );
        }
        CalendarError::Json { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::calendar_emitter",
                %context,
                error = %source,
                "JSON serialization failure details"
            );
        }
        CalendarError::Jq { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::calendar_emitter",
                %context,
                error = %source,
                "jq filter failure details"
            );
        }
    }

    match err {
        CalendarError::Io { .. } => EX_IOERR,
        CalendarError::OsaScriptFailed { .. } => EX_UNAVAILABLE,
        CalendarError::Json { .. } => EX_DATAERR,
        CalendarError::Jq { .. } => EX_DATAERR,
    }
}

/// asimov-apple-calendar-emitter
#[derive(Debug, Parser)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,
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

fn run_emitter(_opts: &Options) -> CoreResult<()> {
    use std::io::{self, BufWriter, Write};

    const APPLESCRIPT: &str = r#"
        on replaceText(theText, searchString, replacementString)
            set oldDelimiters to AppleScript's text item delimiters
            set AppleScript's text item delimiters to searchString
            set textItems to text items of theText
            set AppleScript's text item delimiters to replacementString
            set replacedText to textItems as text
            set AppleScript's text item delimiters to oldDelimiters
            return replacedText
        end replaceText

        on jsonEscape(value)
            if value is missing value then
                return ""
            end if
            set escaped to value as text
            set escaped to my replaceText(escaped, "\\", "\\\\")
            set escaped to my replaceText(escaped, quote, "\\" & quote)
            set escaped to my replaceText(escaped, return, "\\r")
            set escaped to my replaceText(escaped, linefeed, "\\n")
            set escaped to my replaceText(escaped, tab, "\\t")
            return escaped
        end jsonEscape

        on jsonPair(keyName, keyValue)
            return quote & keyName & quote & ":" & quote & my jsonEscape(keyValue) & quote
        end jsonPair

        on joinList(valuesList, delimiter)
            set oldDelimiters to AppleScript's text item delimiters
            set AppleScript's text item delimiters to delimiter
            set joinedText to valuesList as text
            set AppleScript's text item delimiters to oldDelimiters
            return joinedText
        end joinList

        set output to ""
        tell application "Calendar"
            set theCalendars to every calendar
            repeat with cal in theCalendars
                set calName to the name of cal
                set eventsList to every event of cal
                repeat with e in eventsList
                    set eventId to the uid of e
                    set eventTitle to the summary of e
                    set eventStart to the start date of e
                    set eventEnd to the end date of e
                    set eventLoc to the location of e
                    set eventDesc to the description of e

                    set jsonFields to {my jsonPair("@type", "Event")}
                    set end of jsonFields to my jsonPair("@id", "urn:apple:calendar:event:" & eventId)
                    set end of jsonFields to my jsonPair("name", eventTitle)
                    set end of jsonFields to my jsonPair("startDate", eventStart as string)
                    set end of jsonFields to my jsonPair("endDate", eventEnd as string)
                    set end of jsonFields to my jsonPair("isPartOf", calName)
                    set end of jsonFields to my jsonPair("source", "apple-calendar")

                    if eventLoc is not missing value and eventLoc is not "" then
                        set end of jsonFields to my jsonPair("location", eventLoc)
                    end if

                    if eventDesc is not missing value and eventDesc is not "" then
                        set end of jsonFields to my jsonPair("description", eventDesc)
                    end if

                    set output to output & "{" & my joinList(jsonFields, ",") & "}" & linefeed
                end repeat
            end repeat
        end tell
        return output
    "#;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::calendar_emitter",
        "starting apple calendar emitter"
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(APPLESCRIPT)
        .output()
        .map_err(|e| CalendarError::Io {
            context: "invoking osascript",
            source: e,
        })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::debug!(
        target: "asimov_apple_module::calendar_emitter",
        status = ?output.status,
        stdout_len = output.stdout.len(),
        stderr_len = output.stderr.len(),
        "osascript completed"
    );

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CalendarError::OsaScriptFailed {
            status: output.status,
            stderr,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        #[cfg(feature = "tracing")]
        asimov_module::tracing::info!(
            target: "asimov_apple_module::calendar_emitter",
            "no events returned from Apple Calendar"
        );
        return Ok(());
    }

    let locked = io::stdout().lock();
    let mut writer = BufWriter::new(locked);

    let mut count = 0usize;

    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let node = asimov_apple_module::calendar()
            .filter_json_str(line)
            .map_err(|e| CalendarError::Jq {
                context: "filtering calendar event JSON",
                source: e,
            })?;

        #[cfg(feature = "tracing")]
        asimov_module::tracing::debug!(
            target: "asimov_apple_module::calendar_emitter",
            "emitting event"
        );

        serde_json::to_writer(&mut writer, &node)?;
        writer.write_all(b"\n").map_err(|e| CalendarError::Io {
            context: "writing newline to stdout",
            source: e,
        })?;

        count += 1;
    }

    writer.flush().map_err(|e| CalendarError::Io {
        context: "flushing stdout",
        source: e,
    })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::calendar_emitter",
        events = count,
        "finished apple calendar emitter"
    );

    Ok(())
}
