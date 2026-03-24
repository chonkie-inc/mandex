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

fn detect_cline() -> bool {
    // Cline is a VS Code extension — check for .cline or .clinerules
    dirs::home_dir()
        .map(|h| h.join(".cline").is_dir() || h.join(".clinerules").exists())
        .unwrap_or(false)
        || which("cline")
}

fn detect_copilot() -> bool {
    // GitHub Copilot — check for .github/copilot-instructions.md or copilot CLI
    std::path::Path::new(".github/copilot-instructions.md").exists() || which("github-copilot")
}

fn detect_openclaw() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".openclaw").is_dir())
        .unwrap_or(false)
        || which("openclaw")
}

fn detect_amp() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".amp").is_dir() || h.join(".ampcoderc").exists())
        .unwrap_or(false)
        || which("amp")
}

fn detect_antigravity() -> bool {
    // Google Antigravity — check for GEMINI.md or antigravity config
    std::path::Path::new("GEMINI.md").exists()
        || dirs::home_dir()
            .map(|h| h.join(".antigravity").is_dir())
            .unwrap_or(false)
}

fn detect_gemini() -> bool {
    // Gemini CLI
    which("gemini")
        || dirs::home_dir()
            .map(|h| h.join(".gemini").is_dir())
            .unwrap_or(false)
}

fn detect_clawdbot() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".clawdbot").is_dir())
        .unwrap_or(false)
        || which("clawdbot")
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

Use `mx` to search local documentation instead of guessing APIs. Results are offline, instant, and version-pinned. Search results are automatically reranked by semantic relevance.

## Workflow

1. `mx search <package> "<query>"` — find relevant entries
2. `mx show <package> "<entry>"` — read the full entry (use exact name from search results, including `>` hierarchy)
3. Write code using the verified API

## Key commands

- `mx search <package> "<query>"` — search within a package (default ~10 results, use `-n 5` to limit)
- `mx search "<query>"` — search across ALL installed packages (use `-n 3` to avoid large output)
- `mx show <package> "<entry>"` — show full entry content (exact name match; falls back to search if not found)
- `mx list` — show installed packages with entry counts and sizes
- `mx info <package>` — show details for a specific package (versions, paths)
- `mx pull <package>@<version>` — install docs for a package
- `mx sync` — install docs for all project dependencies (reads package.json, requirements.txt, Cargo.toml, etc.)
- `mx remove <package>` — remove an installed package (`--version` to remove a specific version)
- `mx init` — first-time setup: integrates mandex with AI coding assistants (CLAUDE.md, etc.)

## Important behaviors

- Both arguments to `search` are positional — always quote the query: `mx search nextjs "routing"`, NOT `mx search nextjs`
- When multiple versions are installed, search uses the latest version
- If a package isn't installed, you'll get: `Error: Package 'foo' is not installed. Run: mx pull foo`

## Example

```
$ mx search fastapi "dependency injection"
fastapi@0.115.0 — Dependencies
...
$ mx show fastapi "Dependencies"
```

## Tips

- Always search before generating code that calls a library API
- Use `-n 3` for exploratory searches across all packages
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

When you need documentation for a library, use the `mx` CLI tool. Results are local, fast, and version-pinned. Search results are automatically reranked by semantic relevance.

## Commands
- `mx search <package> "<query>"` — search within a package (use `-n 5` to limit results)
- `mx search "<query>"` — search across ALL installed packages (use `-n 3` to avoid large output)
- `mx show <package> "<entry>"` — show full entry content
- `mx list` — show installed packages
- `mx info <package>` — show package details
- `mx pull <package>@<version>` — install docs
- `mx sync` — install docs for all project dependencies
- `mx remove <package>` — remove a package

Both arguments to `search` are positional — always quote the query: `mx search nextjs "routing"`, NOT `mx search nextjs`.
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

When you need documentation for a library, use `mx search <package> "<query>"` or `mx show <package> "<entry>"`. Always quote the query. Use `-n 3` for global searches. Run `mx sync` once per project to install docs for all dependencies.
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

When you need documentation for a library, use the `mx` CLI tool instead of web search. Results are local, fast, version-pinned, and semantically reranked.

- `mx search <package> "<query>"` — search within a package (use `-n 5` to limit)
- `mx search "<query>"` — search across ALL installed packages (use `-n 3` to avoid large output)
- `mx show <package> "<entry>"` — show full entry content
- `mx list` — show installed packages
- `mx pull <package>@<version>` — install docs
- `mx sync` — install docs for all project dependencies (reads package.json, requirements.txt, etc.)
- `mx remove <package>` — remove a package

