//! `pptsx-rs mcp`: servidor MCP (mcp/server.ts do pptsx) por stdio — JSON-RPC 2.0, uma mensagem
//! por linha. Um agente constrói, lê diagnósticos, vê PNGs, roda o lint e consulta guia e schema
//! sem shell. O stdout é o canal do protocolo: build, check e lint rodam o próprio binário como
//! subprocesso (o que eles imprimem volta como texto da ferramenta), e o resto vai para o stderr.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::js::value::{Obj, arr_val, obj_val, str_val, to_json};
use crate::js::{self, Value};

/// O guia do agente e o schema das props das primitivas, embutidos no binário (os originais ficam em assets/).
pub(crate) const GUIDE: &str = include_str!("../assets/agent-guide.md");
const SCHEMA_SOURCE: &str = include_str!("../assets/schema.generated.ts");
const CORE_SOURCE: &str = "pptsx";
const PROTOCOL: &str = "2025-06-18";
const JSON_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

// ───────────── JSON ─────────────

fn obj(entries: Vec<(&str, Value)>) -> Value {
    let mut o = Obj::new();
    for (k, v) in entries {
        o.insert(Rc::from(k), v);
    }
    obj_val(o)
}

fn json(v: &Value) -> String {
    let mut out = String::new();
    to_json(v, &mut out);
    out
}

/// JSON com recuo de 2 (como o JSON.stringify(x, null, 2) das respostas do pptsx).
fn pretty(v: &Value, depth: usize, out: &mut String) {
    let pad = |d: usize| "  ".repeat(d);
    match v {
        Value::Array(a) if !a.borrow().is_empty() => {
            out.push_str("[\n");
            let items = a.borrow();
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                pretty(item, depth + 1, out);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push(']');
        }
        Value::Object(o) if o.iter().any(|(_, v)| !matches!(v, Value::Undefined | Value::Func(_))) => {
            out.push_str("{\n");
            let entries: Vec<_> = o.iter().filter(|(_, v)| !matches!(v, Value::Undefined | Value::Func(_))).collect();
            for (i, (k, item)) in entries.iter().enumerate() {
                out.push_str(&pad(depth + 1));
                to_json(&str_val(k), out);
                out.push_str(": ");
                pretty(item, depth + 1, out);
                out.push_str(if i + 1 < entries.len() { ",\n" } else { "\n" });
            }
            out.push_str(&pad(depth));
            out.push('}');
        }
        other => to_json(other, out),
    }
}

pub fn pretty_json(v: &Value) -> String {
    let mut out = String::new();
    pretty(v, 0, &mut out);
    out
}

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(o) => o.get(key).filter(|v| !v.is_nullish()),
        _ => None,
    }
}

fn get_str(v: &Value, key: &str) -> Option<String> {
    get(v, key).map(Value::to_js_string)
}

fn get_list(v: &Value, key: &str) -> Vec<String> {
    match get(v, key) {
        Some(Value::Array(a)) => a.borrow().iter().map(Value::to_js_string).collect(),
        Some(Value::Str(s)) => s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        _ => Vec::new(),
    }
}

// ───────────── resultado das ferramentas ─────────────

struct ToolResult {
    content: Vec<Value>,
    is_error: bool,
}

fn text(t: &str) -> Value {
    obj(vec![("type", str_val("text")), ("text", str_val(t))])
}

fn png(data: &[u8]) -> Value {
    obj(vec![("type", str_val("image")), ("data", str_val(&crate::preview::base64(data))), ("mimeType", str_val("image/png"))])
}

fn ok(content: Vec<Value>) -> ToolResult {
    ToolResult { content, is_error: false }
}

fn failure(message: &str) -> ToolResult {
    ToolResult { content: vec![text(&format!("Erro: {message}"))], is_error: true }
}

// ───────────── subprocesso ─────────────

pub fn strip_ansi(s: &str) -> String {
    static ANSI_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\x1b\[[0-9;]*m").expect("regex fixa"));
    ANSI_RE.replace_all(s, "").into_owned()
}

/// Roda `pptsx-rs <args>`: (código de saída, stdout + stderr sem cores).
fn run_self(args: &[String], cwd: &Path) -> Result<(i32, String), String> {
    let exe = std::env::current_exe().map_err(|e| format!("não achei o próprio binário: {e}"))?;
    let out = Command::new(exe).args(args).current_dir(cwd).output().map_err(|e| format!("falha ao rodar o build: {e}"))?;
    let mut log = String::from_utf8_lossy(&out.stdout).into_owned();
    let err = String::from_utf8_lossy(&out.stderr);
    if !err.trim().is_empty() {
        if !log.is_empty() && !log.ends_with('\n') {
            log.push('\n');
        }
        log.push_str(&err);
    }
    Ok((out.status.code().unwrap_or(1), strip_ansi(log.trim_end())))
}

fn resolve(cwd: &Path, p: &str) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() { path } else { cwd.join(path) }
}

