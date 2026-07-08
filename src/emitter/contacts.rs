// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-apple-contacts-emitter requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use std::{
    error::Error as StdError,
    fmt, io,
    process::{Command, ExitStatus},
};

type CoreResult<T> = Result<T, ContactsError>;

#[derive(Debug)]
enum ContactsError {
    Io {
        context: &'static str,
        source: io::Error,
    },
    OsaScriptFailed {
        status: ExitStatus,
        stderr: String,
    },
    Jq {
        context: &'static str,
        source: jq::JsonFilterError,
    },
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
}

impl fmt::Display for ContactsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactsError::Io { context, .. } => {
                write!(f, "I/O error while {context}")
            }
            ContactsError::OsaScriptFailed { .. } => {
                write!(f, "failed to talk to Apple Contacts (osascript)")
            }
            ContactsError::Jq { context, .. } => {
                write!(f, "failed to filter JSON while {context}")
            }
            ContactsError::Json { context, .. } => {
                write!(f, "failed to serialize JSON while {context}")
            }
        }
    }
}

impl StdError for ContactsError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ContactsError::Io { source, .. } => Some(source),
            ContactsError::Jq { source, .. } => Some(source),
            ContactsError::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for ContactsError {
    fn from(source: io::Error) -> Self {
        ContactsError::Io {
            context: "performing I/O",
            source,
        }
    }
}

fn handle_error(err: &ContactsError, _flags: &StandardOptions) -> SysexitsError {
    eprintln!("Error: {err}");

    if let ContactsError::OsaScriptFailed { stderr, .. } = err {
        if !stderr.trim().is_empty() {
            eprintln!("{}", stderr.trim());
        }
    }

    #[cfg(feature = "tracing")]
    match err {
        ContactsError::Io { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::contacts_emitter",
                %context,
                error = %source,
                "I/O error details"
            );
        }
        ContactsError::OsaScriptFailed { status, stderr } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::contacts_emitter",
                ?status,
                stderr = %stderr,
                "osascript failure details"
            );
        }
        ContactsError::Jq { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::contacts_emitter",
                %context,
                error = %source,
                "jq filter failure details"
            );
        }
        ContactsError::Json { context, source } => {
            asimov_module::tracing::debug!(
                target: "asimov_apple_module::contacts_emitter",
                %context,
                error = %source,
                "JSON serialization failure details"
            );
        }
    }

    match err {
        ContactsError::Io { .. } => EX_IOERR,
        ContactsError::OsaScriptFailed { .. } => EX_UNAVAILABLE,
        ContactsError::Jq { .. } => EX_DATAERR,
        ContactsError::Json { .. } => EX_DATAERR,
    }
}