Both arguments to `search` are positional — always quote the query: `mx search nextjs "routing"`, NOT `mx search nextjs`.
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

const MANDEX_INSTRUCTIONS: &str = r#"## mandex documentation lookup

When you need documentation for a library, use the `mx` CLI tool instead of web search. Results are local, fast, version-pinned, and semantically reranked.

- `mx search <package> "<query>"` — search within a package (use `-n 5` to limit)
- `mx search "<query>"` — search across ALL installed packages (use `-n 3`)
- `mx show <package> "<entry>"` — show full entry content
- `mx sync` — install docs for all project dependencies
- `mx pull <package>@<version>` — install docs for a specific package
- `mx list` — show installed packages

Always search before generating code that calls a library API. Both arguments to `search` are positional — always quote the query.
"#;

fn install_cline() -> Result<()> {
    let cline_dir = home().join(".cline");
    fs::create_dir_all(&cline_dir)?;
    let rules_file = cline_dir.join("rules");
    append_or_create(&rules_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_copilot() -> Result<()> {
    let gh_dir = std::path::Path::new(".github");
    fs::create_dir_all(gh_dir)?;
    let instructions_file = gh_dir.join("copilot-instructions.md");
    append_or_create(&instructions_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_openclaw() -> Result<()> {
    let openclaw_dir = home().join(".openclaw");
    fs::create_dir_all(&openclaw_dir)?;
    let skills_dir = openclaw_dir.join("skills").join("mandex");
    fs::create_dir_all(&skills_dir)?;
    fs::write(skills_dir.join("SKILL.md"), MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_amp() -> Result<()> {
    // Amp uses AGENTS.md like Codex
    let agents_file = std::path::Path::new("AGENTS.md").to_path_buf();
    append_or_create(&agents_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_antigravity() -> Result<()> {
    // Google Antigravity uses GEMINI.md or AGENTS.md
    let gemini_file = std::path::Path::new("GEMINI.md").to_path_buf();
    append_or_create(&gemini_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_gemini() -> Result<()> {
    // Gemini CLI uses GEMINI.md
    let gemini_file = std::path::Path::new("GEMINI.md").to_path_buf();
    append_or_create(&gemini_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

fn install_clawdbot() -> Result<()> {
    let clawdbot_dir = home().join(".clawdbot");
    fs::create_dir_all(&clawdbot_dir)?;
    let config_file = clawdbot_dir.join("instructions.md");
    append_or_create(&config_file, MANDEX_INSTRUCTIONS)?;
    Ok(())
}

/// Append mandex instructions to a file if not already present, or create it.
fn append_or_create(path: &std::path::Path, content: &str) -> Result<()> {
    if path.exists() {
        let existing = fs::read_to_string(path)?;
        if !existing.contains("mandex") {
            fs::write(path, format!("{}\n\n{}", existing.trim_end(), content))?;
        }
    } else {
        fs::write(path, content)?;
    }
    Ok(())
}

// ─── clack-style UI helpers ──────────────────────────────────────────────

const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const BAR: &str = "│";
const S_STEP_SUBMIT: &str = "◇";
const S_STEP_ACTIVE: &str = "◆";

fn header() {
    println!();
    println!("  {GREEN} ███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗██╗  ██╗{RESET}");
    println!("  {GREEN} ████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝╚██╗██╔╝{RESET}");
    println!("  {GREEN} ██╔████╔██║███████║██╔██╗ ██║██║  ██║█████╗   ╚███╔╝{RESET}");
    println!("  {GREEN} ██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║██╔══╝   ██╔██╗{RESET}");
    println!("  {GREEN} ██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████╗██╔╝ ██╗{RESET}");
    println!("  {GREEN} ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝{RESET}");
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
        Tool {
            name: "Cline",
            detected: detect_cline(),
            install: install_cline,
        },
        Tool {
            name: "Copilot",
            detected: detect_copilot(),
            install: install_copilot,
        },
        Tool {
            name: "OpenClaw",
            detected: detect_openclaw(),
            install: install_openclaw,
        },
        Tool {
            name: "Amp",
            detected: detect_amp(),
            install: install_amp,
        },
        Tool {
            name: "Antigravity",
            detected: detect_antigravity(),
            install: install_antigravity,
        },
        Tool {
            name: "Gemini",
            detected: detect_gemini(),
            install: install_gemini,
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