fn deck_of(input: &Value, cwd: &Path) -> Result<PathBuf, String> {
    let deck = get_str(input, "deck").ok_or("falta `deck` (caminho do .tsx)")?;
    let path = resolve(cwd, &deck);
    if !path.is_file() {
        return Err(format!("deck não encontrado: {}", path.display()));
    }
    Ok(path)
}

fn size_args(input: &Value, key: &str) -> Vec<String> {
    let sizes = get_list(input, key);
    if sizes.is_empty() { Vec::new() } else { vec!["--size".into(), sizes.join(",")] }
}

fn stem_of(deck: &Path) -> String {
    deck.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "deck".into())
}

/// Os .pptx e PNGs que o build gravou para este deck.
fn outputs_of(out: &Path, stem: &str) -> Vec<PathBuf> {
    let prefix = format!("{stem}.");
    let mut files: Vec<PathBuf> = std::fs::read_dir(out)
        .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&prefix) && (n.ends_with(".pptx") || n.ends_with(".overview.png")))).collect())
        .unwrap_or_default();
    files.sort();
    files
}

// ───────────── ferramentas ─────────────

/// build: grava .pptx e PNGs como a CLI; devolve o relatório e o overview de cada tamanho.
fn build_tool(input: &Value, cwd: &Path) -> Result<ToolResult, String> {
    let deck = deck_of(input, cwd)?;
    // Padrão ao lado do deck: dois deck.tsx em pastas diferentes não se sobrescrevem.
    let out = match get_str(input, "out") {
        Some(o) => resolve(cwd, &o),
        None => deck.parent().unwrap_or(cwd).join("dist"),
    };
    let mut args = vec!["build".to_string(), deck.display().to_string(), "--out".into(), out.display().to_string(), "--strict".into()];
    args.extend(size_args(input, "sizes"));
    let (code, log) = run_self(&args, cwd)?;
    let files = outputs_of(&out, &stem_of(&deck));
    let listing: Vec<String> = files.iter().map(|f| format!("  {}", f.display())).collect();
    // Como o pptsx: o resumo e o JSON com os arquivos e os diagnósticos.
    let diagnostics = std::fs::read_to_string(out.join(format!("{}.diagnostics.json", stem_of(&deck)))).unwrap_or_else(|_| "[]".into());
    let files_json: Vec<String> = files.iter().map(|f| format!("{:?}", f.display().to_string())).collect();
    let report = format!("{{\n  \"files\": [{}],\n  \"diagnostics\": {}\n}}", files_json.join(", "), diagnostics);
    let mut content = vec![text(&format!("{log}\n\nArquivos:\n{}\n\n{report}", listing.join("\n")))];
    for f in files.iter().filter(|f| f.to_string_lossy().ends_with(".overview.png")) {
        if let Ok(data) = std::fs::read(f) {
            content.push(text(&format!("Overview: {}", f.display())));
            content.push(png(&data));
        }
    }
    Ok(ToolResult { content, is_error: code != 0 })
}

/// diagnose: só os diagnósticos (sem PNG, sem gravar nada): o caminho rápido para iterar.
fn diagnose_tool(input: &Value, cwd: &Path) -> Result<ToolResult, String> {
    let deck = deck_of(input, cwd)?;
    let mut args = vec!["check".to_string(), deck.display().to_string(), "--strict".into()];
    args.extend(size_args(input, "sizes"));
    let (code, log) = run_self(&args, cwd)?;
    let mut json_args = args.clone();
    json_args.push("--json".into());
    let (_, report) = run_self(&json_args, cwd)?;
    // stdout é só o JSON (o progresso vai ao stderr, que vem depois): do primeiro '[' ao último ']'.
    let json = match (report.find('['), report.rfind(']')) {
        (Some(a), Some(b)) if b > a => &report[a..=b],
        _ => "[]",
    };
    Ok(ToolResult { content: vec![text(&format!("{log}\n\n{{\n  \"diagnostics\": {json}\n}}"))], is_error: code != 0 })
}