/// asimov-apple-contacts-emitter
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

    const JAVASCRIPT: &str = r#"
        ObjC.import("Contacts");
        ObjC.import("Foundation");

        function text(value) {
            const unwrapped = ObjC.unwrap(value);
            if (unwrapped === null || unwrapped === undefined) {
                return "";
            }
            return String(unwrapped);
        }

        function safeText(getter) {
            try {
                return text(getter());
            } catch (error) {
                return "";
            }
        }

        function contactText(contact, key, getter) {
            if (!contact.isKeyAvailable(key)) {
                return "";
            }
            return safeText(getter);
        }

        function contactArray(contact, key, getter) {
            if (!contact.isKeyAvailable(key)) {
                return [];
            }
            try {
                return getter();
            } catch (error) {
                return [];
            }
        }

        function cleanLabel(value) {
            return text(value).replace(/^\_\$!<(.+)>!\$_$/, "$1").toLowerCase();
        }

        function arrayValues(values, mapper) {
            const output = [];
            const count = Number(values.count);
            for (let i = 0; i < count; i += 1) {
                const mapped = mapper(values.objectAtIndex(i));
                if (mapped !== null) {
                    output.push(mapped);
                }
            }
            return output;
        }

        function contactPoint(item, value) {
            const label = cleanLabel(item.label);
            if (!value) {
                return null;
            }
            const point = {
                "@type": "ContactPoint",
                "value": value,
            };
            if (label) {
                point.name = label;
            }
            return point;
        }

        const store = $.CNContactStore.alloc.init;
        let accessDone = false;
        let accessGranted = false;
        let accessError = "";

        store.requestAccessForEntityTypeCompletionHandler(
            $.CNEntityTypeContacts,
            (granted, error) => {
                accessGranted = granted;
                if (error) {
                    accessError = text(error.localizedDescription);
                }
                accessDone = true;
            }
        );

        while (!accessDone) {
            delay(0.1);
        }

        if (!accessGranted) {
            throw new Error(accessError || "Access denied to Apple Contacts");
        }

        const keys = $.NSMutableArray.array;
        keys.addObject($.CNContactIdentifierKey);
        keys.addObject($.CNContactGivenNameKey);
        keys.addObject($.CNContactMiddleNameKey);
        keys.addObject($.CNContactFamilyNameKey);
        keys.addObject($.CNContactOrganizationNameKey);
        keys.addObject($.CNContactJobTitleKey);
        keys.addObject($.CNContactEmailAddressesKey);
        keys.addObject($.CNContactPhoneNumbersKey);
        keys.addObject($.CNContactPostalAddressesKey);

        const request = $.CNContactFetchRequest.alloc.initWithKeysToFetch(keys);
        const lines = [];

        const ok = store.enumerateContactsWithFetchRequestErrorUsingBlock(
            request,
            null,
            (contact, stop) => {
                const givenName = contactText(contact, $.CNContactGivenNameKey, () => contact.givenName);
                const additionalName = contactText(contact, $.CNContactMiddleNameKey, () => contact.middleName);
                const familyName = contactText(contact, $.CNContactFamilyNameKey, () => contact.familyName);
                const organization = contactText(contact, $.CNContactOrganizationNameKey, () => contact.organizationName);
                const jobTitle = contactText(contact, $.CNContactJobTitleKey, () => contact.jobTitle);
                const name = [givenName, additionalName, familyName].filter(Boolean).join(" ") || organization;

                const node = {
                    "@type": "Person",
                    "@id": "urn:apple:contacts:person:" + contactText(contact, $.CNContactIdentifierKey, () => contact.identifier),
                    "name": name,
                    "source": "apple-contacts",
                };

                if (givenName) node.givenName = givenName;
                if (additionalName) node.additionalName = additionalName;
                if (familyName) node.familyName = familyName;
                if (organization) node.affiliation = organization;
                if (jobTitle) node.jobTitle = jobTitle;

                const emails = contactArray(contact, $.CNContactEmailAddressesKey, () => {
                    return arrayValues(contact.emailAddresses, (item) => {
                        return contactPoint(item, safeText(() => item.value));
                    });
                });
                if (emails.length > 0) node.email = emails;

                const phones = contactArray(contact, $.CNContactPhoneNumbersKey, () => {
                    return arrayValues(contact.phoneNumbers, (item) => {
                        return contactPoint(item, safeText(() => item.value.stringValue));
                    });
                });
                if (phones.length > 0) node.telephone = phones;

                const addresses = contactArray(contact, $.CNContactPostalAddressesKey, () => {
                    return arrayValues(contact.postalAddresses, (item) => {
                        const value = item.value;
                        const address = {"@type": "PostalAddress"};
                        const label = cleanLabel(item.label);
                        const street = safeText(() => value.street);
                        const city = safeText(() => value.city);
                        const region = safeText(() => value.state);
                        const postalCode = safeText(() => value.postalCode);
                        const country = safeText(() => value.country);

                        if (label) address.name = label;
                        if (street) address.streetAddress = street;
                        if (city) address.addressLocality = city;
                        if (region) address.addressRegion = region;
                        if (postalCode) address.postalCode = postalCode;
                        if (country) address.addressCountry = country;

                        return Object.keys(address).length > 1 ? address : null;
                    });
                });
                if (addresses.length > 0) node.address = addresses;

                lines.push(JSON.stringify(node));
            }
        );

        if (!ok) {
            throw new Error("Failed to enumerate Apple Contacts");
        }

        lines.join("\n");
    "#;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::contacts_emitter",
        "starting apple contacts emitter"
    );

    let output = Command::new("osascript")
        .arg("-l")
        .arg("JavaScript")
        .arg("-e")
        .arg(JAVASCRIPT)
        .output()
        .map_err(|e| ContactsError::Io {
            context: "invoking osascript",
            source: e,
        })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::debug!(
        target: "asimov_apple_module::contacts_emitter",
        status = ?output.status,
        stdout_len = output.stdout.len(),
        stderr_len = output.stderr.len(),
        "osascript completed"
    );

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ContactsError::OsaScriptFailed {
            status: output.status,
            stderr,
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.trim().is_empty() {
        #[cfg(feature = "tracing")]
        asimov_module::tracing::info!(
            target: "asimov_apple_module::contacts_emitter",
            "no people returned from Apple Contacts"
        );
        return Ok(());
    }

    let locked = io::stdout().lock();
    let mut writer = BufWriter::new(locked);
    let mut count = 0usize;

    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let node = asimov_apple_module::contacts()
            .filter_json_str(line)
            .map_err(|e| ContactsError::Jq {
                context: "filtering contact JSON",
                source: e,
            })?;

        serde_json::to_writer(&mut writer, &node).map_err(|e| ContactsError::Json {
            context: "writing filtered contact JSON",
            source: e,
        })?;
        writer.write_all(b"\n").map_err(|e| ContactsError::Io {
            context: "writing newline to stdout",
            source: e,
        })?;

        count += 1;
    }

    writer.flush().map_err(|e| ContactsError::Io {
        context: "flushing stdout",
        source: e,
    })?;

    #[cfg(feature = "tracing")]
    asimov_module::tracing::info!(
        target: "asimov_apple_module::contacts_emitter",
        contacts = count,
        "finished apple contacts emitter"
    );

    Ok(())
}
