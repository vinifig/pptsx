//! O registro de design systems (`pptsx.registry.json`): a lista de DS que o catálogo visual
//! (`pptsx-rs catalog`) e o servidor MCP enxergam sem `--ds`. É JSON, e não TS, porque não executa
//! nada: só diz onde cada DS mora (um caminho relativo ao arquivo, ou um pacote), e quem lê é o
//! binário, um editor ou um script de CI. `pptsx-rs registry add|remove|list` mantém o arquivo.
//!
//! ```json
//! { "designSystems": [ { "id": "ds-base", "ds": "packages/ds-base/src/index.ts" } ] }
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use crate::js::value::{Obj, Value, arr_val, obj_val, str_val};

pub const REGISTRY_FILE: &str = "pptsx.registry.json";

/// Uma entrada do registro: o id (a chave na URL do catálogo e na pasta de saída) e o especificador.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    /// Caminho relativo à pasta do registro (arquivo `.ts`/`.tsx` ou pasta do DS) ou nome de pacote.
    pub ds: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Registry {
    pub entries: Vec<Entry>,
}

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(o) => o.get(key).filter(|v| !v.is_nullish()),
        _ => None,
    }
}

fn get_str(v: &Value, key: &str) -> Option<String> {
    get(v, key).map(Value::to_js_string).filter(|s| !s.is_empty())
}

/// O id de um DS a partir do nome do manifesto: `@pptsx/ds-base` → `ds-base`, `Aurora DS` → `aurora-ds`.
pub fn id_of(name: &str) -> String {
    let bare = name.rsplit('/').next().unwrap_or(name);
    let slug: String = bare.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
    let parts: Vec<&str> = slug.split('-').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() { "ds".into() } else { parts.join("-") }
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') && !id.starts_with('-')
}

/// Lê o texto do registro. Ids repetidos ou inválidos são erro: o id vira pasta e âncora do catálogo.
pub fn parse(text: &str) -> Result<Registry, String> {
    let root = crate::js::parse_json(text).ok_or("JSON inválido")?;
    let list = match get(&root, "designSystems") {
        Some(Value::Array(a)) => a.borrow().clone(),
        Some(_) => return Err("\"designSystems\" precisa ser uma lista".into()),
        None => Vec::new(),
    };
    let mut entries: Vec<Entry> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let ds = get_str(item, "ds").ok_or_else(|| format!("designSystems[{i}]: falta \"ds\" (caminho ou pacote do DS)"))?;
        let id = get_str(item, "id").unwrap_or_else(|| id_of(&ds));
        if !is_valid_id(&id) {
            return Err(format!("designSystems[{i}]: id \"{id}\" inválido (letras minúsculas, dígitos e hífens)"));
        }
        if entries.iter().any(|e| e.id == id) {
            return Err(format!("designSystems[{i}]: id \"{id}\" repetido"));
        }
        entries.push(Entry { id, ds, description: get_str(item, "description") });
    }
    Ok(Registry { entries })
}

/// O texto do registro, com recuo de 2 e quebra de linha no fim (para ficar bem no diff).
pub fn serialize(registry: &Registry) -> String {
    let entry = |e: &Entry| {
        let mut o = Obj::new();
        o.insert(Rc::from("id"), str_val(&e.id));
        o.insert(Rc::from("ds"), str_val(&e.ds));
        if let Some(d) = &e.description {
            o.insert(Rc::from("description"), str_val(d));
        }
        obj_val(o)
    };
    let mut root = Obj::new();
    root.insert(Rc::from("designSystems"), arr_val(registry.entries.iter().map(entry).collect()));
    format!("{}\n", crate::mcp::pretty_json(&obj_val(root)))
}

/// Acrescenta ou troca (mesmo id) uma entrada; a ordem do arquivo é a ordem do catálogo.
pub fn with_entry(registry: &Registry, entry: Entry) -> Registry {
    let mut entries = registry.entries.clone();
    match entries.iter().position(|e| e.id == entry.id) {
        Some(i) => entries[i] = entry,
        None => entries.push(entry),
    }
    Registry { entries }
}

pub fn without_entry(registry: &Registry, id: &str) -> Option<Registry> {
    let entries: Vec<Entry> = registry.entries.iter().filter(|e| e.id != id).cloned().collect();
    (entries.len() < registry.entries.len()).then_some(Registry { entries })
}