/// preview: o PNG de um slide (número do deck de saída, base 1) num tamanho.
fn preview_tool(input: &Value, cwd: &Path) -> Result<ToolResult, String> {
    let deck = deck_of(input, cwd)?;
    let size = get_str(input, "size").unwrap_or_else(|| "16x9".into());
    let slide = get(input, "slide").map_or(1.0, Value::to_num).max(1.0) as usize;
    let tmp = std::env::temp_dir().join(format!("pptsx-rs-mcp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let args = vec!["build".to_string(), deck.display().to_string(), "--out".into(), tmp.display().to_string(), "--size".into(), size.clone()];
    let (code, log) = run_self(&args, cwd)?;
    let stem = stem_of(&deck);
    // O nome do tamanho na pasta é o do contexto (custom-10x7.5 para "10x7.5in").
    let dir = std::fs::read_dir(&tmp).ok().and_then(|d| d.flatten().map(|e| e.path()).find(|p| p.is_dir() && p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&format!("{stem}.")))));
    let Some(dir) = dir else {
        let _ = std::fs::remove_dir_all(&tmp);
        return Ok(ToolResult { content: vec![text(&format!("O build não gerou PNG (saída {code}).\n{log}"))], is_error: true });
    };
    let total = std::fs::read_dir(&dir).map(|d| d.flatten().filter(|e| e.file_name().to_string_lossy().starts_with("slide-")).count()).unwrap_or(0);
    let image = std::fs::read(dir.join(format!("slide-{slide:02}.png")));
    let _ = std::fs::remove_dir_all(&tmp);
    match image {
        Ok(data) => {
            // Só os diagnósticos deste slide (as linhas "[slide N]").
            let own: Vec<&str> = log.lines().filter(|l| l.contains(&format!("[slide {slide}]"))).collect();
            let summary = if own.is_empty() { "Sem diagnósticos neste slide.".to_string() } else { own.join("\n") };
            Ok(ok(vec![text(&format!("Slide {slide} de {total} ({size}).\n{summary}")), png(&data)]))
        }
        Err(_) => Ok(ToolResult { content: vec![text(&format!("O deck tem {total} slides em {size}; peça de 1 a {total}."))], is_error: true }),
    }
}

fn lint_tool(input: &Value, cwd: &Path) -> Result<ToolResult, String> {
    let deck = deck_of(input, cwd)?;
    let mut args = vec!["lint".to_string(), deck.display().to_string()];
    args.extend(size_args(input, "sizes"));
    // Achado do DS (`origin: "ds"`) só entra a pedido: o agente que escreve o deck não o resolve.
    let verbose = matches!(get(input, "verbose"), Some(Value::Bool(true)));
    if verbose {
        args.push("--verbose".into());
    }
    let (code, summary) = run_self(&args, cwd)?;
    let mut json_args = args.clone();
    json_args.push("--json".into());
    if !verbose {
        json_args.push("--deck-only".into());
    }
    let (_, report) = run_self(&json_args, cwd)?;
    let ds = ds_of_deck(&deck).iter().map(|d| d.name.clone()).collect::<Vec<_>>();
    let ds_line = if ds.is_empty() { String::new() } else { format!("\n\nDS: {}", ds.join(", ")) };
    Ok(ToolResult { content: vec![text(&format!("{summary}{ds_line}\n\n{report}"))], is_error: code != 0 })
}

// ───────────── design systems ─────────────

struct Ds {
    name: String,
    version: String,
    description: Option<String>,
    guide: Option<String>,
    schema: Option<Value>,
    components: Vec<crate::dsinfo::DsComponent>,
}

/// O DS de um manifesto. O schema é o do autor quando ele passou um ao `createDs`; senão, o lido dos
/// tipos das props nos módulos que o avaliador carregou (dsprops.rs), com os exemplos do catálogo.
fn manifest_of(interp: &js::Interp, v: &Value) -> Option<Ds> {
    let name = get_str(v, "name")?;
    let guide = get(v, "guide").map(Value::to_js_string);
    let schema = crate::dsprops::schema_of(interp, v);
    let version = get_str(v, "version").unwrap_or_default();
    let description = get_str(v, "description");
    Some(Ds { name, version, description, guide, schema, components: crate::dsinfo::components_of(v) })
}

fn ds_of_config(path: &Path) -> Vec<Ds> {
    let Ok((interp, cfg)) = js::evaluate_module(path) else { return Vec::new() };
    match get(&cfg, "ds") {
        Some(Value::Array(list)) => list.borrow().iter().filter_map(|v| manifest_of(&interp, v)).collect(),
        _ => Vec::new(),
    }
}

