mod managed;
mod mcp;
use clap::{Parser, Subcommand};
use compass_core::{Error, Result, codec, compare, storage};
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Parser)]
#[command(
    name = "compass",
    version,
    about = "Observe and compare local repository states with explicit provenance"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Write one canonical JSON object to stdout"
    )]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(flatten)]
    Managed(managed::Managed),
    /// Serve configured snapshot sources via local MCP stdio (no network listener).
    Mcp {
        #[arg(long)]
        config: PathBuf,
    },
    /// Create a persistent identity and default observation policy.
    Init {
        #[arg(long)]
        root: PathBuf,
    },
    /// Observe eligible files and publish an immutable manifest.
    Snapshot {
        #[arg(long)]
        root: PathBuf,
    },
    /// Measure exact textual line-mass divergence using verified external packages.
    D0 {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        current: PathBuf,
        #[arg(long)]
        baseline_source: Option<PathBuf>,
        #[arg(long)]
        current_source: Option<PathBuf>,
        #[arg(long)]
        limits: Option<PathBuf>,
        #[arg(long, value_parser=clap::value_parser!(u64).range(1..=3_600_000))]
        deadline_ms: Option<u64>,
    },
    /// Search literal UTF-8 lines in a verified declared snapshot package.
    Search {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        source: PathBuf,
        #[arg(long, allow_hyphen_values = true)]
        query: String,
        #[arg(long, default_value = "20", value_parser=clap::value_parser!(u64).range(1..=1000))]
        max_matches: u64,
        #[arg(long)]
        limits: Option<PathBuf>,
        #[arg(long, value_parser=clap::value_parser!(u64).range(1..=3_600_000))]
        deadline_ms: Option<u64>,
    },
    /// Read exact lines from one fully verified object of a declared snapshot.
    Read {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        source: PathBuf,
        #[arg(long, allow_hyphen_values = true)]
        path_key: String,
        #[arg(long, default_value = "1", value_parser=clap::value_parser!(u64).range(1..))]
        start_line: u64,
        #[arg(long, default_value = "20", value_parser=clap::value_parser!(u64).range(1..=1000))]
        max_lines: u64,
        #[arg(long)]
        limits: Option<PathBuf>,
        #[arg(long, value_parser=clap::value_parser!(u64).range(1..=3_600_000))]
        deadline_ms: Option<u64>,
    },
    /// Compare two saved manifests without accessing their source tree.
    Compare {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        current: PathBuf,
    },
}

fn run(cli: &Cli, cancel: &AtomicBool) -> Result<(Value, i32)> {
    match &cli.command {
        Command::Managed(command) => managed::run(command, cancel),
        Command::D0 { .. }
        | Command::Search { .. }
        | Command::Read { .. }
        | Command::Mcp { .. } => {
            unreachable!("D0 has its own transport and error contract")
        }
        Command::Init { root } => Ok((
            serde_json::to_value(storage::init(root, cancel)?)
                .map_err(|e| Error::corrupt(e.to_string()))?,
            0,
        )),
        Command::Snapshot { root } => {
            let receipt = storage::snapshot(root, cancel)?;
            let code = if receipt.quality == "partial" { 3 } else { 0 };
            Ok((
                serde_json::to_value(receipt).map_err(|e| Error::corrupt(e.to_string()))?,
                code,
            ))
        }
        Command::Compare { baseline, current } => {
            let baseline = storage::load_snapshot(baseline)?;
            let current = storage::load_snapshot(current)?;
            let comparison = compare::compare(&baseline, &current)?;
            let code = if comparison.quality == "partial" {
                3
            } else {
                0
            };
            Ok((
                serde_json::to_value(comparison).map_err(|e| Error::corrupt(e.to_string()))?,
                code,
            ))
        }
    }
}

fn output(value: &Value, as_json: bool) -> Result<()> {
    let bytes = if as_json {
        codec::canonical(value)?
    } else {
        serde_json::to_vec_pretty(value).map_err(|e| Error::corrupt(e.to_string()))?
    };
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&bytes)
        .and_then(|_| stdout.write_all(b"\n"))
        .and_then(|_| stdout.flush())
        .map_err(|e| Error::io("write stdout", e))
}