/// O `pptsx.registry.json` desta pasta ou da mais próxima acima (como o `pptsx.config.ts`).
pub fn find(dir: &Path) -> Option<PathBuf> {
    let mut dir = std::fs::canonicalize(dir).ok()?;
    loop {
        let candidate = dir.join(REGISTRY_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn load(file: &Path) -> Result<Registry, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", file.display()))
}

/// Um DS já resolvido para carregar: o id, o especificador e a pasta a partir da qual ele resolve.
#[derive(Clone, Debug)]
pub struct Source {
    /// Do registro; um `--ds` avulso não tem (o catálogo usa o nome do manifesto).
    pub id: Option<String>,
    pub spec: String,
    pub base: PathBuf,
    pub description: Option<String>,
}

/// As fontes do registro (resolvidas a partir da pasta dele).
pub fn sources_of(file: &Path, registry: &Registry) -> Vec<Source> {
    let base = file.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
    registry.entries.iter().map(|e| Source { id: Some(e.id.clone()), spec: e.ds.clone(), base: base.clone(), description: e.description.clone() }).collect()
}

/// As fontes de um comando: as do `--registry` (ou do registro achado a partir do cwd) e as dos `--ds`.
pub fn collect(registry: Option<&Path>, ds_args: &[String], cwd: &Path) -> Result<(Vec<Source>, Option<PathBuf>), String> {
    let file = match registry {
        Some(f) => Some(if f.is_absolute() { f.to_path_buf() } else { cwd.join(f) }),
        // Com `--ds` explícito e sem `--registry`, o registro achado não entra: o pedido foi só aquele DS.
        None if ds_args.is_empty() => find(cwd),
        None => None,
    };
    let mut sources = match &file {
        Some(f) => sources_of(f, &load(f)?),
        None => Vec::new(),
    };
    sources.extend(ds_args.iter().map(|spec| Source { id: None, spec: spec.clone(), base: cwd.to_path_buf(), description: None }));
    Ok((sources, file))
}

// ───────────── `pptsx-rs registry` ─────────────

const USAGE: &str = "uso:
  pptsx-rs registry list [--registry <arquivo>] [--json]
  pptsx-rs registry add <caminho|pasta|pacote> [--id <id>] [--description <texto>] [--registry <arquivo>]
  pptsx-rs registry remove <id> [--registry <arquivo>]

O registro (pptsx.registry.json, nesta pasta ou acima) lista os design systems que `pptsx-rs catalog` e
`pptsx-rs mcp` enxergam sem --ds. Sem registro, `add` cria um na pasta atual.";

struct Args {
    action: String,
    target: Option<String>,
    id: Option<String>,
    description: Option<String>,
    registry: Option<PathBuf>,
    json: bool,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut it = raw.iter();
    let action = it.next().cloned().ok_or_else(|| USAGE.to_string())?;
    let mut args = Args { action, target: None, id: None, description: None, registry: None, json: false };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--id" => args.id = Some(it.next().cloned().ok_or("--id precisa de um valor")?),
            "--description" => args.description = Some(it.next().cloned().ok_or("--description precisa de um texto")?),
            "--registry" => args.registry = Some(PathBuf::from(it.next().ok_or("--registry precisa de um arquivo")?)),
            "--json" => args.json = true,
            "-h" | "--help" => return Err(USAGE.into()),
            other if other.starts_with("--") => return Err(format!("opção desconhecida: {other}\n\n{USAGE}")),
            other => args.target = Some(other.to_string()),
        }
    }
    Ok(args)
}

/// O caminho como o registro guarda: relativo à pasta dele e com `/` (o arquivo vai para o git).
fn stored_spec(spec: &str, cwd: &Path, registry_dir: &Path) -> String {
    let (_, by_path) = crate::dsinfo::entry_of(spec, cwd).unwrap_or((PathBuf::new(), false));
    if !by_path {
        return spec.to_string();
    }
    let target = if Path::new(spec).is_absolute() { PathBuf::from(spec) } else { cwd.join(spec) };
    let rel = crate::dsinfo::relative_path(registry_dir, &target);
    rel.strip_prefix("./").map(str::to_string).unwrap_or(rel)
}

fn write(file: &Path, registry: &Registry) -> Result<(), String> {
    std::fs::write(file, serialize(registry)).map_err(|e| format!("{}: {e}", file.display()))
}

fn list(file: Option<&Path>, json: bool) -> Result<(), String> {
    let Some(file) = file else { return Err(format!("nenhum {REGISTRY_FILE} nesta pasta nem acima; crie com `pptsx-rs registry add <ds>`")) };
    let registry = load(file)?;
    let base = file.parent().unwrap_or(Path::new("."));
    let mut rows: Vec<Value> = Vec::new();
    if !json {
        println!("{}", file.display());
    }
    for e in &registry.entries {
        let loaded = crate::dsinfo::load(&e.ds, base, base);
        let mut o = Obj::new();
        o.insert(Rc::from("id"), str_val(&e.id));
        o.insert(Rc::from("ds"), str_val(&e.ds));
        match &loaded {
            Ok(info) => {
                o.insert(Rc::from("name"), str_val(&info.name));
                o.insert(Rc::from("version"), str_val(&info.version));
                o.insert(Rc::from("components"), Value::Num(info.components.len() as f64));
                if !json {
                    let cases: usize = info.components.iter().map(|c| c.cases.len()).sum();
                    println!("  \x1b[32m✓\x1b[0m {:<16} {} {} · {} componentes, {} casos · {}", e.id, info.name, info.version, info.components.len(), cases, e.ds);
                }
            }
            Err(err) => {
                o.insert(Rc::from("error"), str_val(err));
                if !json {
                    println!("  \x1b[31m✗\x1b[0m {:<16} {} · {err}", e.id, e.ds);
                }
            }
        }
        rows.push(obj_val(o));
    }
    if json {
        println!("{}", crate::mcp::pretty_json(&arr_val(rows)));
    }
    Ok(())
}