/// `--ds <caminho|pasta|pacote>`: o manifesto que o módulo exporta (`ds` nomeado ou default).
fn ds_of_arg(spec: &str, cwd: &Path) -> Result<Ds, String> {
    let (file, _) = crate::dsinfo::entry_of(spec, cwd)?;
    let (interp, exports) = js::evaluate_exports(&file)?;
    let (manifest, _) = crate::dsinfo::manifest_of(&exports).ok_or("o módulo precisa exportar \"ds\" (ou default) um defineDs/createDs")?;
    manifest_of(&interp, &manifest).ok_or_else(|| "manifesto sem nome".to_string())
}

/// DS da config da pasta do deck (ou acima), como o build e o lint.
fn ds_of_deck(deck: &Path) -> Vec<Ds> {
    find_config(deck.parent().unwrap_or(Path::new("."))).map(|c| ds_of_config(&c)).unwrap_or_default()
}

fn find_config(dir: &Path) -> Option<PathBuf> {
    let mut dir = std::fs::canonicalize(dir).ok()?;
    loop {
        let candidate = dir.join(crate::config::CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

struct Server {
    cwd: PathBuf,
    /// `--ds <ds.ts>` e os da config do cwd do servidor.
    ds: Vec<Ds>,
}

/// Os DS do servidor mais os da config do deck, quando a chamada traz um deck (o primeiro nome vence).
fn merged_ds<'a>(server: &'a [Ds], extra: &'a [Ds]) -> Vec<&'a Ds> {
    let mut out: Vec<&Ds> = Vec::new();
    for d in server.iter().chain(extra) {
        if !out.iter().any(|o| o.name == d.name) {
            out.push(d);
        }
    }
    out
}

// ───────────── guia ─────────────

struct Section<'a> {
    heading: &'a str,
    level: usize,
    body: String,
}

/// Seção = um `## ` ou `### ` até o próximo cabeçalho do mesmo nível ou acima (mcp/guide.ts).
fn sections_of(markdown: &str) -> Vec<Section<'_>> {
    let lines: Vec<&str> = markdown.split('\n').collect();
    let starts: Vec<(usize, usize, &str)> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let level = if l.starts_with("### ") { 3 } else if l.starts_with("## ") { 2 } else { return None };
            Some((i, level, l[level + 1..].trim()))
        })
        .collect();
    starts
        .iter()
        .map(|&(index, level, heading)| {
            let end = starts.iter().find(|(i, l, _)| *i > index && *l <= level).map_or(lines.len(), |(i, _, _)| *i);
            Section { heading, level, body: lines[index..end].join("\n").trim_end().to_string() }
        })
        .collect()
}

fn section_matches(query: &str, heading: &str) -> bool {
    let q = query.trim();
    let number = q.trim_start_matches('§').trim().trim_end_matches('.');
    if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return heading.starts_with(&format!("{number}.")) || heading.starts_with(&format!("{number} "));
    }
    heading.to_lowercase().contains(&q.to_lowercase())
}

fn guide_lookup(markdown: &str, query: Option<&str>, title: &str) -> (String, bool) {
    let sections = sections_of(markdown);
    match query.filter(|q| !q.trim().is_empty()) {
        None => {
            let index: Vec<String> = sections.iter().map(|s| format!("{}- {}", if s.level == 2 { "" } else { "  " }, s.heading)).collect();
            (format!("Seções do {title} (peça uma com section: \"§8\", \"6.17\" ou um trecho do título):\n{}", index.join("\n")), true)
        }
        Some(q) => match sections.iter().find(|s| section_matches(q, s.heading)) {
            Some(s) => (s.body.clone(), true),
            None => (format!("Nenhuma seção do guia casa com \"{q}\". Chame guide() sem argumento para ver o índice."), false),
        },
    }
}

fn unknown_source(source: &str, ds: &[&Ds]) -> ToolResult {
    let known: String = ds.iter().map(|d| format!(", \"{}\"", d.name)).collect();
    ToolResult { content: vec![text(&format!("Fonte \"{source}\" não registrada. Fontes: \"{CORE_SOURCE}\"{known}. Um DS entra pelo pptsx.config.ts (ds: [...]) ou por pptsx-rs mcp --ds <caminho|pacote>."))], is_error: true }
}

fn sources_line(ds: &[&Ds], has: impl Fn(&Ds) -> bool) -> String {
    let others: Vec<String> = ds.iter().filter(|d| has(d)).map(|d| format!("\"{}\"", d.name)).collect();
    if others.is_empty() { String::new() } else { format!("\n\nOutras fontes (passe source): {}.", others.join(", ")) }
}

