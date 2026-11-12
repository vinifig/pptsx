//! pptsx-rs — MVP do pptsx em Rust: TSX → PPTX num binário só, sem Node e sem motor JS.

// Tuplas e closures nomeadas pelo uso (ícones por célula, testes por kind): um alias por tipo
// não ajudaria a ler.
#![allow(clippy::type_complexity)]

mod catalog;
mod config;
mod containers;
mod css;
mod diag;
mod dsinfo;
mod dsprops;
mod embed;
mod emit;
mod gradient;
mod grid;
mod init;
mod js;
mod layout;
mod lint;
mod mcp;
mod model;
mod numfmt;
mod objects;
mod package;
mod preview;
mod registry;
mod report;
mod serve;
mod skill;
mod text;
mod types;
mod validate;
mod yoga;
mod yoga_layout;
mod vector;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use diag::{Diags, Level};

const USAGE: &str = "pptsx-rs — TSX → PPTX num binário só

uso:
  pptsx-rs build <deck.tsx> [--size 16x9,4x3] [--out dist/] [--lang pt-BR] [--line-breaks forced|auto]
                 [--preview native|none] [--preview-width 1280] [--embed-fonts] [--watch] [--serve [--port 4173]]
  pptsx-rs check <deck.tsx> [--size 16x9] [--debug] [--json]   avalia, faz o layout e mostra diagnósticos
                                                   (--debug: frames em pt de cada nó)
  pptsx-rs tree  <deck.tsx> [--size 16x9]     mostra a árvore avaliada do deck
  pptsx-rs lint  <deck.tsx> [--size 16x9,4x3] [--json] [--verbose]
                 regras da engine e as dos DS registrados no pptsx.config.ts; os achados que são
                 do DS (não do deck) só saem com --verbose ou no --json (\"origin\": \"ds\")
  pptsx-rs init  [pasta] [--ds <pacote|caminho>] [--force] [--json]
                                                   cria deck.tsx, tsconfig.json, pptsx.config.ts e os tipos do
                                                   pptsx para o editor (.pptsx/types); com --ds, o deck usa o DS
  pptsx-rs types [pasta]                          (re)grava os tipos do pptsx e do @pptsx/ds-kit e o `typeRoots` do tsconfig
  pptsx-rs skill [--out <pasta>] [--name <nome>] [--description <texto>] [--ds <caminho|pacote>]
                 [--bin <binário> | --no-bin]
                                                   a skill para agentes (SKILL.md, references/, mcp/) com este
                                                   binário em bin/; com --ds <caminho>, o DS vai junto em ds/ e a
                                                   pasta skill/ dele (SKILL.md, references/*.md) entra na skill
  pptsx-rs catalog [--ds <caminho|pacote>]... [--registry <arquivo>] [--out dist/catalog] [--size 16x9,4x3]
                 [--json | --prompt]
                                                   o catálogo visual dos design systems registrados: index.html
                                                   (galeria com cada caso de cada componente), catalog.json (o
                                                   contrato), prompt.md e .pptx por DS; --json/--prompt só imprimem
  pptsx-rs registry list|add <ds> [--id <id>]|remove <id> [--registry <arquivo>]
                                                   o pptsx.registry.json: os DS que catalog e mcp enxergam sem --ds
  pptsx-rs mcp [--ds <caminho|pacote>]...         servidor MCP por stdio (build, diagnose,
                                                   preview, lint, guide, schema, catalog)

@pptsx/ds-kit (tokens, tema, variantes, slots, regras, createDs) vem dentro do binário: um DS que o importa
compila sem node_modules.

tamanhos: 16x9, 16x10, 4x3, wide ou <L>x<A>in (ex.: 10x7.5in)";

struct Args {
    command: String,
    deck: PathBuf,
    /// None = sem --size: vale o da config, senão 16x9.
    sizes: Option<Vec<js::DeckCtx>>,
    out: PathBuf,
    lang: Option<String>,
    /// check --debug: árvore posicionada (frames em pt) de cada slide.
    debug: bool,
    /// build: PNG de cada slide e overview (como o `--preview native` do pptsx).
    preview: bool,
    preview_width: u32,
    /// build --watch: refaz o build quando o deck, um módulo importado ou a config mudam.
    watch: bool,
    /// lint --json.
    json: bool,
    /// lint --verbose: mostra também os achados do design system (`origin: "ds"`).
    verbose: bool,
    /// lint --deck-only: tira os achados do design system também do --json (o MCP usa).
    deck_only: bool,
    /// --line-breaks: "forced" (uma quebra por linha calculada) ou "auto" (o renderizador quebra).
    line_breaks: Option<String>,
    /// Interno (o `--watch` usa): grava os arquivos lidos pela avaliação, um por linha.
    deps_out: Option<PathBuf>,
    /// build --serve: preview ao vivo no navegador (implica --watch).
    serve: bool,
    /// --embed-fonts: embute as faces do autor usadas (vence `embedFonts` da config).
    embed_fonts: bool,
    port: u16,
    /// Interno (o `--serve` usa): grava state.json, diagnostics.json e os SVGs dos slides nesta pasta.
    state_out: Option<PathBuf>,
}

fn parse_size(token: &str) -> Result<js::DeckCtx, String> {
    let preset = |name: &str, w: f64, h: f64| js::DeckCtx { name: name.into(), width: w, height: h };
    match token {
        "16x9" => return Ok(preset("16x9", 720.0, 405.0)),
        "16x10" => return Ok(preset("16x10", 720.0, 450.0)),
        "4x3" => return Ok(preset("4x3", 720.0, 540.0)),
        "wide" => return Ok(preset("wide", 960.0, 540.0)),
        _ => {}
    }
    let custom = token.strip_suffix("in").and_then(|t| t.split_once('x')).and_then(|(w, h)| Some((w.parse::<f64>().ok()?, h.parse::<f64>().ok()?)));
    match custom {
        Some((w, h)) if w > 0.0 && h > 0.0 => Ok(js::DeckCtx { name: format!("custom-{w}x{h}"), width: w * 72.0, height: h * 72.0 }),
        _ => Err(format!("tamanho \"{token}\" desconhecido. Válidos: 16x9, 16x10, 4x3, wide; custom: \"<L>x<A>in\" (ex.: \"10x7.5in\")")),
    }
}

fn parse_args() -> Result<Args, String> {
    let mut raw = std::env::args().skip(1);
    let command = raw.next().ok_or_else(|| USAGE.to_string())?;
    if command == "--help" || command == "-h" {
        return Err(USAGE.into());
    }
    if command == "--version" || command == "-V" {
        println!("pptsx-rs {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    let mut deck = None;
    let mut sizes = None;
    let mut out = PathBuf::from("dist");
    let mut lang = None;
    let mut debug = false;
    let mut preview = true;
    let mut watch = false;
    let mut json = false;
    let mut verbose = false;
    let mut deck_only = false;
    let mut line_breaks = None;
    let mut deps_out = None;
    let mut serve = false;
    let mut embed_fonts = false;
    let mut port: u16 = 4173;
    let mut state_out = None;
    let mut preview_width = preview::PREVIEW_WIDTH;
    while let Some(a) = raw.next() {
        match a.as_str() {
            "--size" => {
                let v = raw.next().ok_or("--size precisa de um valor")?;
                sizes = Some(v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(parse_size).collect::<Result<_, _>>()?);
            }
            "--out" => out = PathBuf::from(raw.next().ok_or("--out precisa de uma pasta")?),
            "--deps-out" => deps_out = Some(PathBuf::from(raw.next().ok_or("--deps-out precisa de um arquivo")?)),
            "--state-out" => state_out = Some(PathBuf::from(raw.next().ok_or("--state-out precisa de uma pasta")?)),
            "--serve" => {
                serve = true;
                watch = true;
            }
            "--embed-fonts" => embed_fonts = true,
            "--port" => port = raw.next().and_then(|v| v.parse().ok()).ok_or("--port precisa de um número")?,
            "--lang" => lang = Some(raw.next().ok_or("--lang precisa de um valor")?),
            // Como o pptsx: error sempre sai com 1; a flag fica aceita por compatibilidade.
            "--strict" => {}
            "--debug" => debug = true,
            "--watch" => watch = true,
            "--json" => json = true,
            "--verbose" => verbose = true,
            "--deck-only" => deck_only = true,
            "--line-breaks" => {
                line_breaks = match raw.next().as_deref() {
                    Some(v @ ("forced" | "auto")) => Some(v.to_string()),
                    other => return Err(format!("--line-breaks inválido: {:?} (use forced ou auto)", other.unwrap_or(""))),
                }
            }
            "--preview" => {
                preview = match raw.next().as_deref() {
                    Some("native") => true,
                    Some("none") => false,
                    other => return Err(format!("--preview inválido: {:?} (use native ou none)", other.unwrap_or(""))),
                }
            }
            "--preview-width" => {
                preview_width = raw.next().and_then(|v| v.parse().ok()).filter(|w| *w > 0).ok_or("--preview-width precisa de um inteiro positivo (px)")?;
            }
            other if other.starts_with('-') => return Err(format!("opção desconhecida: {other}\n\n{USAGE}")),
            other => deck = Some(PathBuf::from(other)),
        }
    }
    let deck = deck.ok_or_else(|| format!("faltou o arquivo do deck\n\n{USAGE}"))?;
    Ok(Args { command, deck, sizes, out, lang, debug, preview, preview_width, watch, json, verbose, deck_only, line_breaks, deps_out, serve, port, state_out, embed_fonts })
}

fn dump(v: &js::Value, depth: usize) {
    let pad = "  ".repeat(depth);
    match v {
        js::Value::Element(e) => {
            let mut props = String::new();
            for (k, val) in e.props.iter() {
                let mut s = String::new();
                js::value::to_json(val, &mut s);
                props.push_str(&format!(" {k}={s}"));
            }
            println!("{pad}<{}{props}>", e.kind.name());
            for c in &e.children {
                dump(c, depth + 1);
            }
        }
        other => println!("{pad}{:?}", other.to_js_string()),
    }
}

/// Linhas de progresso: no stdout, ou no stderr quando o stdout é só o JSON.
fn progress(json: bool, line: &str) {
    if json {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::new();
    js::value::to_json(&js::value::str_val(s), &mut out);
    out
}

fn stem_of(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "deck".into())
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.first().map(String::as_str) == Some("init") {
        return init::run(&raw[1..]);
    }
    if raw.first().map(String::as_str) == Some("types") {
        return init::run_types(&raw[1..]);
    }
    if raw.first().map(String::as_str) == Some("catalog") {
        return catalog::run(&raw[1..]);
    }
    if raw.first().map(String::as_str) == Some("registry") {
        return registry::run(&raw[1..]);
    }
    if raw.first().map(String::as_str) == Some("skill") {
        return skill::run(&raw[1..]);
    }
    if raw.first().map(String::as_str) == Some("mcp") {
        let ds: Vec<String> = raw.iter().enumerate().filter(|(i, a)| *i > 0 && raw[i - 1] == "--ds" && !a.starts_with("--")).map(|(_, a)| a.clone()).collect();
        return mcp::serve(&ds);
    }
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    if !matches!(args.command.as_str(), "build" | "check" | "tree" | "lint") {
        eprintln!("comando desconhecido: {}\n\n{USAGE}", args.command);
        return ExitCode::from(2);
    }
    if !args.watch {
        let mut deps = Vec::new();
        let code = run(&args, &mut deps);
        if let Some(file) = &args.deps_out {
            deps.sort();
            deps.dedup();
            let lines: Vec<String> = deps.iter().map(|d| d.display().to_string()).collect();
            let _ = std::fs::write(file, lines.join("\n"));
        }
        return code;
    }
    watch_loop(&raw, &args)
}

fn mtimes(files: &[PathBuf]) -> Vec<Option<std::time::SystemTime>> {
    files.iter().map(|f| std::fs::metadata(f).and_then(|m| m.modified()).ok()).collect()
}

/// Observa o deck, os módulos que ele importou e a config; refaz o build a cada mudança
/// (debounce de 150 ms, DEFAULTS.watchDebounceMs). Cada build roda num processo filho (o mesmo
/// binário, sem `--watch` e com `--deps-out`): o avaliador vaza a AST de propósito (`Box::leak`,
/// processo curto), e assim a memória volta a zero a cada rebuild. Ctrl+C sai.
fn watch_loop(raw: &[String], args: &Args) -> ExitCode {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m não achei o próprio binário: {e}");
            return ExitCode::from(1);
        }
    };
    let deps_file = std::env::temp_dir().join(format!("pptsx-rs-deps-{}.txt", std::process::id()));
    // --serve e --port são do pai; o filho recebe --state-out com a pasta que o servidor lê.
    let mut skip_next = false;
    let child_args: Vec<&String> = raw
        .iter()
        .filter(|a| {
            if skip_next {
                skip_next = false;
                return false;
            }
            if *a == "--port" {
                skip_next = true;
                return false;
            }
            *a != "--watch" && *a != "--serve"
        })
        .collect();
    let state_dir = std::env::temp_dir().join(format!("pptsx-rs-serve-{}", std::process::id()));
    let server = if args.serve {
        let _ = std::fs::create_dir_all(&state_dir);
        match serve::Server::start(args.port, state_dir.clone()) {
            Ok(s) => {
                println!("  preview ao vivo em \x1b[1mhttp://127.0.0.1:{}/\x1b[0m", s.port);
                Some(s)
            }
            Err(e) => {
                eprintln!("\x1b[31merro\x1b[0m não consegui abrir a porta {}: {e}", args.port);
                return ExitCode::from(1);
            }
        }
    } else {
        None
    };
    loop {
        let mut deps = vec![std::fs::canonicalize(&args.deck).unwrap_or_else(|_| args.deck.clone())];
        let _ = std::fs::remove_file(&deps_file);
        let mut command = std::process::Command::new(&exe);
        command.args(&child_args).arg("--deps-out").arg(&deps_file);
        if args.serve {
            command.arg("--state-out").arg(&state_dir);
        }
        // Com servidor, a saída do filho é repetida aqui e o erro (deck que não avalia) vai à página.
        let outcome = if server.is_some() { command.output().map(|o| (o.status, Some(o))) } else { command.status().map(|s| (s, None)) };
        let (status, captured) = match outcome {
            Ok(v) => v,
            Err(e) => {
                eprintln!("\x1b[31merro\x1b[0m não consegui rodar o build: {e}");
                return ExitCode::from(1);
            }
        };
        if let (Some(server), Some(out)) = (&server, captured) {
            print!("{}", String::from_utf8_lossy(&out.stdout));
            eprint!("{}", String::from_utf8_lossy(&out.stderr));
            let stderr = String::from_utf8_lossy(&out.stderr);
            let error = (!status.success() && !state_dir.join("state.json").exists() || stderr.contains("\x1b[31merro")).then(|| mcp::strip_ansi(stderr.trim()));
            server.notify(error);
        }
        if let Ok(list) = std::fs::read_to_string(&deps_file) {
            deps.extend(list.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
        }
        deps.sort();
        deps.dedup();
        println!("\x1b[2m  observando {} arquivo(s)… (Ctrl+C para sair)\x1b[0m", deps.len());
        let before = mtimes(&deps);
        loop {
            std::thread::sleep(std::time::Duration::from_millis(150));
            if mtimes(&deps) != before {
                // Editor que salva em duas etapas: espera assentar antes de reconstruir.
                std::thread::sleep(std::time::Duration::from_millis(150));
                println!();
                break;
            }
        }
    }
}

fn run(args: &Args, deps: &mut Vec<PathBuf>) -> ExitCode {
    let started = Instant::now();
    // Flags da CLI vencem a config; o que nenhuma das duas define fica com o padrão.
    let config = match config::load(&args.deck) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(1);
        }
    };
    // Flags vencem a config: largura de quebra e modo das quebras de linha no arquivo.
    let text_options = text::TextOptions {
        safety: config.text_safety.unwrap_or(0.97),
        auto_breaks: args.line_breaks.clone().or_else(|| config.line_breaks.clone()).as_deref() == Some("auto"),
    };
    let sizes = match args.sizes.clone() {
        Some(s) => s,
        None if !config.sizes.is_empty() => match config.sizes.iter().map(|t| parse_size(t)).collect::<Result<Vec<_>, _>>() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("\x1b[31merro\x1b[0m pptsx.config.ts: {e}");
                return ExitCode::from(2);
            }
        },
        None => vec![parse_size("16x9").expect("preset")],
    };
    if args.command == "lint" {
        return match lint::lint(&args.deck, &sizes, &config, text_options) {
            Ok(findings) => {
                println!("{}", if args.json { lint::json(&findings, !args.deck_only) } else { lint::summary(&findings, args.verbose && !args.deck_only) });
                // Como o pptsx lint: sai 1 quando há achado error ou warning.
                if findings.iter().any(|f| f.severity != "info") { ExitCode::from(1) } else { ExitCode::SUCCESS }
            }
            Err(e) => {
                eprintln!("\x1b[31merro\x1b[0m {e}");
                ExitCode::from(1)
            }
        };
    }
    let lang = args.lang.clone().or_else(|| config.lang.clone()).unwrap_or_else(|| "pt-BR".into());
    if let Some(p) = &config.path {
        deps.push(p.clone());
    }
    let (interp, decks) = match js::evaluate_deck_with(&args.deck, &sizes, deps) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(1);
        }
    };
    if args.command == "tree" {
        dump(&js::Value::Element(decks[0].clone()), 0);
        return ExitCode::SUCCESS;
    }
    let eval_ms = started.elapsed().as_secs_f64() * 1000.0;
    if let Some(p) = &config.path {
        progress(args.json, &format!("  config {}", p.display()));
    }
    let (fonts, font_warnings) = text::Fonts::new(&config.fonts, text_options);
    let stem = stem_of(&args.deck);
    let title = config.title.clone().unwrap_or_else(|| stem.clone());
    let deck_path = std::fs::canonicalize(&args.deck).unwrap_or(args.deck.clone());
    if args.command == "build"
        && let Err(e) = std::fs::create_dir_all(&args.out) {
            eprintln!("\x1b[31merro\x1b[0m não foi possível criar {}: {e}", args.out.display());
            return ExitCode::from(1);
        }
    let mut failed = false;
    let mut all: Vec<(String, diag::Diag)> = Vec::new();
    // --serve (state_out): um JSON por tamanho com os slides, os frames por path e os diagnósticos.
    let mut state_sizes: Vec<String> = Vec::new();
    let mut notes: Vec<String> = config.warnings.iter().chain(&font_warnings).cloned().collect();
    let mut interp = interp;
    // console.warn/error de módulos do deck e dos DS: diagnósticos do deck inteiro (slide 0).
    let module_notes: Vec<String> = std::mem::take(&mut interp.notes);
    let containers = std::cell::RefCell::new(model::ContainerRun::new(interp));
    for (ctx, deck) in sizes.iter().zip(&decks) {
        let t0 = Instant::now();
        // Como o resolve do pptsx: prop removida ou inválida derruba o build (sem .pptx).
        let mut checked = validate::validate(deck, ctx);
        if let Some(f) = &checked.fatal {
            eprintln!("{}", f.text());
            return ExitCode::from(1);
        }
        let (spec, diags) = match containers::resolve(deck, ctx, &deck_path, &fonts, &containers) {
            Ok(r) => r,
            Err(f) => {
                eprintln!("{}", f.text());
                return ExitCode::from(1);
            }
        };
        // css-not-emitted do conteúdo dos Containers entra com o do resto do slide.
        checked.notes.append(&mut containers.borrow_mut().notes);
        checked.notes.sort_by_key(|(slide, _)| *slide);
        let mut families: Vec<&str> = Vec::new();
        let mut used_faces: Vec<usize> = Vec::new();
        let mut media = emit::Media::default();
        // Paginação primeiro (o deck de saída pode ter mais slides), depois a numeração.
        let mut output: Vec<model::SlideSpec> = Vec::new();
        // Slide do autor (base 1) de cada slide de saída: as seções apontam para o do autor.
        let mut sources: Vec<usize> = Vec::new();
        // slide-continued vai junto dos diagnósticos da 1ª parte.
        let mut continued: Vec<(usize, diag::Diag)> = Vec::new();
        for (i, slide) in spec.slides.iter().enumerate() {
            match layout::paginate(slide, &fonts) {
                Some(parts) => {
                    let counts: Vec<usize> = parts.iter().map(|(_, n)| *n).collect();
                    let (kind, path) = layout::paginate_target(slide).unwrap_or(("box", String::from("Slide")));
                    continued.push((output.len() + 1, report::slide_continued(output.len() + 1, &path, i + 1, &counts, kind)));
                    sources.extend(std::iter::repeat_n(i + 1, parts.len()));
                    output.extend(parts.into_iter().map(|(p, _)| p));
                }
                None => {
                    sources.push(i + 1);
                    output.push(slide.clone());
                }
            }
        }
        // emit/sections.ts: seção começa na 1ª parte do slide `from`; antes da 1ª, a seção padrão.
        let mut sections: Vec<(String, usize)> = spec
            .sections
            .iter()
            .map(|(title, from)| (title.clone(), sources.iter().position(|s| s >= from).unwrap_or(sources.len())))
            .collect();
        if sections.first().is_some_and(|(_, start)| *start > 0) {
            sections.insert(0, ("Seção padrão".into(), 0));
        }
        // css-not-emitted em cada slide de saída do slide do autor, agrupado por mensagem (fit/css.ts).
        let css: Vec<diag::Diag> = sources
            .iter()
            .enumerate()
            .flat_map(|(i, src)| checked.notes.iter().filter(move |(s, _)| s == src).map(move |(_, n)| validate::note_diag(i + 1, n)))
            .collect();
        for d in report::group_by_message(css) {
            diags.add(d);
        }
        let total = output.len();
        for (i, s) in output.iter_mut().enumerate() {
            model::number_slide(&mut s.root, i + 1, total);
        }
        let mut slides = Vec::new();
        let mut layouts: Vec<(std::rc::Rc<model::LayoutSpec>, Option<emit::LayoutPart>)> = Vec::new();
        let mut svgs: Vec<String> = Vec::new();
        let mut state_slides: Vec<String> = Vec::new();
        for (i, slide) in output.iter().enumerate() {
            let own = Diags::default();
            if let Some((_, d)) = continued.iter().find(|(at, _)| *at == i + 1) {
                own.add(d.clone());
            }
            let laid = layout::layout_slide(slide, &fonts, i + 1, &own);
            // fit/deck.ts withSources: partes de paginação e slides deslocados dizem o slide do autor.
            let source = sources[i];
            let paginated = sources.iter().filter(|&&s| s == source).count() > 1;
            for mut d in own.into_vec() {
                if paginated || source != i + 1 {
                    d.set("sourceSlide", diag::Data::Num(source as f64));
                }
                diags.add(d);
            }
            if args.debug {
                println!("── slide {} ({})", i + 1, ctx.name);
                for p in laid.placed.iter() {
                    let depth = p.node.label.matches(" > ").count();
                    let text = p.text.as_ref().map(|t| {
                        let first: String = t.laid.paragraphs.iter().flatten().next().map(|l| l.iter().map(|x| x.text.as_str()).collect()).unwrap_or_default();
                        format!(" {:.1}pt \"{}\"", t.size, first.chars().take(40).collect::<String>())
                    }).unwrap_or_default();
                    println!("{}{} [{:.1}, {:.1}, {:.1} × {:.1}]{text}", "  ".repeat(depth), p.node.label, p.frame.x, p.frame.y, p.frame.w, p.frame.h);
                }
            }
            for p in &laid.placed {
                if let model::NodeKind::Text(t) = &p.node.kind {
                    if !fonts.has_family(&t.family) && !families.contains(&t.family.as_str()) {
                        families.push(&t.family);
                    }
                    // M8: faces do autor que algum trecho usa (render.ts usedFacesOf).
                    for run in layout::runs_of(t, &fonts, 1.0) {
                        if fonts.author_face(run.face).is_some() && !used_faces.contains(&run.face) {
                            used_faces.push(run.face);
                        }
                    }
                }
            }
            if args.command == "build" && (args.preview || args.state_out.is_some()) {
                let svg = preview::slide_svg(slide, &laid, &fonts, spec.width, spec.height, &lang);
                if let Some(dir) = &args.state_out {
                    let size_dir = dir.join(&ctx.name);
                    let _ = std::fs::create_dir_all(&size_dir);
                    let _ = std::fs::write(size_dir.join(format!("{}.svg", i + 1)), &svg);
                    let nodes: Vec<String> = laid
                        .placed
                        .iter()
                        .map(|p| format!("{{\"path\":{},\"frame\":{{\"x\":{},\"y\":{},\"width\":{},\"height\":{}}}}}", json_string(&p.node.label), p.frame.x, p.frame.y, p.frame.w, p.frame.h))
                        .collect();
                    state_slides.push(format!("{{\"number\":{},\"sourceSlide\":{},\"nodes\":[{}]}}", i + 1, source, nodes.join(",")));
                }
                if args.preview {
                    svgs.push(svg);
                }
            }
            // M15: cada defineLayout vira um slideLayout; o 1º slide dele dá posições e conteúdo fixo.
            let layout_index = slide.layout.as_ref().map(|l| match layouts.iter().position(|(spec, _): &(std::rc::Rc<model::LayoutSpec>, _)| spec.name == l.name) {
                Some(i) => (i, false),
                None => {
                    layouts.push((l.clone(), None));
                    (layouts.len() - 1, true)
                }
            });
            let first = layout_index.is_some_and(|(_, first)| first);
            let mut xml = emit::slide_xml(slide, &laid, &mut media, &lang, &fonts, first);
            if let (Some((li, true)), Some(part)) = (layout_index, xml.layout.take()) {
                let declared = &layouts[li].0.placeholders;
                let missing: Vec<String> = declared.iter().filter(|n| !part.placeholders.iter().any(|p| &p.name == *n)).map(|n| format!("\"{n}\"")).collect();
                if !missing.is_empty() {
                    let names: Vec<String> = declared.iter().filter(|n| !part.placeholders.iter().any(|p| &p.name == *n)).cloned().collect();
                    let many = names.len() > 1;
                    diags.add(
                        diag::Diag::new(Level::Info, "placeholder-unused", i + 1, slide.root.label.clone(), format!(
                            "O layout \"{}\" declara {} mas nenhum nó do 1º slide dele marca {}, então {} no layout do arquivo (as posições vêm do 1º slide). Marque um nó com placeholder=\"…\" ou tire o nome do defineLayout.",
                            layouts[li].0.name,
                            missing.join(", "),
                            if many { "esses placeholders" } else { "esse placeholder" },
                            if many { "eles não saem" } else { "ele não sai" }
                        ))
                        .with("layout", diag::Data::Str(layouts[li].0.name.clone()))
                        .with("placeholders", diag::Data::List(names)),
                    );
                }
                layouts[li].1 = Some(part);
            }
            slides.push(package::SlideOut { slide: xml, notes: slide.notes.clone(), notes_runs: slide.notes_runs.clone(), layout: layout_index.map(|(i, _)| i) });
        }
        for f in families {
            notes.push(format!("fonte \"{f}\" medida com a métrica da Liberation Sans (Arial); quebras podem diferir"));
        }
        // Notas dos módulos (avaliação do deck e funções de Container), sem repetir.
        let container_notes: Vec<String> = std::mem::take(&mut containers.borrow_mut().interp.notes);
        let mut seen_notes: Vec<String> = Vec::new();
        for note in module_notes.iter().cloned().chain(container_notes) {
            if !seen_notes.contains(&note) {
                diags.add(diag::Diag::new(Level::Warn, "module-warning", 0, "", format!("{note} (aviso de um módulo do deck ou do design system, via console.warn)")));
                seen_notes.push(note);
            }
        }
        // M8: EOT das faces usadas; o que não dá para embutir vira warning (font-not-embeddable).
        let mut font_parts: Vec<embed::FontPart> = Vec::new();
        if args.embed_fonts || config.embed_fonts {
            used_faces.sort_unstable();
            for face in used_faces {
                let Some(author) = fonts.author_face(face) else { continue };
                let label = format!("{} {}{}", author.family, author.weight, if author.italic { " italic" } else { "" });
                if author.index > 0 || !embed::is_truetype(author.data) {
                    diags.add(diag::Diag::new(Level::Warn, "font-not-embeddable", 0, "", format!("A fonte \"{label}\" não é um TrueType de uma face só (.otf CFF ou coleção .ttc) e não foi embutida.")));
                    continue;
                }
                match embed::fs_type_of(author.data) {
                    Ok(fs) if embed::is_embeddable(fs) => {}
                    Ok(_) => {
                        diags.add(diag::Diag::new(Level::Warn, "font-not-embeddable", 0, "", format!("A fonte \"{label}\" não permite embed (fsType restrito) e não foi embutida.")));
                        continue;
                    }
                    Err(e) => {
                        diags.add(diag::Diag::new(Level::Warn, "font-not-embeddable", 0, "", format!("A fonte \"{label}\": {e}")));
                        continue;
                    }
                }
                // Peso fora de 400/700 vai sob o nome da própria face ("Inter SemiBold"), como regular: é o typeface dos runs.
                let canonical = author.weight == 400 || author.weight == 700;
                let (family, style) = if canonical { (author.family.clone(), embed::style_of(author.weight == 700, author.italic)) } else { (fonts.face_name(face).to_string(), embed::style_of(false, author.italic)) };
                if font_parts.iter().any(|p| p.family == family && p.style == style) {
                    continue;
                }
                match embed::eot_of(author.data, &family) {
                    Ok(eot) => font_parts.push(embed::FontPart { family, style, eot }),
                    Err(e) => diags.add(diag::Diag::new(Level::Warn, "font-not-embeddable", 0, "", format!("A fonte \"{label}\": {e}"))),
                }
            }
        }
        let list = diags.into_vec();
        failed |= list.iter().any(|d| d.level == Level::Error);
        if args.state_out.is_some() {
            let diagnostics: Vec<(String, diag::Diag)> = list.iter().map(|d| (ctx.name.clone(), d.clone())).collect();
            state_sizes.push(format!(
                "{{\"size\":{},\"width\":{},\"height\":{},\"slides\":[{}],\"diagnostics\":{}}}",
                json_string(&ctx.name),
                spec.width,
                spec.height,
                state_slides.join(","),
                report::json(&diagnostics)
            ));
        }
        let count = slides.len();
        if args.command == "build" {
            let file = args.out.join(format!("{stem}.{}.pptx", ctx.name));
            let theme = spec.theme.clone().or_else(|| config.theme.clone());
            let out = package::DeckOut { title: &title, document: config.document.clone(), width: spec.width, height: spec.height, slides, media, lang: &lang, theme, sections, layouts, fonts: font_parts };
            if let Err(e) = package::write_pptx(&file, &out) {
                eprintln!("\x1b[31merro\x1b[0m ao gravar {}: {e}", file.display());
                return ExitCode::from(1);
            }
            if args.preview && !svgs.is_empty() {
                // <deck>.<size>/slide-NN.png e <deck>.<size>.overview.png, como no pptsx.
                let dir = args.out.join(format!("{stem}.{}", ctx.name));
                let _ = std::fs::create_dir_all(&dir);
                // Um PNG por slide, em paralelo (cada thread rasteriza o próprio SVG).
                std::thread::scope(|scope| {
                    for (i, svg) in svgs.iter().enumerate() {
                        let dir = &dir;
                        let width = args.preview_width;
                        scope.spawn(move || {
                            if let Some(png) = preview::render_png(svg, width) {
                                let _ = std::fs::write(dir.join(format!("slide-{:02}.png", i + 1)), png);
                            }
                        });
                    }
                });
                let marks: Vec<(String, bool, bool)> = svgs
                    .into_iter()
                    .enumerate()
                    .map(|(i, svg)| (svg, list.iter().any(|d| d.level == Level::Error && d.slide == i + 1), output[i].hidden))
                    .collect();
                let overview = preview::overview_svg(&marks, spec.width, spec.height, &fonts);
                if let Some(png) = resvg::usvg::Tree::from_str(&overview, &resvg::usvg::Options::default()).ok().and_then(|t| {
                    let size = t.size();
                    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width().ceil() as u32, size.height().ceil() as u32)?;
                    resvg::render(&t, resvg::tiny_skia::Transform::identity(), &mut pixmap.as_mut());
                    pixmap.encode_png().ok()
                }) {
                    let _ = std::fs::write(args.out.join(format!("{stem}.{}.overview.png", ctx.name)), png);
                }
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            progress(args.json, &format!("\x1b[32m✓\x1b[0m {} ({count} slides, {ms:.1} ms)", file.display()));
        } else {
            progress(args.json, &format!("\x1b[32m✓\x1b[0m {stem} {} ({count} slides)", ctx.name));
        }
        all.extend(list.into_iter().map(|d| (ctx.name.clone(), d)));
    }
    // Avisos do Rust sem código do pptsx (resolve, fontes) vão à parte; o resto é o contrato do pptsx.
    for d in all.iter().filter(|(_, d)| d.code == "note").map(|(_, d)| d) {
        let place = if d.slide == 0 { "deck".to_string() } else { format!("slide {}", d.slide) };
        notes.push(format!("[{place}] {}", d.msg));
    }
    all.retain(|(_, d)| d.code != "note");
    for n in &notes {
        eprintln!("  \x1b[33maviso\x1b[0m {n}");
    }
    if args.command == "build" {
        let _ = std::fs::write(args.out.join(format!("{stem}.diagnostics.json")), report::json(&all));
    }
    if let Some(dir) = &args.state_out {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let built_at = format!("{:02}:{:02}:{:02} UTC", secs / 3600 % 24, secs / 60 % 60, secs % 60);
        let _ = std::fs::write(dir.join("diagnostics.json"), report::json(&all));
        let _ = std::fs::write(dir.join("state.json"), format!("{{\"builtAt\":{},\"sizes\":[{}]}}", json_string(&built_at), state_sizes.join(",")));
    }
    // check --json: os diagnósticos estruturados (o MCP e agentes leem isto).
    if args.json && args.command == "check" {
        println!("{}", report::json(&all));
    } else {
        println!("{}", report::summary(&all));
    }
    let total = started.elapsed().as_secs_f64() * 1000.0;
    progress(args.json, &format!("  avaliação do TSX {eval_ms:.1} ms · total {total:.1} ms"));
    // Como o pptsx: sai 1 com qualquer error (agentes e CI dependem disso).
    if failed {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