fn add(args: &Args, cwd: &Path) -> Result<(), String> {
    let spec = args.target.as_deref().ok_or_else(|| format!("faltou o DS (caminho, pasta ou pacote)\n\n{USAGE}"))?;
    // Carregar antes de gravar: um DS que não avalia não entra no registro.
    let info = crate::dsinfo::load(spec, cwd, cwd)?;
    let file = args.registry.clone().map(|f| if f.is_absolute() { f } else { cwd.join(f) }).or_else(|| find(cwd)).unwrap_or_else(|| cwd.join(REGISTRY_FILE));
    let current = if file.is_file() { load(&file)? } else { Registry::default() };
    let id = args.id.clone().unwrap_or_else(|| id_of(&info.name));
    if !is_valid_id(&id) {
        return Err(format!("id \"{id}\" inválido (letras minúsculas, dígitos e hífens)"));
    }
    let dir = file.parent().map(Path::to_path_buf).unwrap_or_else(|| cwd.to_path_buf());
    let entry = Entry { id: id.clone(), ds: stored_spec(spec, cwd, &dir), description: args.description.clone().or(info.description.clone()) };
    let replaced = current.entries.iter().any(|e| e.id == id);
    write(&file, &with_entry(&current, entry))?;
    let cases: usize = info.components.iter().map(|c| c.cases.len()).sum();
    println!(
        "\x1b[32m✓\x1b[0m {} \"{id}\" em {}: {} {} ({} componentes, {} casos de catálogo)",
        if replaced { "atualizado" } else { "registrado" },
        file.display(),
        info.name,
        info.version,
        info.components.len(),
        cases
    );
    println!("  próximo: pptsx-rs catalog   (o catálogo visual de todos os DS registrados)");
    Ok(())
}

fn remove(args: &Args, cwd: &Path) -> Result<(), String> {
    let id = args.target.as_deref().ok_or_else(|| format!("faltou o id\n\n{USAGE}"))?;
    let file = args.registry.clone().map(|f| if f.is_absolute() { f } else { cwd.join(f) }).or_else(|| find(cwd)).ok_or(format!("nenhum {REGISTRY_FILE} nesta pasta nem acima"))?;
    let next = without_entry(&load(&file)?, id).ok_or_else(|| format!("\"{id}\" não está em {}", file.display()))?;
    write(&file, &next)?;
    println!("\x1b[32m✓\x1b[0m \"{id}\" removido de {}", file.display());
    Ok(())
}

pub fn run(raw: &[String]) -> ExitCode {
    let args = match parse_args(raw) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let result = match args.action.as_str() {
        "list" | "ls" => list(args.registry.as_deref().map(|f| cwd.join(f)).or_else(|| find(&cwd)).as_deref(), args.json),
        "add" => add(&args, &cwd),
        "remove" | "rm" => remove(&args, &cwd),
        other => Err(format!("ação desconhecida: {other}\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_of_drops_scope_and_slugifies() {
        assert_eq!(id_of("@pptsx/ds-base"), "ds-base");
        assert_eq!(id_of("Aurora DS"), "aurora-ds");
        assert_eq!(id_of("../ds/"), "ds");
        assert_eq!(id_of("--"), "ds");
    }

    #[test]
    fn parse_reads_entries_and_defaults_the_id() {
        let r = parse(r#"{ "designSystems": [ { "ds": "@acme/ds" }, { "id": "aurora", "ds": "examples/aurora/ds.ts", "description": "x" } ] }"#).expect("ok");
        assert_eq!(r.entries[0], Entry { id: "ds".into(), ds: "@acme/ds".into(), description: None });
        assert_eq!(r.entries[1].id, "aurora");
        assert_eq!(r.entries[1].description.as_deref(), Some("x"));
    }

    #[test]
    fn parse_rejects_repeated_and_invalid_ids() {
        assert!(parse(r#"{ "designSystems": [ { "id": "a", "ds": "x" }, { "id": "a", "ds": "y" } ] }"#).unwrap_err().contains("repetido"));
        assert!(parse(r#"{ "designSystems": [ { "id": "Não", "ds": "x" } ] }"#).unwrap_err().contains("inválido"));
        assert!(parse(r#"{ "designSystems": [ { "id": "a" } ] }"#).unwrap_err().contains("falta"));
    }

    #[test]
    fn serialize_round_trips_and_with_entry_replaces_by_id() {
        let r = Registry { entries: vec![Entry { id: "a".into(), ds: "x.ts".into(), description: None }] };
        let r = with_entry(&r, Entry { id: "b".into(), ds: "y.ts".into(), description: Some("d".into()) });
        let r = with_entry(&r, Entry { id: "a".into(), ds: "z.ts".into(), description: None });
        assert_eq!(parse(&serialize(&r)).expect("ok"), r);
        assert_eq!(r.entries[0].ds, "z.ts");
        assert!(without_entry(&r, "nada").is_none());
        assert_eq!(without_entry(&r, "a").expect("removido").entries.len(), 1);
    }
}