fn guide_tool(input: &Value, ds: &[&Ds]) -> ToolResult {
    let source = get_str(input, "source").unwrap_or_else(|| CORE_SOURCE.into());
    let section = get_str(input, "section");
    if source != CORE_SOURCE {
        let Some(d) = ds.iter().find(|d| d.name == source) else { return unknown_source(&source, ds) };
        let Some(guide) = &d.guide else { return ToolResult { content: vec![text(&format!("O DS \"{}\" não registrou guia.", d.name))], is_error: true } };
        let (body, found) = guide_lookup(guide, section.as_deref(), &format!("guia do DS \"{}\"", d.name));
        return ToolResult { content: vec![text(&body)], is_error: !found };
    }
    let (body, found) = guide_lookup(GUIDE, section.as_deref(), "guia do pptsx");
    let extra = if section.is_none() { sources_line(ds, |d| d.guide.is_some()) } else { String::new() };
    ToolResult { content: vec![text(&format!("{body}{extra}"))], is_error: !found }
}

// ───────────── catálogo ─────────────

fn catalog_tool(input: &Value, ds: &[&Ds]) -> ToolResult {
    if ds.is_empty() {
        return ToolResult { content: vec![text("Nenhum design system registrado. Um DS entra pelo pptsx.config.ts (ds: [...]), pelo pptsx.registry.json (pptsx-rs registry add) ou por pptsx-rs mcp --ds <caminho|pacote>.")], is_error: true };
    }
    let source = get_str(input, "source");
    let chosen: Vec<&&Ds> = match &source {
        Some(name) => match ds.iter().find(|d| &d.name == name) {
            Some(d) => vec![d],
            None => return unknown_source(name, ds),
        },
        None => ds.iter().collect(),
    };
    let audience = get_str(input, "audience");
    let kind = get_str(input, "kind");
    let parts: Vec<String> = chosen
        .iter()
        .map(|d| {
            let index = crate::catalog::IndexDs { name: &d.name, version: &d.version, description: d.description.as_deref(), components: &d.components, schema: d.schema.as_ref() };
            crate::catalog::index_markdown(&index, audience.as_deref(), kind.as_deref())
        })
        .collect();
    ok(vec![text(&format!("{}\nProps completas: schema(source, primitive=<Componente>). Guia: guide(source).", parts.join("\n---\n\n")))])
}

// ───────────── schema ─────────────

pub(crate) fn core_schema() -> Option<Value> {
    // O schema gerado é um objeto JSON literal depois de `PROPS_SCHEMA ... =`.
    let at = SCHEMA_SOURCE.find("PROPS_SCHEMA")?;
    let start = at + SCHEMA_SOURCE[at..].find("= {")? + 2;
    let end = SCHEMA_SOURCE.rfind('}')? + 1;
    js::parse_json(&SCHEMA_SOURCE[start..end])
}

/// Um tipo TypeScript (texto) → o JSON Schema que dá para dizer: enum de literais, tipos simples, ou só `x-tsType`.
fn prop_to_json_schema(raw: &Value) -> Value {
    let ts = get_str(raw, "type").unwrap_or_default().trim().to_string();
    let parts: Vec<&str> = ts.split('|').map(str::trim).filter(|p| !p.is_empty()).collect();
    // Literal de string com aspas duplas (schema do núcleo) ou simples (tipos lidos de um DS).
    let literal = |p: &str| ['"', '\''].iter().find_map(|q| p.strip_prefix(*q).and_then(|r| r.strip_suffix(*q)).filter(|r| !r.contains(*q))).map(str::to_string);
    let simple = |p: &str| matches!(p, "string" | "number" | "boolean" | "array" | "object" | "null");
    let mut out: Vec<(&str, Value)> = Vec::new();
    if !parts.is_empty() && parts.iter().all(|p| literal(p).is_some()) {
        out.push(("type", str_val("string")));
        out.push(("enum", arr_val(parts.iter().filter_map(|p| literal(p)).map(|l| str_val(&l)).collect())));
    } else if !parts.is_empty() && parts.iter().all(|p| simple(p)) {
        out.push(("type", if parts.len() == 1 { str_val(parts[0]) } else { arr_val(parts.iter().map(|p| str_val(p)).collect()) }));
    }
    if let Some(d) = get(raw, "description") {
        out.push(("description", d.clone()));
    }
    if let Some(d) = get(raw, "default") {
        out.push(("x-default", d.clone()));
    }
    if let Some(Value::Array(e)) = get(raw, "examples")
        && !e.borrow().is_empty() {
            out.push(("examples", Value::Array(e.clone())));
        }
    if !ts.is_empty() {
        out.push(("x-tsType", str_val(&ts)));
    }
    obj(out)
}

