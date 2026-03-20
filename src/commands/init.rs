use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use std::fs;
use std::path::PathBuf;

struct Tool {
    name: &'static str,
    detected: bool,
    install: fn() -> Result<()>,
}

fn detect_claude_code() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".claude").is_dir())
        .unwrap_or(false)
}

fn detect_cursor() -> bool {
    let dir_exists = dirs::home_dir()
        .map(|h| h.join(".cursor").is_dir())
        .unwrap_or(false);
    dir_exists || which("cursor")
}

fn detect_windsurf() -> bool {
    let dir_exists = dirs::home_dir()
        .map(|h| h.join(".windsurf").is_dir())
        .unwrap_or(false);
    dir_exists || which("windsurf")
}

fn detect_codex() -> bool {
    let dir_exists = dirs::home_dir()
        .map(|h| h.join(".codex").is_dir())
        .unwrap_or(false);
    dir_exists || which("codex")
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn home() -> PathBuf {
    dirs::home_dir().expect("could not determine home directory")
}

fn install_claude_code() -> Result<()> {
    let skill_dir = home().join(".claude/skills/mandex");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
description: Use mandex (mx) to look up local, version-pinned documentation before generating code that uses any library API
---

When you need documentation for a library, check mandex first — it has local, offline, version-specific docs indexed and ready to search.

## Commands

- `mx search <package> "<query>"` — search a package's docs
- `mx search "<query>"` — search across all installed packages
- `mx show <package> <entry>` — get a specific documentation entry in full
- `mx list` — list installed packages
- `mx pull <package>@<version>` — download a package if not installed
- `mx sync` — auto-install docs for all project dependencies

## When to use

- Before generating code that calls a library API
- When unsure about a function signature, parameter, or pattern
- When the user asks about a specific class, method, or concept from a library
- When you need to verify current API behavior for a specific version

## Sub-agent pattern

For complex questions that span multiple documentation sections, spawn a sub-agent that runs several targeted searches and synthesizes the results before returning to the main conversation. This produces more complete answers than a single search.

## Notes

- mandex queries are local and instant — no network required after initial install
- Results are version-pinned to whatever version was pulled
- Run `mx sync` once in each project to pull docs for all detected dependencies
"#,
    )?;
    Ok(())
}

fn install_cursor() -> Result<()> {
    let rules_dir = home().join(".cursor");
    fs::create_dir_all(&rules_dir)?;
    let rules_file = rules_dir.join("rules");
    let content = r#"# mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool.

## Commands
- `mx search <package> "<query>"` — search installed docs
- `mx show <package> <entry>` — get a full documentation entry
- `mx sync` — install docs for all project dependencies

Prefer mx over web search for library documentation — results are local, fast, and version-pinned.
"#;
    if rules_file.exists() {
        let existing = fs::read_to_string(&rules_file)?;
        if !existing.contains("mandex") {
            fs::write(&rules_file, format!("{}\n\n{}", existing.trim_end(), content))?;
        }
    } else {
        fs::write(&rules_file, content)?;
    }
    Ok(())
}

fn install_windsurf() -> Result<()> {
    let rules_file = home().join(".windsurfrules");
    let content = r#"# mandex documentation lookup

When you need documentation for a library, use `mx search <package> "<query>"` or `mx show <package> <entry>`. Run `mx sync` once per project to install docs for all dependencies.
"#;
    if rules_file.exists() {
        let existing = fs::read_to_string(&rules_file)?;
        if !existing.contains("mandex") {
            fs::write(&rules_file, format!("{}\n\n{}", existing.trim_end(), content))?;
        }
    } else {
        fs::write(&rules_file, content)?;
    }
    Ok(())
}

fn install_codex() -> Result<()> {
    let codex_dir = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(".codex"));
    fs::create_dir_all(&codex_dir)?;
    let agents_file = codex_dir.join("AGENTS.md");
    let content = r#"## mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool instead of web search.

- `mx search <package> "<query>"` — search installed docs
- `mx search "<query>"` — search across all installed packages
- `mx show <package> <entry>` — get a full documentation entry
- `mx sync` — install docs for all project dependencies (reads package.json, requirements.txt, etc.)

Prefer mx over web search — results are local, fast, and version-pinned to the exact library version in use.
"#;
    if agents_file.exists() {
        let existing = fs::read_to_string(&agents_file)?;
        if !existing.contains("mandex") {
            fs::write(&agents_file, format!("{}\n\n{}", existing.trim_end(), content))?;
        }
    } else {
        fs::write(&agents_file, content)?;
    }
    Ok(())
}

pub fn run(yes: bool) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!();
    println!("  \x1b[1mmandex setup\x1b[0m");
    println!("  \x1b[2mConfigure AI coding assistant integrations\x1b[0m");
    println!();

    let tools: Vec<Tool> = vec![
        Tool {
            name: "Claude Code",
            detected: detect_claude_code(),
            install: install_claude_code,
        },
        Tool {
            name: "Cursor",
            detected: detect_cursor(),
            install: install_cursor,
        },
        Tool {
            name: "Windsurf",
            detected: detect_windsurf(),
            install: install_windsurf,
        },
        Tool {
            name: "Codex",
            detected: detect_codex(),
            install: install_codex,
        },
    ];

    // Show detection results
    let detected: Vec<&Tool> = tools.iter().filter(|t| t.detected).collect();
    let not_detected: Vec<&Tool> = tools.iter().filter(|t| !t.detected).collect();

    if !detected.is_empty() {
        println!(
            "  \x1b[32m✓\x1b[0m Detected: {}",
            detected
                .iter()
                .map(|t| t.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !not_detected.is_empty() {
        println!(
            "  \x1b[2m⊘ Not found: {}\x1b[0m",
            not_detected
                .iter()
                .map(|t| t.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!();

    let selected = if yes {
        // Non-interactive: install all detected
        tools
            .iter()
            .enumerate()
            .filter(|(_, t)| t.detected)
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    } else {
        let labels: Vec<String> = tools
            .iter()
            .map(|t| {
                if t.detected {
                    format!("{} (detected)", t.name)
                } else {
                    t.name.to_string()
                }
            })
            .collect();
        let defaults: Vec<bool> = tools.iter().map(|t| t.detected).collect();

        println!("  \x1b[2m<space> toggle  <enter> confirm\x1b[0m");
        println!();

        // Loop: select → confirm → if "no", go back to select
        loop {
            let sel = MultiSelect::with_theme(&theme)
                .with_prompt("Select integrations to install")
                .items(&labels)
                .defaults(&defaults)
                .interact()?;

            if sel.is_empty() {
                println!("  \x1b[2mNo integrations selected.\x1b[0m");
                println!();
                return Ok(());
            }

            let names: Vec<&str> = sel.iter().map(|&i| tools[i].name).collect();
            let proceed = Confirm::with_theme(&theme)
                .with_prompt(format!("Install {}?", names.join(", ")))
                .default(true)
                .interact()?;

            if proceed {
                break sel;
            }
            // "No" → loop back to selection
            println!();
        }
    };

    println!();

    // Install selected integrations
    for &i in &selected {
        let tool = &tools[i];
        match (tool.install)() {
            Ok(()) => println!("  \x1b[32m✓\x1b[0m {}", tool.name),
            Err(e) => println!("  \x1b[31m✗\x1b[0m {} — {}", tool.name, e),
        }
    }

    println!();
    println!(
        "  \x1b[1m\x1b[32mDone.\x1b[0m Run \x1b[1mmx sync\x1b[0m in any project to get started."
    );
    println!();

    Ok(())
}
