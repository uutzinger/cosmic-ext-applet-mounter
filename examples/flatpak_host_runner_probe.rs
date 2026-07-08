// SPDX-License-Identifier: MIT

use std::time::Duration;

use cosmic_ext_applet_mounter::diagnostics::DependencyInventory;
use cosmic_ext_applet_mounter::process::{
    CommandError, CommandOutput, CommandRequest, CommandRunner, Executable,
    FlatpakHostCommandRunner, RuntimeCommandRunner, SystemCommandRunner,
};
use tokio_util::sync::CancellationToken;

struct Probe {
    label: &'static str,
    request: CommandRequest,
}

struct ErrorProbe {
    label: &'static str,
    request: CommandRequest,
    expected: ExpectedError,
}

enum ExpectedError {
    NonZero,
    NonZeroWithStderr(&'static str),
    Timeout,
    Cancelled,
}

#[tokio::main]
async fn main() {
    let mode = match std::env::args().nth(1).as_deref() {
        None | Some("--flatpak-host") => ProbeMode::FlatpakHost,
        Some("--native") => ProbeMode::Native,
        Some("--help") | Some("-h") => {
            print_help();
            return;
        }
        Some(other) => {
            eprintln!("unknown argument `{other}`");
            print_help();
            std::process::exit(2);
        }
    };

    let failed = run(mode).await;
    if failed {
        std::process::exit(1);
    }
}

#[derive(Clone, Copy)]
enum ProbeMode {
    Native,
    FlatpakHost,
}

async fn run(mode: ProbeMode) -> bool {
    let probes = match probes() {
        Ok(probes) => probes,
        Err(error) => {
            eprintln!("failed to build probes: {error}");
            return true;
        }
    };
    let cancellation = CancellationToken::new();
    let mut failed = false;

    println!("COSMIC Mounter host-runner probe");
    println!(
        "mode: {}",
        match mode {
            ProbeMode::Native => "native",
            ProbeMode::FlatpakHost => "flatpak-spawn --host",
        }
    );
    match mode {
        ProbeMode::Native => run_inventory(&SystemCommandRunner, &cancellation).await,
        ProbeMode::FlatpakHost => run_inventory(&FlatpakHostCommandRunner, &cancellation).await,
    }

    for probe in probes {
        let result = match mode {
            ProbeMode::Native => run_probe(&SystemCommandRunner, &probe, &cancellation).await,
            ProbeMode::FlatpakHost => {
                run_probe(&FlatpakHostCommandRunner, &probe, &cancellation).await
            }
        };
        if let Err(error) = result {
            failed = true;
            println!("FAIL {}: {error}", probe.label);
        }
    }
    let error_probes = match error_probes() {
        Ok(probes) => probes,
        Err(error) => {
            eprintln!("failed to build error probes: {error}");
            return true;
        }
    };
    for probe in error_probes {
        let result = match mode {
            ProbeMode::Native => run_error_probe(&SystemCommandRunner, &probe, &cancellation).await,
            ProbeMode::FlatpakHost => {
                run_error_probe(&FlatpakHostCommandRunner, &probe, &cancellation).await
            }
        };
        if let Err(error) = result {
            failed = true;
            println!("FAIL {}: {error}", probe.label);
        }
    }

    if failed {
        println!("result: failed");
    } else {
        println!("result: passed");
    }
    failed
}

async fn run_inventory(runner: &dyn CommandRunner, cancellation: &CancellationToken) {
    println!("dependency inventory:");
    let inventory = DependencyInventory::inspect(runner, cancellation.child_token()).await;
    for report in inventory.reports {
        println!(
            "  {:?}: {:?}; version={}; path={}; {}",
            report.kind,
            report.state,
            report
                .version
                .map_or_else(|| "unknown".into(), |version| version.to_string()),
            report.path.as_deref().unwrap_or("not found"),
            report.summary
        );
    }
}

async fn run_error_probe(
    runner: &dyn CommandRunner,
    probe: &ErrorProbe,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let token = cancellation.child_token();
    if matches!(probe.expected, ExpectedError::Cancelled) {
        let trigger = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            trigger.cancel();
        });
    }

    match runner.run(probe.request.clone(), token).await {
        Ok(output) => Err(format!(
            "expected error but command succeeded: {}",
            output.command
        )),
        Err(error) if expected_error_matches(&probe.expected, &error) => {
            println!("OK {}: expected {}", probe.label, error_kind(&error));
            if let CommandError::NonZero { stderr, .. } = &error
                && !stderr.text.trim().is_empty()
            {
                println!("  stderr: {}", first_line(&stderr.text));
            }
            Ok(())
        }
        Err(error) => Err(format!(
            "unexpected error {}; expected {}",
            error_kind(&error),
            expected_error_name(&probe.expected)
        )),
    }
}