fn to_json_schema(source: &str, raw: &Value) -> (Value, Vec<(String, Value)>) {
    let mut defs: Vec<(String, Value)> = Vec::new();
    if let Value::Object(o) = raw {
        for (name, def) in o.iter() {
            let mut props = Obj::new();
            if let Some(Value::Object(p)) = get(def, "properties") {
                for (k, v) in p.iter() {
                    props.insert(k.clone(), prop_to_json_schema(v));
                }
            }
            let required = match get(def, "required") {
                Some(Value::Array(r)) => Value::Array(r.clone()),
                _ => arr_val(Vec::new()),
            };
            let mut fields = vec![("type", str_val("object")), ("properties", obj_val(props)), ("required", required)];
            // Do schema lido dos tipos: de onde veio e o que as props herdam sem o binário abrir.
            fields.extend(["x-extends", "x-source"].iter().filter_map(|key| get(def, key).map(|v| (*key, v.clone()))));
            defs.push((name.to_string(), obj(fields)));
        }
    }
    let mut d = Obj::new();
    for (k, v) in &defs {
        d.insert(Rc::from(k.as_str()), v.clone());
    }
    let document = obj(vec![("$schema", str_val(JSON_SCHEMA_DIALECT)), ("$id", str_val(&format!("pptsx:schema/{source}"))), ("title", str_val(&format!("Props de {source}"))), ("$defs", obj_val(d))]);
    (document, defs)
}

fn lookup_schema(source: &str, raw: &Value, name: Option<&str>, label: &str, extra: &str) -> ToolResult {
    let (document, defs) = to_json_schema(source, raw);
    let Some(name) = name else { return ok(vec![text(&format!("{}{extra}", pretty_json(&document)))]) };
    // "Text", "text" e "TEXT" valem: o agente costuma escrever como a tag JSX.
    let Some((key, def)) = defs.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)) else {
        let known: Vec<&str> = defs.iter().map(|(k, _)| k.as_str()).collect();
        return ToolResult { content: vec![text(&format!("\"{name}\" não existe em {label}; use {}.", known.join(", ")))], is_error: true };
    };
    let mut o = Obj::new();
    o.insert(Rc::from("$schema"), str_val(JSON_SCHEMA_DIALECT));
    o.insert(Rc::from("$id"), str_val(&format!("pptsx:schema/{source}#/$defs/{key}")));
    if let Value::Object(fields) = def {
        for (k, v) in fields.iter() {
            o.insert(k.clone(), v.clone());
        }
    }
    ok(vec![text(&pretty_json(&obj_val(o)))])
}

fn schema_tool(input: &Value, ds: &[&Ds]) -> ToolResult {
    let source = get_str(input, "source").unwrap_or_else(|| CORE_SOURCE.into());
    let primitive = get_str(input, "primitive");
    if source != CORE_SOURCE {
        let Some(d) = ds.iter().find(|d| d.name == source) else { return unknown_source(&source, ds) };
        let Some(schema) = &d.schema else { return ToolResult { content: vec![text(&format!("O DS \"{}\" não registrou schema.", d.name))], is_error: true } };
        return lookup_schema(&d.name, schema, primitive.as_deref(), &d.name, "");
    }
    let Some(schema) = core_schema() else { return failure("schema do pptsx ilegível no binário") };
    let extra = if primitive.is_none() { sources_line(ds, |d| d.schema.is_some()) } else { String::new() };
    lookup_schema(CORE_SOURCE, &schema, primitive.as_deref(), "pptsx (primitivas)", &extra)
}

// ───────────── registro ─────────────

fn prop_schema(ty: &str, description: &str) -> Value {
    obj(vec![("type", str_val(ty)), ("description", str_val(description))])
}

fn sizes_schema() -> Value {
    obj(vec![("type", str_val("array")), ("items", obj(vec![("type", str_val("string"))])), ("description", str_val("Tamanhos: \"16x9\", \"4x3\", \"16x10\", \"wide\" ou \"10x7.5in\". Padrão: os da config, senão 16x9."))])
}

fn deck_schema() -> Value {
    prop_schema("string", "Caminho do deck .tsx (relativo ao diretório do servidor ou absoluto).")
}

fn input_schema(props: Vec<(&str, Value)>, required: &[&str]) -> Value {
    obj(vec![("type", str_val("object")), ("properties", obj(props)), ("required", arr_val(required.iter().map(|r| str_val(r)).collect()))])
}

