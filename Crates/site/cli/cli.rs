//! The command line every site binary gets: `build` and `serve`.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a command-line tool reports its result and its usage to the terminal"
)]

use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::{Site, index, serve, write};

/// The preview server's default port.
const DEFAULT_PORT: u16 = 8080;

/// What a run was asked to do.
#[derive(Debug, PartialEq, Eq)]
struct Command {
    /// Serve the result after writing it.
    serve: bool,
    /// The output folder, when not the default.
    out: Option<PathBuf>,
    /// The preview server's port.
    port: u16,
}

/// The usage text for `program`.
fn usage(program: &str) -> String {
    format!(
        "usage: {program} [build] [--out <dir>]\n       {program} serve [--out <dir>] [--port <port>]"
    )
}

/// Parses `[build|serve] [--out <dir>] [--port <port>]`. `--port` only applies to `serve`.
fn parse(args: impl Iterator<Item = String>) -> Option<Command> {
    let mut args = args.peekable();
    let serve = match args.peek().map(String::as_str) {
        Some("serve") => true,
        Some("build") => false,
        _ => return parse_options(args, false),
    };
    let _subcommand = args.next();
    parse_options(args, serve)
}

/// Parses the options after the subcommand.
fn parse_options(mut args: impl Iterator<Item = String>, serve: bool) -> Option<Command> {
    let mut command = Command {
        serve,
        out: None,
        port: DEFAULT_PORT,
    };
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--out", Some(dir)) => command.out = Some(PathBuf::from(dir)),
            ("--port", Some(port)) if serve => command.port = port.parse().ok()?,
            _ => return None,
        }
    }
    Some(command)
}

/// Runs a site's command line.
///
/// Builds the [`Site`] with `build`, writes it to `--out` (default `default_out`), indexes it with
/// Pagefind when the site asks for search, and with `serve` previews it under the site's base
/// path. Returns the process exit code: 0 on success, 1 on failure, 2 on bad usage.
pub fn run(
    program: &str,
    default_out: &Path,
    build: impl FnOnce() -> io::Result<Site>,
) -> ExitCode {
    let Some(command) = parse(std::env::args().skip(1)) else {
        eprintln!("{}", usage(program));
        return ExitCode::from(2);
    };
    let out = command.out.unwrap_or_else(|| default_out.to_path_buf());

    let site = match build() {
        Ok(site) => site,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let summary = match write(&site, &out) {
        Ok(summary) => summary,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "Wrote {} page(s) and {} file(s) in all to {}",
        summary.pages,
        summary.files,
        out.display()
    );

    if site.search() {
        if let Err(error) = index(&out) {
            eprintln!("error: indexing for search: {error}");
            return ExitCode::FAILURE;
        }
        println!("Indexed {} for search", out.display());
    }

    if command.serve
        && let Err(error) = serve(&out, site.base(), command.port)
    {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