async fn run_probe(
    runner: &dyn CommandRunner,
    probe: &Probe,
    cancellation: &CancellationToken,
) -> Result<(), CommandError> {
    match runner.resolve(probe.request.executable) {
        Some(path) => println!("FOUND {}: {}", probe.label, path.display()),
        None => println!("MISSING {} during resolve; attempting run", probe.label),
    }

    let output = runner
        .run(probe.request.clone(), cancellation.child_token())
        .await?;
    print_output(probe.label, &output);
    Ok(())
}

fn error_probes() -> Result<Vec<ErrorProbe>, CommandError> {
    Ok(vec![
        ErrorProbe {
            label: "nonzero exit",
            request: CommandRequest::new(Executable::False).with_timeout(Duration::from_secs(5)),
            expected: ExpectedError::NonZero,
        },
        ErrorProbe {
            label: "stderr capture",
            request: CommandRequest::new(Executable::Cat)
                .arg("/nonexistent/cosmic-mounter-flatpak-probe")?
                .with_timeout(Duration::from_secs(5)),
            expected: ExpectedError::NonZeroWithStderr("No such file"),
        },
        ErrorProbe {
            label: "timeout",
            request: CommandRequest::new(Executable::Sleep)
                .arg("5")?
                .with_timeout(Duration::from_millis(100)),
            expected: ExpectedError::Timeout,
        },
        ErrorProbe {
            label: "cancellation",
            request: CommandRequest::new(Executable::Sleep)
                .arg("5")?
                .with_timeout(Duration::from_secs(5)),
            expected: ExpectedError::Cancelled,
        },
    ])
}

fn probes() -> Result<Vec<Probe>, CommandError> {
    Ok(vec![
        Probe {
            label: "rclone version",
            request: CommandRequest::new(Executable::Rclone)
                .arg("version")?
                .with_timeout(Duration::from_secs(10)),
        },
        Probe {
            label: "nmcli general status",
            request: CommandRequest::new(Executable::Nmcli)
                .arg("general")?
                .arg("status")?
                .with_timeout(Duration::from_secs(10)),
        },
        Probe {
            label: "systemctl --user --version",
            request: CommandRequest::new(Executable::Systemctl)
                .arg("--user")?
                .arg("--version")?
                .with_timeout(Duration::from_secs(10)),
        },
        Probe {
            label: "fusermount3 --version",
            request: CommandRequest::new(Executable::Fusermount3)
                .arg("--version")?
                .with_timeout(Duration::from_secs(10)),
        },
    ])
}

fn expected_error_matches(expected: &ExpectedError, actual: &CommandError) -> bool {
    match (expected, actual) {
        (ExpectedError::NonZero, CommandError::NonZero { .. }) => true,
        (ExpectedError::NonZeroWithStderr(pattern), CommandError::NonZero { stderr, .. }) => {
            stderr.text.contains(pattern)
        }
        (ExpectedError::Timeout, CommandError::Timeout { .. }) => true,
        (ExpectedError::Cancelled, CommandError::Cancelled { .. }) => true,
        _ => false,
    }
}

fn expected_error_name(expected: &ExpectedError) -> &'static str {
    match expected {
        ExpectedError::NonZero => "nonzero",
        ExpectedError::NonZeroWithStderr(_) => "nonzero with stderr",
        ExpectedError::Timeout => "timeout",
        ExpectedError::Cancelled => "cancelled",
    }
}

fn error_kind(error: &CommandError) -> &'static str {
    match error {
        CommandError::MissingExecutable(_) => "missing executable",
        CommandError::InvalidArgument => "invalid argument",
        CommandError::Spawn { .. } => "spawn",
        CommandError::Timeout { .. } => "timeout",
        CommandError::Cancelled { .. } => "cancelled",
        CommandError::NonZero { .. } => "nonzero",
    }
}

fn print_output(label: &str, output: &CommandOutput) {
    println!("OK {label}: {}", output.command);
    if !output.stdout.text.trim().is_empty() {
        println!("  stdout: {}", first_line(&output.stdout.text));
    }
    if !output.stderr.text.trim().is_empty() {
        println!("  stderr: {}", first_line(&output.stderr.text));
    }
}

fn first_line(value: &str) -> String {
    value
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(160)
        .collect()
}

fn print_help() {
    println!("Usage: cargo run --example flatpak_host_runner_probe -- [--flatpak-host|--native]");
    println!();
    println!("  --flatpak-host  Run commands through flatpak-spawn --host. Default.");
    println!("  --native        Run commands directly for a host sanity check.");
    let auto = RuntimeCommandRunner::detect_current();
    println!("  detected runtime runner: {auto:?}");
}