fn tools() -> Value {
    let tool = |name: &str, title: &str, description: &str, schema: Value| obj(vec![("name", str_val(name)), ("title", str_val(title)), ("description", str_val(description)), ("inputSchema", schema)]);
    let source = || prop_schema("string", "\"pptsx\" (padrão) ou o nome de um DS registrado.");
    let deck_opt = || prop_schema("string", "Um deck: os DS da config da pasta dele (ou acima) viram fontes.");
    arr_val(vec![
        tool(
            "build",
            "Construir o deck",
            "Gera .pptx e PNGs (como `pptsx-rs build`) e devolve os diagnósticos e o overview de cada tamanho. isError = o deck tem diagnóstico de severidade error (a ferramenta funcionou; corrija o deck).",
            input_schema(vec![("deck", deck_schema()), ("sizes", sizes_schema()), ("out", prop_schema("string", "Pasta de saída. Padrão: dist/ na pasta do deck. Os arquivos saem como <nome-do-deck>.<tamanho>.pptx, .overview.png e um PNG por slide."))], &["deck"]),
        ),
        tool("diagnose", "Diagnosticar o deck", "Só os diagnósticos (sem PNG, sem gravar arquivos): o caminho rápido para iterar. isError = o deck tem diagnóstico de severidade error.", input_schema(vec![("deck", deck_schema()), ("sizes", sizes_schema())], &["deck"])),
        tool(
            "preview",
            "PNG de um slide",
            "O PNG de um slide (número do deck de saída, base 1, depois da paginação) num tamanho, com os diagnósticos dele.",
            input_schema(vec![("deck", deck_schema()), ("size", prop_schema("string", "Um tamanho (\"16x9\").")), ("slide", obj(vec![("type", str_val("integer")), ("minimum", Value::Num(1.0))]))], &["deck", "size", "slide"]),
        ),
        tool(
            "guide",
            "Guia do agente",
            "Trechos do guia. Fonte padrão \"pptsx\" (o guia do binário); um DS registrado é outra fonte (`source` = nome do DS). Sem `section`: o índice (e as fontes). Com \"§8\", \"6.17\" ou parte do título: a seção.",
            input_schema(vec![("source", source()), ("section", obj(vec![("type", str_val("string"))])), ("deck", deck_opt())], &[]),
        ),
        tool(
            "schema",
            "Props das primitivas",
            "Props aceitas por primitiva, em JSON Schema 2020-12 (`$id` por fonte; o tipo TypeScript em `x-tsType`); Com `source` = nome de um DS: as props dos componentes dele. Sem `primitive`: todas.",
            input_schema(vec![("source", source()), ("primitive", prop_schema("string", "\"box\", \"text\", \"chart\"… ou o nome de um componente do DS")), ("deck", deck_opt())], &[]),
        ),
        tool(
            "catalog",
            "Catálogo dos design systems",
            "O índice dos componentes dos DS registrados, por camada (template, structure, chart, component, widget): assinatura, para que serve, público, quando usar, o que evitar e o JSX de cada caso do catálogo. Use antes de escrever um deck com um DS: comece pelo template, preencha com estruturas, gráficos e componentes.",
            input_schema(
                vec![
                    ("source", prop_schema("string", "O nome de um DS registrado. Sem ele: todos.")),
                    ("audience", prop_schema("string", "Só o que serve a este público: \"executive\", \"strategic\", \"tactical\", \"operational\".")),
                    ("kind", prop_schema("string", "Só esta camada: \"template\", \"structure\", \"chart\", \"component\" ou \"widget\".")),
                    ("deck", deck_opt()),
                ],
                &[],
            ),
        ),
        tool(
            "lint",
            "Lint do deck",
            "As regras da engine (props antigas, lineHeight alto, fr solto, cores fora do tema, estrutura) e as dos design systems da config da pasta do deck (por nó e do deck inteiro). isError = há achado error ou warning. Achados que são do design system e não do deck (`\"origin\": \"ds\"`) só saem com `verbose`.",
            input_schema(vec![("deck", deck_schema()), ("sizes", sizes_schema()), ("verbose", prop_schema("boolean", "Inclui os achados do design system (cores dos tokens fora do tema, nós de dentro dos componentes). Padrão: false."))], &["deck"]),
        ),
    ])
}