fn fail(error: Error, as_json: bool) -> i32 {
    if as_json {
        if let Err(output_error) = output(
            &json!({"error":{"code":error.code,"message":error.message}}),
            true,
        ) {
            eprintln!("{output_error}");
        }
    } else {
        eprintln!("{}: {}", error.code, error.message);
    }
    error.exit_code
}

fn d0_fail(error: compass_core::d0::Error, as_json: bool) -> i32 {
    if as_json {
        if codec::canonical(&error.value()).ok().is_none_or(|bytes| {
            let mut out = io::stdout().lock();
            out.write_all(&bytes).and_then(|_| out.flush()).is_err()
        }) {
            eprintln!("operational_error: write stdout");
            return 1;
        }
    } else {
        eprintln!("{error}");
    }
    error.exit_code()
}
fn run_d0(cli: &Cli, cancel: &AtomicBool) -> i32 {
    use compass_core::d0;
    let Command::D0 {
        baseline,
        current,
        baseline_source,
        current_source,
        limits,
        deadline_ms,
    } = &cli.command
    else {
        unreachable!()
    };
    let result = d0::limits(limits.as_deref(), *deadline_ms).and_then(|limits| {
        d0::measure(
            &d0::Request {
                baseline: baseline.clone(),
                current: current.clone(),
                baseline_source: baseline_source.clone(),
                current_source: current_source.clone(),
                limits,
            },
            cancel,
        )
    });
    if cancel.load(Ordering::Relaxed) {
        return d0_fail(d0::Error("cancelled"), cli.json);
    }
    match result {
        Err(error) => d0_fail(error, cli.json),
        Ok(result) => {
            let bytes = if cli.json {
                result.bytes
            } else {
                let m = &result.value["measurement"];
                format!("D0 status: {}\nintegral: {}\nknown subset (resolved text units only): {}\nreasons: {}\n",m["status"],m["value"],m["known_subset"],m["reasons"]).into_bytes()
            };
            let mut out = io::stdout().lock();
            if out.write_all(&bytes).and_then(|_| out.flush()).is_err() {
                eprintln!("operational_error: write stdout");
                1
            } else {
                result.exit_code
            }
        }
    }
}

fn search_fail(error: compass_core::search::Error, as_json: bool) -> i32 {
    let value = error.value();
    let bytes = if as_json {
        codec::canonical(&value).ok()
    } else {
        serde_json::to_vec_pretty(&value).ok().map(|mut b| {
            b.push(b'\n');
            b
        })
    };
    let Some(bytes) = bytes else {
        return 1;
    };
    let mut out = io::stdout().lock();
    if out.write_all(&bytes).and_then(|_| out.flush()).is_err() {
        1
    } else {
        error.exit_code()
    }
}

fn run_search(cli: &Cli, cancel: &AtomicBool) -> i32 {
    use compass_core::search;
    let Command::Search {
        snapshot,
        source,
        query,
        max_matches,
        limits,
        deadline_ms,
    } = &cli.command
    else {
        unreachable!()
    };
    let result = search::limits(limits.as_deref(), *deadline_ms).and_then(|limits| {
        search::search(
            &search::Request {
                snapshot: snapshot.clone(),
                source: source.clone(),
                query: query.clone(),
                max_matches: *max_matches,
                limits,
            },
            cancel,
        )
    });
    let result = if cancel.load(Ordering::Relaxed) {
        Err(search::Error("cancelled"))
    } else {
        result
    };
    let (bytes, code) = match result {
        Ok(o) => (if cli.json { o.bytes } else { o.pretty_bytes }, o.exit_code),
        Err(e) => return search_fail(e, cli.json),
    };
    let mut out = io::stdout().lock();
    if out.write_all(&bytes).and_then(|_| out.flush()).is_err() {
        1
    } else {
        code
    }
}

