#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Development-time binary artifact and unused dependency analyzer.
//!
//! Runs `cargo bloat`, `cargo udeps`, and `twiggy` via [`std::process::Command`],
//! tolerates missing tools or non-zero exits, and produces a single consolidated
//! Markdown report to `stdout` and `diorama-artifact-report.md`.

use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Output file written in the current working directory.
pub const REPORT_FILENAME: &str = "diorama-artifact-report.md";

/// Captured result of running a single external analysis command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRun {
    /// Human-readable section title.
    pub title: String,
    /// Full command line string displayed in the report.
    pub command_line: String,
    /// Process exit code (`None` if the binary could not be spawned or was terminated by signal).
    pub exit_code: Option<i32>,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
    /// Optional spawn error (e.g., tool not found on `PATH`).
    pub spawn_error: Option<String>,
}

impl ToolRun {
    /// Returns `true` when the command exited with status `0`.
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        matches!(self.exit_code, Some(0)) && self.spawn_error.is_none()
    }
}

/// Runs all three artifact analysis tools (`cargo bloat`, `cargo udeps`, `twiggy`)
/// from `project_root` and returns the rendered Markdown report.
#[must_use]
pub fn run_all_analyses(project_root: &Path) -> String {
    let mut runs = vec![
        invoke_tool(
            project_root,
            "1a. Binary Size by Crate (`cargo bloat --crates`)",
            "cargo",
            &["bloat", "--release", "--crates", "-n", "15"],
        ),
        invoke_tool(
            project_root,
            "1b. Binary Size by Function (`cargo bloat`)",
            "cargo",
            &["bloat", "--release", "-n", "15"],
        ),
        invoke_tool(
            project_root,
            "2. Unused Dependencies (`cargo udeps`)",
            "cargo",
            &["udeps", "--all-targets"],
        ),
    ];

    let artifact = locate_release_binary(project_root);
    let artifact_arg = artifact.as_ref().map_or_else(
        || "target/release/diorama".to_string(),
        |p| p.to_string_lossy().into_owned(),
    );
    runs.push(invoke_tool(
        project_root,
        "3a. Shallow & Retained Code Size (`twiggy top`)",
        "twiggy",
        &["top", "-n", "15", &artifact_arg],
    ));
    runs.push(invoke_tool(
        project_root,
        "3b. Call-Graph Dominators (`twiggy dominators`)",
        "twiggy",
        &["dominators", "-r", "15", &artifact_arg],
    ));

    render_consolidated_report(project_root, artifact.as_deref(), &runs)
}

/// Executes a single command in `workdir` and captures its `stdout`, `stderr`, and status.
#[must_use]
pub fn invoke_tool(workdir: &Path, title: &str, program: &str, args: &[&str]) -> ToolRun {
    let command_line = if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {}", args.join(" "))
    };

    match Command::new(program)
        .args(args)
        .current_dir(workdir)
        .output()
    {
        Ok(output) => ToolRun {
            title: title.to_string(),
            command_line,
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            spawn_error: None,
        },
        Err(err) => ToolRun {
            title: title.to_string(),
            command_line,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            spawn_error: Some(format!(
                "{err} (ensure `{program}` is installed via `cargo install cargo-bloat cargo-udeps twiggy`)"
            )),
        },
    }
}

/// Renders the consolidated Markdown report from collected tool runs.
#[must_use]
pub fn render_consolidated_report(
    project_root: &Path,
    artifact: Option<&Path>,
    runs: &[ToolRun],
) -> String {
    let mut out = String::with_capacity(8192);
    let _ = writeln!(out, "# Diorama Artifact Analysis Report\n");
    let _ = writeln!(out, "- **Project Root**: `{}`", project_root.display());
    match artifact {
        Some(path) => {
            let size_mib = fs::metadata(path).map_or(0, |m| m.len() / (1024 * 1024));
            let _ = writeln!(
                out,
                "- **Release Artifact**: `{}` (~{size_mib} MiB)\n",
                path.display()
            );
        },
        None => {
            let _ = writeln!(
                out,
                "- **Release Artifact**: *(none found under `target/release/`; `cargo bloat` builds one when available)*\n"
            );
        },
    }

    render_executive_summary(&mut out, runs);

    for run in runs {
        render_tool_section(&mut out, run);
    }
    out
}