impl Server {
    fn call(&self, name: &str, input: &Value) -> ToolResult {
        let deck_ds: Vec<Ds> = get_str(input, "deck").map(|d| ds_of_deck(&resolve(&self.cwd, &d))).unwrap_or_default();
        let ds = merged_ds(&self.ds, &deck_ds);
        let result = match name {
            "build" => build_tool(input, &self.cwd),
            "diagnose" => diagnose_tool(input, &self.cwd),
            "preview" => preview_tool(input, &self.cwd),
            "lint" => lint_tool(input, &self.cwd),
            "guide" => Ok(guide_tool(input, &ds)),
            "schema" => Ok(schema_tool(input, &ds)),
            "catalog" => Ok(catalog_tool(input, &ds)),
            other => Err(format!("ferramenta desconhecida: {other}")),
        };
        result.unwrap_or_else(|e| failure(&e))
    }

    /// Uma mensagem JSON-RPC; None para notificações (sem `id`).
    fn handle(&self, message: &Value) -> Option<Value> {
        let id = get(message, "id").cloned();
        let method = get_str(message, "method").unwrap_or_default();
        let params = get(message, "params").cloned().unwrap_or(Value::Undefined);
        let reply = |result: Value| obj(vec![("jsonrpc", str_val("2.0")), ("id", id.clone().unwrap_or(Value::Null)), ("result", result)]);
        let error = |code: f64, msg: &str| obj(vec![("jsonrpc", str_val("2.0")), ("id", id.clone().unwrap_or(Value::Null)), ("error", obj(vec![("code", Value::Num(code)), ("message", str_val(msg))]))]);
        id.as_ref()?;
        Some(match method.as_str() {
            "initialize" => {
                let version = get_str(&params, "protocolVersion").unwrap_or_else(|| PROTOCOL.into());
                reply(obj(vec![
                    ("protocolVersion", str_val(&version)),
                    ("capabilities", obj(vec![("tools", obj(vec![]))])),
                    ("serverInfo", obj(vec![("name", str_val("pptsx-rs")), ("version", str_val(env!("CARGO_PKG_VERSION")))])),
                    ("instructions", str_val("pptsx em Rust: decks TSX → PPTX. Comece por guide() e schema() (com um design system, catalog()); itere com diagnose e preview; build grava os arquivos.")),
                ]))
            }
            "ping" => reply(obj(vec![])),
            "tools/list" => reply(obj(vec![("tools", tools())])),
            "tools/call" => {
                let name = get_str(&params, "name").unwrap_or_default();
                let input = get(&params, "arguments").cloned().unwrap_or_else(|| obj(vec![]));
                let r = self.call(&name, &input);
                reply(obj(vec![("content", arr_val(r.content)), ("isError", Value::Bool(r.is_error))]))
            }
            other => error(-32601.0, &format!("método desconhecido: {other}")),
        })
    }
}

pub fn serve(ds_args: &[String]) -> std::process::ExitCode {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut ds: Vec<Ds> = find_config(&cwd).map(|c| ds_of_config(&c)).unwrap_or_default();
    // Os DS do pptsx.registry.json (desta pasta ou acima) também viram fontes, sem --ds.
    if let Some(file) = crate::registry::find(&cwd)
        && let Ok(registry) = crate::registry::load(&file)
    {
        for source in crate::registry::sources_of(&file, &registry) {
            match ds_of_arg(&source.spec, &source.base) {
                Ok(m) if !ds.iter().any(|d| d.name == m.name) => ds.push(m),
                Ok(_) => {}
                Err(e) => eprintln!("{}: {}: {e}", file.display(), source.spec),
            }
        }
    }
    for spec in ds_args {
        match ds_of_arg(spec, &cwd) {
            Ok(m) if !ds.iter().any(|d| d.name == m.name) => ds.push(m),
            Ok(_) => {}
            Err(e) => eprintln!("--ds {spec}: {e}"),
        }
    }
    let server = Server { cwd, ds };
    eprintln!("pptsx-rs mcp: pronto (stdio)");
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let response = match js::parse_json(&line) {
            // Lote (array) também vale no JSON-RPC.
            Some(Value::Array(batch)) => {
                let replies: Vec<Value> = batch.borrow().iter().filter_map(|m| server.handle(m)).collect();
                (!replies.is_empty()).then(|| arr_val(replies))
            }
            Some(message) => server.handle(&message),
            None => Some(obj(vec![("jsonrpc", str_val("2.0")), ("id", Value::Null), ("error", obj(vec![("code", Value::Num(-32700.0)), ("message", str_val("JSON inválido"))]))])),
        };
        if let Some(r) = response {
            let _ = writeln!(stdout, "{}", json(&r));
            let _ = stdout.flush();
        }
    }
    std::process::ExitCode::SUCCESS
}