fn run_read(cli: &Cli, cancel: &AtomicBool) -> i32 {
    use compass_core::read;
    let Command::Read {
        snapshot,
        source,
        path_key,
        start_line,
        max_lines,
        limits,
        deadline_ms,
    } = &cli.command
    else {
        unreachable!()
    };
    let result = read::limits(limits.as_deref(), *deadline_ms).and_then(|limits| {
        read::read(
            &read::Request {
                snapshot: snapshot.clone(),
                source: source.clone(),
                path_key: path_key.clone(),
                start_line: *start_line,
                max_lines: *max_lines,
                limits,
            },
            cancel,
        )
    });
    if cancel.load(Ordering::Relaxed) {
        return search_fail(read::Error("cancelled"), cli.json);
    }
    match result {
        Err(e) => search_fail(e, cli.json),
        Ok(o) => {
            let bytes = if cli.json { o.bytes } else { o.pretty_bytes };
            let mut out = io::stdout().lock();
            if out.write_all(&bytes).and_then(|_| out.flush()).is_err() {
                1
            } else {
                o.exit_code
            }
        }
    }
}

// After a parse error only the output mode is needed, not a second parser.
// Query is the sole search option accepting a separate hyphenated value; that
// next token belongs to the query even when it spells --json. Other options
// leave a following --json as an option, including when their value is missing.
fn search_error_json_requested(value_option: &str) -> bool {
    let mut args = std::env::args_os().skip(1);
    let mut requested = false;
    while let Some(arg) = args.next() {
        if arg == value_option {
            args.next();
        } else if arg == "--" {
            break;
        } else if arg == "--json" {
            requested = true;
        }
    }
    requested
}

fn main() -> std::process::ExitCode {
    let wants_json = std::env::args_os().any(|a| a == "--json");
    let subcommand = std::env::args_os().skip(1).find(|a| a != "--json");
    let wants_read = subcommand.as_ref().is_some_and(|a| a == "read");
    let wants_rd = wants_read || subcommand.as_ref().is_some_and(|a| a == "search");
    let wants_d0 = std::env::args_os()
        .skip(1)
        .find(|a| {
            ["init", "snapshot", "compare", "d0"]
                .iter()
                .any(|name| a == *name)
        })
        .is_some_and(|a| a == "d0");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) if e.exit_code() == 0 => {
            let _ = e.print();
            return std::process::ExitCode::SUCCESS;
        }
        Err(e) => {
            if wants_rd {
                return std::process::ExitCode::from(search_fail(
                    compass_core::search::Error("invalid_arguments"),
                    search_error_json_requested(if wants_read { "--path-key" } else { "--query" }),
                ) as u8);
            }
            if wants_d0 {
                return std::process::ExitCode::from(d0_fail(
                    compass_core::d0::Error("invalid_arguments"),
                    wants_json,
                ) as u8);
            }
            return std::process::ExitCode::from(
                fail(Error::invalid(e.to_string()), wants_json) as u8
            );
        }
    };
    let cancel = Arc::new(AtomicBool::new(false));
    if let Err(e) = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&cancel)) {
        if wants_rd {
            return std::process::ExitCode::from(search_fail(
                compass_core::search::Error("operational_error"),
                cli.json,
            ) as u8);
        }
        if wants_d0 {
            return std::process::ExitCode::from(d0_fail(
                compass_core::d0::Error("operational_error"),
                cli.json,
            ) as u8);
        }
        return std::process::ExitCode::from(fail(
            Error::io("register interrupt handler", e),
            cli.json,
        ) as u8);
    }
    if let Command::Mcp { config } = &cli.command {
        return std::process::ExitCode::from(mcp::serve(config, cancel) as u8);
    }
    if matches!(cli.command, Command::Read { .. }) {
        return std::process::ExitCode::from(run_read(&cli, &cancel) as u8);
    }
    if matches!(cli.command, Command::Search { .. }) {
        return std::process::ExitCode::from(run_search(&cli, &cancel) as u8);
    }
    if matches!(cli.command, Command::D0 { .. }) {
        return std::process::ExitCode::from(run_d0(&cli, &cancel) as u8);
    }
    let result = run(&cli, &cancel);
    if cancel.load(Ordering::Relaxed) {
        return std::process::ExitCode::from(fail(Error::cancelled(), cli.json) as u8);
    }
    let status = match result {
        Ok((value, code)) => match output(&value, cli.json) {
            Ok(()) => code,
            Err(e) => fail(e, false),
        },
        Err(error) => fail(error, cli.json),
    };
    std::process::ExitCode::from(status as u8)
}
