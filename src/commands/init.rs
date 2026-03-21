use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use std::fs;
use std::path::PathBuf;

use crate::config::{self, ConfigFile};

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
description: Look up local, version-pinned library documentation before writing code that uses any API
---

Use `mx` to search local documentation instead of guessing APIs. Results are offline, instant, and version-pinned.

## Workflow

1. `mx search <package> "<query>"` — find relevant entries (use `--rerank` for better accuracy)
2. `mx show <package> <entry>` — read the full entry from the search results
3. Write code using the verified API

## Key commands

- `mx search <package> "<query>"` — search within a package (`-n 5` to limit results)
- `mx search "<query>"` — search across all installed packages
- `mx show <package> <entry>` — show full content of a specific entry
- `mx pull <package>@<version>` — install docs for a package
- `mx sync` — install docs for all project dependencies (reads package.json, requirements.txt, Cargo.toml, etc.)
- `mx list` — show installed packages

## Example

```
$ mx search fastapi "dependency injection"
fastapi@0.115.0 — Dependencies
...
$ mx show fastapi "Dependencies"
```

## Tips

- Always search before generating code that calls a library API
- For broad questions, run 2-3 targeted searches in a sub-agent and synthesize
- Run `mx sync` once per project to pull docs for all detected dependencies
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

// ─── clack-style UI helpers ──────────────────────────────────────────────

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const BAR: &str = "│";
const S_STEP_SUBMIT: &str = "◇";
const S_STEP_ACTIVE: &str = "◆";

fn header() {
    println!();
    println!(
        "  {CYAN}{BOLD}╭─────────────────────────────────────╮{RESET}"
    );
    println!(
        "  {CYAN}{BOLD}│{RESET}  {BOLD}mandex{RESET} — docs for AI agents       {CYAN}{BOLD}│{RESET}"
    );
    println!(
        "  {CYAN}{BOLD}╰─────────────────────────────────────╯{RESET}"
    );
    println!();
    let version = env!("CARGO_PKG_VERSION");
    println!("  {DIM}v{version} · https://mandex.dev{RESET}");
}

fn step_title(title: &str) {
    println!();
    let line = "─".repeat(40);
    println!("  {S_STEP_SUBMIT}  {BOLD}{title}{RESET} {DIM}{line}{RESET}");
}

fn bar_line(text: &str) {
    println!("  {BAR}  {text}");
}

fn bar_empty() {
    println!("  {BAR}");
}

pub fn run(yes: bool) -> Result<()> {
    let theme = ColorfulTheme::default();

    header();

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

    // Filter to detected tools only
    let detected_tools: Vec<(usize, &Tool)> = tools
        .iter()
        .enumerate()
        .filter(|(_, t)| t.detected)
        .collect();

    // ── Selection ────────────────────────────────────────────────────────
    let selected: Vec<usize> = if detected_tools.is_empty() {
        step_title("AI coding tools");
        bar_empty();
        bar_line(&format!("{DIM}No tools detected (Claude Code, Cursor, Windsurf, Codex).{RESET}"));
        bar_line(&format!("{DIM}Run mx init again after installing one.{RESET}"));
        vec![]
    } else if yes {
        detected_tools.iter().map(|(i, _)| *i).collect::<Vec<_>>()
    } else {
        step_title("Install integrations");
        bar_empty();
        bar_line(&format!(
            "{DIM}<space> toggle · <enter> confirm{RESET}"
        ));
        bar_empty();

        let labels: Vec<&str> = detected_tools.iter().map(|(_, t)| t.name).collect();

        let sel = MultiSelect::with_theme(&theme)
            .with_prompt(format!("  {S_STEP_ACTIVE}  Choose"))
            .items(&labels)
            .interact()?;

        // Map back to original tool indices
        sel.iter().map(|&i| detected_tools[i].0).collect::<Vec<_>>()
    };

    // ── Install ──────────────────────────────────────────────────────────
    if !selected.is_empty() {
        step_title("Installing");
        bar_empty();

        for &i in &selected {
            let tool = &tools[i];
            match (tool.install)() {
                Ok(()) => bar_line(&format!("{GREEN}✓{RESET}  {}", tool.name)),
                Err(e) => bar_line(&format!("{RED}✗{RESET}  {} — {}", tool.name, e)),
            }
        }
    }

    // ── Reranker ──────────────────────────────────────────────────────────
    let cfg = ConfigFile::load()?;
    let enable_reranker;

    if yes {
        enable_reranker = true;
    } else {
        step_title("Search quality");
        bar_empty();
        bar_line("The reranker uses a local ONNX model (~19 MB) to");
        bar_line("re-score results for much better search accuracy.");
        bar_empty();

        enable_reranker = Confirm::with_theme(&theme)
            .with_prompt(format!("  {S_STEP_ACTIVE}  Enable reranker? (recommended)"))
            .default(true)
            .interact()?;
    }

    // Write config with reranker preference
    let config_path = crate::storage::paths::mandex_dir()?.join("config.toml");
    if !config_path.exists() || yes || enable_reranker != cfg.search.rerank {
        let toml_content = format!(
            "[search]\nresults = {}\nrerank = {}\nrerank_model = \"{}\"\nrerank_candidates = {}\n\n[network]\ncdn_url = \"{}\"\napi_url = \"{}\"\n\n[display]\ncolor = \"{}\"\n",
            cfg.search.results,
            enable_reranker,
            cfg.search.rerank_model,
            cfg.search.rerank_candidates,
            cfg.network.cdn_url,
            cfg.network.api_url,
            cfg.display.color,
        );
        std::fs::write(&config_path, toml_content)?;
    }

    if enable_reranker {
        bar_line(&format!("{GREEN}✓{RESET}  Reranker enabled"));
        bar_empty();
        bar_line(&format!("{DIM}Downloading reranker model...{RESET}"));
        match config::ensure_setup(&ConfigFile {
            search: crate::config::SearchConfig {
                rerank: true,
                ..cfg.search
            },
            ..cfg
        }) {
            Ok(()) => bar_line(&format!("{GREEN}✓{RESET}  Model ready")),
            Err(e) => bar_line(&format!("{RED}✗{RESET}  Model download failed: {e}")),
        }
    } else {
        bar_line(&format!("{DIM}○  Reranker disabled{RESET}"));
    }

    // ── Next steps ──────────────────────────────────────────────────────
    bar_empty();
    step_title("Next steps");
    bar_empty();
    bar_line(&format!("{BOLD}mx sync{RESET}        install docs for all project dependencies"));
    bar_line(&format!("{BOLD}mx pull <pkg>{RESET}   download a specific package"));
    bar_line(&format!("{BOLD}mx search{RESET}       search across installed docs"));
    bar_empty();
    println!(
        "  └  {GREEN}{BOLD}Done.{RESET} Run {BOLD}mx sync{RESET} in any project to get started."
    );
    println!();

    Ok(())
}