fn render_executive_summary(out: &mut String, runs: &[ToolRun]) {
    let _ = writeln!(out, "## Executive Summary\n");
    let _ = writeln!(out, "| Check | Command | Status |");
    let _ = writeln!(out, "| :--- | :--- | :--- |");
    for run in runs {
        let status = if run.succeeded() {
            "OK (exit 0)".to_string()
        } else if let Some(code) = run.exit_code {
            format!("Non-zero (exit {code})")
        } else {
            "Unavailable / Not Installed".to_string()
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {} |",
            run.title, run.command_line, status
        );
    }
    let _ = writeln!(out);

    if let Some(crates_run) = runs.first() {
        let highlights = extract_bloat_highlights(&crates_run.stdout, 5);
        if !highlights.is_empty() {
            let _ = writeln!(out, "### Largest Crates (Top 5)");
            for line in highlights {
                let _ = writeln!(out, "- `{line}`");
            }
            let _ = writeln!(out);
        }
    }

    if let Some(fns_run) = runs.get(1) {
        let highlights = extract_bloat_highlights(&fns_run.stdout, 5);
        if !highlights.is_empty() {
            let _ = writeln!(out, "### Largest Functions (Top 5)");
            for line in highlights {
                let _ = writeln!(out, "- `{line}`");
            }
            let _ = writeln!(out);
        }
    }

    if let Some(udeps_run) = runs.get(2) {
        let unused = extract_udeps_highlights(&udeps_run.stdout, &udeps_run.stderr);
        if !unused.is_empty() {
            let _ = writeln!(out, "### Unused Dependencies (`cargo udeps`)");
            for line in unused {
                let _ = writeln!(out, "- `{line}`");
            }
            let _ = writeln!(out);
        }
    }
}

fn render_tool_section(out: &mut String, run: &ToolRun) {
    let _ = writeln!(out, "## {}\n", run.title);
    let _ = writeln!(out, "- **Command**: `{}`", run.command_line);
    if let Some(err) = &run.spawn_error {
        let _ = writeln!(out, "- **Status**: Failed to launch — {err}\n");
        return;
    }
    let code_str = run
        .exit_code
        .map_or_else(|| "terminated by signal".to_string(), |c| c.to_string());
    let _ = writeln!(out, "- **Exit Code**: `{code_str}`\n");

    if !run.stdout.is_empty() {
        let _ = writeln!(out, "### Standard Output\n\n```text\n{}\n```\n", run.stdout);
    }
    if !run.stderr.is_empty() {
        let _ = writeln!(out, "### Standard Error\n\n```text\n{}\n```\n", run.stderr);
    }
    if run.stdout.is_empty() && run.stderr.is_empty() {
        let _ = writeln!(out, "*(no output captured)*\n");
    }
}

/// Extracts up to `limit` data rows from `cargo bloat` tabular output.
#[must_use]
pub fn extract_bloat_highlights(stdout: &str, limit: usize) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains('%') && !line.starts_with("File") && !line.starts_with("Total")
        })
        .take(limit)
        .map(ToOwned::to_owned)
        .collect()
}

/// Extracts unused crate lines reported by `cargo udeps`.
#[must_use]
pub fn extract_udeps_highlights(stdout: &str, stderr: &str) -> Vec<String> {
    stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .filter(|line| {
            line.starts_with("unused")
                || line.contains("unused dependencies:")
                || line.starts_with("└──")
                || line.starts_with("├──")
        })
        .take(15)
        .map(ToOwned::to_owned)
        .collect()
}

fn locate_release_binary(project_root: &Path) -> Option<PathBuf> {
    let release_dir = project_root.join("target/release");
    for candidate in ["diorama", "dioramactl", "diorama-artifact-analysis"] {
        let path = release_dir.join(candidate);
        if path.is_file() {
            return Some(path);
        }
    }
    let entries = fs::read_dir(&release_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path.extension().is_none()
            && !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
        {
            return Some(path);
        }
    }
    None
}

fn main() -> io::Result<()> {
    let cwd = std::env::current_dir()?;
    let report = run_all_analyses(&cwd);
    let report_path = cwd.join(REPORT_FILENAME);
    let _ = fs::write(&report_path, &report);

    let mut stdout = io::stdout().lock();
    stdout.write_all(report.as_bytes())?;
    writeln!(
        stdout,
        "\n---\nConsolidated report written to `{}`",
        report_path.display()
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bloat_and_udeps_highlights_parse_expected_rows() {
        let sample_bloat = "\
File  .text     Size          Crate
0.8%  14.2% 1.1MiB          tokio
0.5%   9.1% 720KiB          quiche
4.2% 100.0% 7.9MiB          .text section size
";
        let top = extract_bloat_highlights(sample_bloat, 5);
        assert_eq!(top.len(), 3);
        assert!(top[0].contains("tokio"));

        let sample_udeps =
            "unused dependencies:\n`diorama v0.1.0`\n└─── dependencies\n     └─── \"foo\"";
        let unused = extract_udeps_highlights(sample_udeps, "");
        assert_eq!(unused.len(), 3);
    }

    #[test]
    fn missing_tool_is_captured_gracefully_in_report() {
        let run = invoke_tool(
            Path::new("."),
            "Synthetic Check",
            "nonexistent-diorama-tool-xyz",
            &["--version"],
        );
        assert!(!run.succeeded());
        assert!(run.spawn_error.is_some());

        let md = render_consolidated_report(Path::new("."), None, &[run]);
        assert!(md.contains("# Diorama Artifact Analysis Report"));
        assert!(md.contains("Unavailable / Not Installed"));
    }
}
