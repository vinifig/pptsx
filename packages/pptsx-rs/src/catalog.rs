//! `pptsx-rs catalog`: o catálogo visual dos design systems registrados. Para cada DS (do
//! `pptsx.registry.json` ou de `--ds`), cada caso de catálogo (`meta.catalog`) vira um slide de um
//! deck gerado, construído pelo próprio binário (o mesmo layout, fit e diagnósticos de um deck real).
//! Sai uma pasta com:
//!
//! ```text
//! <out>/index.html              o visualizador (um arquivo só, SVGs embutidos, abre sem servidor)
//! <out>/catalog.json            o contrato exportado (pptsx.catalog/1): DS, componentes, props, casos
//! <out>/<id>/prompt.md          o guia do DS para um agente (o mesmo `ds.md` da skill)
//! <out>/<id>/catalog.<tam>.pptx um slide por caso, para abrir no PowerPoint/Keynote
//! <out>/<id>/previews/<tam>/N.svg  o SVG de cada slide (e state.json/diagnostics.json do build)
//! ```
//!
//! `--json` e `--prompt` só leem os manifestos (sem construir nada) e imprimem o contrato ou o
//! prompt de todos os DS: é o caminho de um agente ou de um script.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::rc::Rc;

use crate::dsinfo::{DsComponent, DsInfo};
use crate::js::value::{Obj, Value, arr_val, obj_val, str_val, to_json};
use crate::registry::Source;

/// Versão do formato de `catalog.json`: muda só quando um campo existente muda de sentido.
pub const FORMAT: &str = "pptsx.catalog/1";
const DEFAULT_OUT: &str = "dist/catalog";
const DEFAULT_SIZES: &str = "16x9,4x3";
/// Margem do slide em volta do componente (a mesma do `fragmentSlide` do legado).
const CASE_PADDING: u32 = 36;
const DECK_FILE: &str = "catalog.tsx";
const VIEWER: &str = include_str!("../assets/catalog.html");
const DATA_MARK: &str = "/*__CATALOG__*/null";

// ───────────── o deck de catálogo ─────────────

/// Um slide do deck gerado: o componente e o caso (índices em `DsInfo::components` e em `cases`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaseRef {
    pub component: usize,
    pub case: usize,
}

/// Todos os casos do DS, na ordem dos componentes.
pub fn case_refs(ds: &DsInfo) -> Vec<CaseRef> {
    ds.components.iter().enumerate().flat_map(|(ci, c)| (0..c.cases.len()).map(move |k| CaseRef { component: ci, case: k })).collect()
}

fn js_string(s: &str) -> String {
    let mut out = String::new();
    to_json(&str_val(s), &mut out);
    out
}

/// O JSX de um slide: um layout (`slide: true`) entra direto no Deck e recebe `ctx`; um caso `fill`
/// preenche o corpo; o resto fica no tamanho natural, numa coluna de grid (como o `fragmentSlide`:
/// a largura toda, a altura do conteúdo, e um `flex={1}` do componente não colapsa).
fn slide_jsx(ds: &DsInfo, at: CaseRef) -> String {
    let component = &ds.components[at.component];
    let case = &component.cases[at.case];
    let name = js_string(&component.name);
    if component.slide {
        return format!("    {{show({name}, {}, {{ ctx }})}}", at.case);
    }
    let mut slide_props = vec![format!("padding={{{CASE_PADDING}}}")];
    if let Some((_, bg)) = ds.theme_colors.iter().find(|(slot, _)| slot == "light1") {
        slide_props.push(format!("backgroundColor=\"{bg}\""));
    }
    if let Some((_, ink)) = ds.theme_colors.iter().find(|(slot, _)| slot == "dark1") {
        slide_props.push(format!("color=\"{ink}\""));
    }
    let open = format!("<Slide {}>", slide_props.join(" "));
    let body = format!("{{show({name}, {})}}", at.case);
    if case.fill {
        return format!("    {open}{body}</Slide>");
    }
    format!("    {open}\n      <Box display=\"grid\" gridTemplateColumns=\"minmax(0, 1fr)\">{body}</Box>\n    </Slide>")
}

/// O deck de catálogo: importa o módulo do DS inteiro e lê cada caso do próprio manifesto, então as
/// props saem como o autor do DS escreveu (inclusive elementos em slots), sem serializar nada.
pub fn deck_source(ds: &DsInfo, import_from: &str, cases: &[CaseRef]) -> String {
    let slides: Vec<String> = cases.iter().map(|&at| slide_jsx(ds, at)).collect();
    format!(
        "// Gerado por `pptsx-rs catalog`: um slide por caso do catálogo do {name}. Não edite: o catálogo é refeito a cada execução.
import {{ Box, COMPONENT_META, Deck, Slide, defineDeck }} from 'pptsx';
import * as mod from {from};

const ds = mod.ds ?? mod.default;
const implementations = ds.implementations ?? [];
// createDs guarda as funções em `implementations`; um defineDs puro só tem os metas, e a função é o export de mesmo nome.
const viewOf = (name) => implementations.find((c) => c[COMPONENT_META]?.name === name) ?? mod[name];
const caseOf = (name, index) => ds.components.find((meta) => meta.name === name).catalog[index];
const show = (name, index, extra) => {{
  const View = viewOf(name);
  return <View {{...caseOf(name, index).props}} {{...extra}} />;
}};

export default defineDeck((ctx) => (
  <Deck>
{slides}
  </Deck>
));
",
        name = ds.name,
        from = js_string(import_from),
        slides = slides.join("\n"),
    )
}

/// A config da pasta do deck de catálogo: o tema do DS no arquivo (as cores do `.pptx` saem do tema).
pub fn config_source(import_from: &str) -> String {
    format!(
        "// Gerado por `pptsx-rs catalog`.\nimport {{ defineConfig }} from 'pptsx';\nimport * as mod from {};\n\nconst ds = mod.ds ?? mod.default;\nexport default defineConfig({{ ds: [ds], theme: ds.theme }});\n",
        js_string(import_from)
    )
}

// ───────────── resultado do build ─────────────

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Diagnostic {
    pub size: String,
    pub severity: String,
    pub code: String,
    pub path: String,
    pub message: String,
}

/// O que o build do deck de catálogo deu para um caso: os SVGs por tamanho (um caso pode paginar em
/// vários slides) e os diagnósticos dele.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaseRender {
    /// (tamanho, caminhos dos SVGs relativos à pasta de saída)
    pub previews: Vec<(String, Vec<String>)>,
    pub diagnostics: Vec<Diagnostic>,
    pub error: Option<String>,
}

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(o) => o.get(key).filter(|v| !v.is_nullish()),
        _ => None,
    }
}

fn get_str(v: &Value, key: &str) -> String {
    get(v, key).map(Value::to_js_string).unwrap_or_default()
}

fn list(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(Value::Array(a)) => a.borrow().clone(),
        _ => Vec::new(),
    }
}

/// Lê o `state.json` e o `diagnostics.json` do build: o slide de saída → o slide do autor (o caso), e
/// os diagnósticos de cada caso. `prefix` é a pasta dos SVGs relativa à saída do catálogo.
pub fn renders_of(state: &Value, diagnostics: &Value, prefix: &str, count: usize) -> Vec<CaseRender> {
    let mut out: Vec<CaseRender> = vec![CaseRender::default(); count];
    for size in list(get(state, "sizes")) {
        let name = get_str(&size, "size");
        let mut per_case: Vec<Vec<String>> = vec![Vec::new(); count];
        // Número do slide de saída → slide do autor, para achar o caso de um diagnóstico de uma parte paginada.
        let mut source_of: Vec<(usize, usize)> = Vec::new();
        for slide in list(get(&size, "slides")) {
            let number = get(&slide, "number").map_or(0.0, Value::to_num) as usize;
            let source = get(&slide, "sourceSlide").map_or(0.0, Value::to_num) as usize;
            source_of.push((number, source));
            if (1..=count).contains(&source) {
                per_case[source - 1].push(format!("{prefix}/{name}/{number}.svg"));
            }
        }
        for (i, files) in per_case.into_iter().enumerate() {
            out[i].previews.push((name.clone(), files));
        }
        for d in list(Some(diagnostics)).iter().filter(|d| get_str(d, "size") == name) {
            let slide = get(d, "slide").map_or(0.0, Value::to_num) as usize;
            let source = get(d, "data").and_then(|data| get(data, "sourceSlide")).map(|s| s.to_num() as usize);
            let source = source.or_else(|| source_of.iter().find(|(n, _)| *n == slide).map(|(_, s)| *s)).unwrap_or(0);
            if !(1..=count).contains(&source) {
                continue;
            }
            out[source - 1].diagnostics.push(Diagnostic { size: name.clone(), severity: get_str(d, "severity"), code: get_str(d, "code"), path: get_str(d, "path"), message: get_str(d, "message") });
        }
    }
    out
}

// ───────────── o contrato exportado (catalog.json) ─────────────

fn obj(entries: Vec<(&str, Value)>) -> Value {
    let mut o = Obj::new();
    for (k, v) in entries {
        if !matches!(v, Value::Undefined) {
            o.insert(Rc::from(k), v);
        }
    }
    obj_val(o)
}

fn opt(s: &Option<String>) -> Value {
    s.as_deref().map_or(Value::Undefined, str_val)
}

fn strings(items: &[String]) -> Value {
    arr_val(items.iter().map(|s| str_val(s)).collect())
}

/// Um DS do catálogo: o manifesto lido, de onde veio e, quando houve build, o resultado por caso.
pub struct Entry {
    pub id: String,
    /// O especificador como o registro (ou o `--ds`) o escreveu.
    pub spec: String,
    pub info: Result<DsInfo, String>,
    /// Um por caso, na ordem de `case_refs`; vazio sem build.
    pub renders: Vec<CaseRender>,
    pub pptx: Vec<(String, String)>,
    pub prompt_file: Option<String>,
    pub description: Option<String>,
    /// A pasta a partir da qual o especificador resolve (a do registro, ou o cwd de um `--ds`).
    pub base: PathBuf,
}

fn diagnostic_value(d: &Diagnostic) -> Value {
    obj(vec![("size", str_val(&d.size)), ("severity", str_val(&d.severity)), ("code", str_val(&d.code)), ("path", str_val(&d.path)), ("message", str_val(&d.message))])
}

fn case_value(component: &DsComponent, k: usize, render: Option<&CaseRender>) -> Value {
    let case = &component.cases[k];
    let previews = render.map(|r| {
        let mut o = Obj::new();
        for (size, files) in &r.previews {
            o.insert(Rc::from(size.as_str()), strings(files));
        }
        obj_val(o)
    });
    obj(vec![
        ("name", str_val(&case.name)),
        ("description", opt(&case.description)),
        ("fill", if case.fill { Value::Bool(true) } else { Value::Undefined }),
        ("audience", strings(&case.audience)),
        ("usage", str_val(&crate::dsinfo::usage_with(&component.name, Some(&case.props)))),
        ("previews", previews.unwrap_or(Value::Undefined)),
        ("diagnostics", render.map_or(Value::Undefined, |r| arr_val(r.diagnostics.iter().map(diagnostic_value).collect()))),
        ("error", render.map_or(Value::Undefined, |r| opt(&r.error))),
    ])
}

fn component_value(ds: &DsInfo, ci: usize, renders: &[CaseRender], refs: &[CaseRef]) -> Value {
    let c = &ds.components[ci];
    let schema = ds.schema.as_ref().and_then(|s| get(s, &c.name)).cloned();
    let cases: Vec<Value> = (0..c.cases.len())
        .map(|k| {
            let render = refs.iter().position(|r| r.component == ci && r.case == k).and_then(|i| renders.get(i));
            case_value(c, k, render)
        })
        .collect();
    obj(vec![
        ("name", str_val(&c.name)),
        ("displayName", opt(&c.display_name)),
        ("description", opt(&c.description)),
        ("category", opt(&c.category)),
        ("tags", strings(&c.tags)),
        ("kind", str_val(c.kind.name())),
        ("slide", if c.slide { Value::Bool(true) } else { Value::Undefined }),
        ("whenToUse", strings(&c.when_to_use)),
        ("avoid", strings(&c.avoid)),
        ("audience", strings(&c.audience)),
        ("guide", opt(&c.guide)),
        ("signature", schema.as_ref().map_or_else(|| str_val(&format!("<{} />", c.name)), |s| str_val(&crate::dsinfo::signature_of(&c.name, s)))),
        ("schema", schema.unwrap_or(Value::Undefined)),
        ("cases", arr_val(cases)),
    ])
}

/// Quantos componentes o DS tem em cada camada.
fn tiers_value(ds: &DsInfo) -> Value {
    let mut o = Obj::new();
    for tier in crate::dsinfo::Tier::ALL {
        o.insert(Rc::from(tier.name()), Value::Num(ds.components.iter().filter(|c| c.kind == tier).count() as f64));
    }
    obj_val(o)
}

/// Recomendações do contrato que o DS não cumpre (não são erro: o catálogo e o prompt saem igual).
pub fn notes_of(ds: &DsInfo) -> Vec<String> {
    let mut notes = Vec::new();
    if !ds.components.iter().any(|c| c.kind == crate::dsinfo::Tier::Template) {
        notes.push("Sem templates de slide (kind: \"template\"): recomendado, para o agente começar cada slide por um layout da marca.".to_string());
    }
    let without: Vec<&str> = ds.components.iter().filter(|c| c.cases.is_empty()).map(|c| c.name.as_str()).collect();
    if !without.is_empty() {
        notes.push(format!("Sem caso de catálogo (não aparecem na galeria): {}.", without.join(", ")));
    }
    let unaimed: Vec<&str> = ds.components.iter().filter(|c| c.audience.is_empty() && c.cases.iter().all(|k| k.audience.is_empty())).map(|c| c.name.as_str()).collect();
    if !unaimed.is_empty() {
        notes.push(format!("Sem audience (o agente não sabe para que público servem): {}.", unaimed.join(", ")));
    }
    let undescribed: Vec<&str> = ds.components.iter().filter(|c| c.description.is_none()).map(|c| c.name.as_str()).collect();
    if !undescribed.is_empty() {
        notes.push(format!("Sem description (o prompt fica sem a linha de propósito): {}.", undescribed.join(", ")));
    }
    notes
}

fn ds_value(entry: &Entry) -> Value {
    let base = vec![("id", str_val(&entry.id)), ("source", str_val(&entry.spec))];
    let ds = match &entry.info {
        Ok(ds) => ds,
        Err(e) => return obj(base.into_iter().chain([("name", str_val(&entry.id)), ("error", str_val(e)), ("components", arr_val(Vec::new()))]).collect()),
    };
    let refs = case_refs(ds);
    let mut colors = Obj::new();
    for (slot, hex) in &ds.theme_colors {
        colors.insert(Rc::from(slot.as_str()), str_val(hex));
    }
    let mut pptx = Obj::new();
    for (size, file) in &entry.pptx {
        pptx.insert(Rc::from(size.as_str()), str_val(file));
    }
    let rules: Vec<Value> = ds.rules.iter().map(|(id, sev)| obj(vec![("id", str_val(id)), ("severity", str_val(sev))])).collect();
    let components: Vec<Value> = (0..ds.components.len()).map(|ci| component_value(ds, ci, &entry.renders, &refs)).collect();
    obj(base
        .into_iter()
        .chain([
            ("name", str_val(&ds.name)),
            ("version", str_val(&ds.version)),
            ("description", opt(&entry.description.clone().or_else(|| ds.description.clone()))),
            ("theme", if colors.is_empty() { Value::Undefined } else { obj(vec![("colors", obj_val(colors))]) }),
            ("tiers", tiers_value(ds)),
            ("notes", strings(&notes_of(ds))),
            ("rules", arr_val(rules)),
            ("guide", opt(&ds.guide)),
            ("prompt", opt(&entry.prompt_file)),
            ("pptx", if pptx.is_empty() { Value::Undefined } else { obj_val(pptx) }),
            ("components", arr_val(components)),
        ])
        .collect())
}

/// O `catalog.json`: o contrato que um visualizador, um agente ou outra ferramenta lê.
pub fn catalog_value(entries: &[Entry], sizes: &[crate::js::DeckCtx]) -> Value {
    let sizes: Vec<Value> = sizes.iter().map(|s| obj(vec![("name", str_val(&s.name)), ("width", Value::Num(s.width)), ("height", Value::Num(s.height))])).collect();
    obj(vec![
        ("format", str_val(FORMAT)),
        ("generator", str_val(&format!("pptsx-rs {}", env!("CARGO_PKG_VERSION")))),
        ("sizes", arr_val(sizes)),
        ("designSystems", arr_val(entries.iter().map(ds_value).collect())),
    ])
}

/// O prompt de um DS: o mesmo `ds.md` da skill, com o import e a config apontando para o especificador.
/// `spec` é o DS como o registro o escreveu; o import é o arquivo de entrada a partir da pasta do registro.
pub fn prompt_of(ds: &DsInfo, spec: &str, base: &Path) -> String {
    let entry = if ds.by_path { crate::dsinfo::relative_path(base, &ds.entry) } else { spec.to_string() };
    let relative = if ds.by_path { "; o caminho é a partir da raiz do projeto, no deck use o relativo até ele" } else { "" };
    let location = crate::dsinfo::DsLocation {
        import_from: entry.trim_start_matches("./").to_string(),
        config_hint: format!("`pptsx-rs init <pasta> --ds {spec}` cria o deck e esse arquivo com o caminho certo{relative}"),
    };
    crate::dsinfo::markdown(ds, &location, crate::skill::ds_props_table)
}

/// O visualizador: a página com o catálogo e os SVGs embutidos (abre direto do disco, sem servidor).
/// `</` vira `<\/` para o JSON não fechar o `<script>`.
pub fn viewer_html(catalog: &Value, svgs: &[(String, String)], prompts: &[(String, String)]) -> String {
    let mut files = Obj::new();
    for (path, svg) in svgs {
        files.insert(Rc::from(path.as_str()), str_val(svg));
    }
    let mut prompt_texts = Obj::new();
    for (id, text) in prompts {
        prompt_texts.insert(Rc::from(id.as_str()), str_val(text));
    }
    let data = obj(vec![("catalog", catalog.clone()), ("files", obj_val(files)), ("prompts", obj_val(prompt_texts))]);
    let mut json = String::new();
    to_json(&data, &mut json);
    VIEWER.replacen(DATA_MARK, &json.replace("</", "<\\/"), 1)
}

// ───────────── índice para agentes (MCP) ─────────────

/// Um DS como o índice o mostra.
pub struct IndexDs<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub description: Option<&'a str>,
    pub components: &'a [DsComponent],
    pub schema: Option<&'a Value>,
}

/// O índice curto do catálogo, por camada (template → widget): para o agente escolher o que usar antes
/// de pedir o `schema` de um componente. Filtra por público (`audience`) e camada (`kind`); um componente
/// sem `audience` serve a qualquer público.
pub fn index_markdown(ds: &IndexDs, audience: Option<&str>, kind: Option<&str>) -> String {
    let fits = |c: &DsComponent| {
        let audiences: Vec<&String> = c.audience.iter().chain(c.cases.iter().flat_map(|k| k.audience.iter())).collect();
        audience.is_none_or(|a| audiences.is_empty() || audiences.iter().any(|x| x.as_str() == a)) && kind.is_none_or(|k| c.kind.name() == k)
    };
    let mut out = vec![format!("# {} {}", ds.name, ds.version)];
    if let Some(d) = ds.description {
        out.push(format!("\n{}", d.trim()));
    }
    let tiers: Vec<crate::dsinfo::Tier> = crate::dsinfo::Tier::ALL.into_iter().filter(|t| ds.components.iter().any(|c| c.kind == *t)).collect();
    if !tiers.contains(&crate::dsinfo::Tier::Template) {
        out.push("\nSem templates de slide: monte cada slide com `<Slide>` e as estruturas.".into());
    }
    let mut shown = 0;
    for tier in tiers {
        let list: Vec<&DsComponent> = ds.components.iter().filter(|c| c.kind == tier && fits(c)).collect();
        if list.is_empty() {
            continue;
        }
        out.push(format!("\n## {} (`kind: \"{}\"`)\n", tier.title(), tier.name()));
        for c in list {
            shown += 1;
            let schema = ds.schema.and_then(|s| get(s, &c.name));
            let signature = schema.map_or_else(|| format!("<{} />", c.name), |s| crate::dsinfo::signature_of(&c.name, s));
            let mut line = format!("- `{signature}`");
            if let Some(d) = &c.description {
                line.push_str(&format!(" — {}", d.trim()));
            }
            if !c.audience.is_empty() {
                line.push_str(&format!(" Público: {}.", c.audience.join(", ")));
            }
            out.push(line);
            for w in &c.when_to_use {
                out.push(format!("  - use: {w}"));
            }
            for a in &c.avoid {
                out.push(format!("  - evite: {a}"));
            }
            for k in c.cases.iter().filter(|k| audience.is_none_or(|a| k.audience.is_empty() || k.audience.iter().any(|x| x == a))) {
                let aud = if k.audience.is_empty() { String::new() } else { format!(" ({})", k.audience.join(", ")) };
                out.push(format!("  - caso `{}`{aud}: `{}`", k.name, crate::dsinfo::usage_with(&c.name, Some(&k.props))));
            }
        }
    }
    if shown == 0 {
        out.push("\nNenhum componente com esse filtro.".into());
    }
    out.join("\n") + "\n"
}

// ───────────── CLI ─────────────

const USAGE: &str = "uso:
  pptsx-rs catalog [--ds <caminho|pasta|pacote>]... [--registry <arquivo>] [--out dist/catalog] [--size 16x9,4x3]
  pptsx-rs catalog --json      o contrato (pptsx.catalog/1) de todos os DS, sem construir nada
  pptsx-rs catalog --prompt    o prompt (markdown) de todos os DS, para colar num agente

Sem --ds, os DS do pptsx.registry.json desta pasta ou da mais próxima acima (`pptsx-rs registry add`).
Gera <out>/index.html (o visualizador), catalog.json, e por DS prompt.md, catalog.<tamanho>.pptx e os SVGs.";

struct Args {
    ds: Vec<String>,
    registry: Option<PathBuf>,
    out: PathBuf,
    sizes: String,
    json: bool,
    prompt: bool,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut args = Args { ds: Vec::new(), registry: None, out: PathBuf::from(DEFAULT_OUT), sizes: DEFAULT_SIZES.into(), json: false, prompt: false };
    let mut it = raw.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--ds" => args.ds.push(it.next().cloned().ok_or("--ds precisa de um caminho ou pacote")?),
            "--registry" => args.registry = Some(PathBuf::from(it.next().ok_or("--registry precisa de um arquivo")?)),
            "--out" => args.out = PathBuf::from(it.next().ok_or("--out precisa de uma pasta")?),
            "--size" => args.sizes = it.next().cloned().ok_or("--size precisa de um valor")?,
            "--json" => args.json = true,
            "--prompt" => args.prompt = true,
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("opção desconhecida: {other}\n\n{USAGE}")),
        }
    }
    Ok(args)
}

/// Carrega cada fonte e dá a ela um id único (o do registro, senão o do nome do manifesto).
fn load_all(sources: &[Source]) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    for s in sources {
        let info = crate::dsinfo::load(&s.spec, &s.base, &s.base);
        let wanted = s.id.clone().unwrap_or_else(|| info.as_ref().map_or_else(|_| crate::registry::id_of(&s.spec), |i| crate::registry::id_of(&i.name)));
        // Um `--ds` com o mesmo DS de uma entrada do registro não aparece duas vezes.
        if s.id.is_none() && entries.iter().any(|e| e.id == wanted && e.info.as_ref().ok().map(|i| &i.entry) == info.as_ref().ok().map(|i| &i.entry)) {
            continue;
        }
        let mut id = wanted.clone();
        let mut n = 2;
        while entries.iter().any(|e| e.id == id) {
            id = format!("{wanted}-{n}");
            n += 1;
        }
        entries.push(Entry { id, spec: s.spec.clone(), info, renders: Vec::new(), pptx: Vec::new(), prompt_file: None, description: s.description.clone(), base: s.base.clone() });
    }
    entries
}

/// Roda `pptsx-rs build` sobre o deck de catálogo num processo filho (como o `--watch` e o MCP): a
/// memória do avaliador volta a zero e um DS que derruba a avaliação não derruba o catálogo.
fn build_deck(deck: &Path, out: &Path, state: &Path, sizes: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("não achei o próprio binário: {e}"))?;
    let _ = std::fs::remove_dir_all(state);
    let output = Command::new(exe)
        .args(["build", &deck.to_string_lossy(), "--size", sizes, "--out", &out.to_string_lossy(), "--preview", "none", "--state-out", &state.to_string_lossy()])
        .output()
        .map_err(|e| format!("falha ao rodar o build: {e}"))?;
    if state.join("state.json").is_file() {
        return Ok(());
    }
    let log = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    Err(crate::mcp::strip_ansi(log.trim()).replace("erro ", ""))
}

fn read_json(path: &Path) -> Value {
    std::fs::read_to_string(path).ok().and_then(|t| crate::js::parse_json(&t)).unwrap_or(Value::Undefined)
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Constrói os casos de um DS. Se o deck inteiro não avalia, cada caso é construído sozinho: o caso
/// que quebra fica marcado com o erro e os outros continuam no catálogo.
fn render_ds(entry: &mut Entry, out: &Path, sizes: &str, size_names: &[String]) -> Result<(), String> {
    let Ok(ds) = &entry.info else { return Ok(()) };
    let refs = case_refs(ds);
    let dir = out.join(&entry.id);
    let work = dir.join("deck");
    let import_from = crate::dsinfo::relative_path(&work, &ds.entry);
    write(&work.join(crate::config::CONFIG_FILE), &config_source(&import_from))?;
    let prompt = prompt_of(ds, &entry.spec, &entry.base);
    write(&dir.join("prompt.md"), &prompt)?;
    entry.prompt_file = Some(format!("{}/prompt.md", entry.id));
    if refs.is_empty() {
        return Ok(());
    }
    let deck = work.join(DECK_FILE);
    write(&deck, &deck_source(ds, &import_from, &refs))?;
    let state = dir.join("previews");
    match build_deck(&deck, &dir, &state, sizes) {
        Ok(()) => {
            entry.renders = renders_of(&read_json(&state.join("state.json")), &read_json(&state.join("diagnostics.json")), &format!("{}/previews", entry.id), refs.len());
            let stem = DECK_FILE.trim_end_matches(".tsx");
            entry.pptx = size_names.iter().map(|s| (s.clone(), format!("{}/{stem}.{s}.pptx", entry.id))).collect();
        }
        Err(_) => {
            entry.renders = refs
                .iter()
                .enumerate()
                .map(|(i, &at)| {
                    let one = work.join(format!("case-{}.tsx", i + 1));
                    let state = dir.join(format!("previews/case-{}", i + 1));
                    let built = write(&one, &deck_source(ds, &import_from, &[at])).and_then(|()| build_deck(&one, &work.join("out"), &state, sizes));
                    match built {
                        Ok(()) => renders_of(&read_json(&state.join("state.json")), &read_json(&state.join("diagnostics.json")), &format!("{}/previews/case-{}", entry.id, i + 1), 1).remove(0),
                        Err(e) => CaseRender { error: Some(e), ..CaseRender::default() },
                    }
                })
                .collect();
        }
    }
    Ok(())
}

fn svgs_of(entries: &[Entry], out: &Path) -> Vec<(String, String)> {
    entries
        .iter()
        .flat_map(|e| e.renders.iter())
        .flat_map(|r| r.previews.iter().flat_map(|(_, files)| files.iter()))
        .filter_map(|file| std::fs::read_to_string(out.join(file)).ok().map(|svg| (file.clone(), svg)))
        .collect()
}

fn count_line(entries: &[Entry]) -> String {
    let ok: Vec<&DsInfo> = entries.iter().filter_map(|e| e.info.as_ref().ok()).collect();
    let components: usize = ok.iter().map(|d| d.components.len()).sum();
    let cases: usize = ok.iter().map(|d| d.components.iter().map(|c| c.cases.len()).sum::<usize>()).sum();
    format!("{} design systems, {components} componentes, {cases} casos", entries.len())
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
    let (sources, registry) = match crate::registry::collect(args.registry.as_deref(), &args.ds, &cwd) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(1);
        }
    };
    if sources.is_empty() {
        eprintln!("\x1b[31merro\x1b[0m nenhum design system: passe --ds <caminho|pacote> ou registre com `pptsx-rs registry add <ds>` ({} nesta pasta ou acima)", crate::registry::REGISTRY_FILE);
        return ExitCode::from(1);
    }
    let sizes: Vec<crate::js::DeckCtx> = match args.sizes.split(',').map(str::trim).filter(|s| !s.is_empty()).map(crate::parse_size).collect() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(2);
        }
    };
    let size_names: Vec<String> = sizes.iter().map(|s| s.name.clone()).collect();
    let mut entries = load_all(&sources);
    if args.json {
        println!("{}", crate::mcp::pretty_json(&catalog_value(&entries, &sizes)));
        return ExitCode::SUCCESS;
    }
    if args.prompt {
        let prompts: Vec<String> = entries.iter().filter_map(|e| e.info.as_ref().ok().map(|ds| prompt_of(ds, &e.spec, &e.base))).collect();
        println!("{}", prompts.join("\n---\n\n"));
        return ExitCode::SUCCESS;
    }
    if let Some(file) = &registry {
        println!("  registro {}", file.display());
    }
    let out = if args.out.is_absolute() { args.out.clone() } else { cwd.join(&args.out) };
    let mut failed = false;
    for entry in &mut entries {
        if let Err(e) = render_ds(entry, &out, &args.sizes, &size_names) {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(1);
        }
        match &entry.info {
            Ok(ds) => {
                let broken = entry.renders.iter().filter(|r| r.error.is_some()).count();
                let errors = entry.renders.iter().filter(|r| r.diagnostics.iter().any(|d| d.severity == "error")).count();
                let mark = if broken + errors == 0 { "\x1b[32m✓\x1b[0m" } else { "\x1b[33m!\x1b[0m" };
                let extra = [(broken, "não renderizam"), (errors, "com error")].iter().filter(|(n, _)| *n > 0).map(|(n, what)| format!(", {n} {what}")).collect::<String>();
                println!("{mark} {:<16} {} {} · {} componentes, {} casos{extra}", entry.id, ds.name, ds.version, ds.components.len(), entry.renders.len());
                for note in notes_of(ds) {
                    println!("    \x1b[33mrecomendação\x1b[0m {note}");
                }
            }
            Err(e) => {
                failed = true;
                println!("\x1b[31m✗\x1b[0m {:<16} {}", entry.id, e);
            }
        }
    }
    let catalog = catalog_value(&entries, &sizes);
    let prompts: Vec<(String, String)> = entries.iter().filter_map(|e| e.info.as_ref().ok().map(|ds| (e.id.clone(), prompt_of(ds, &e.spec, &e.base)))).collect();
    let html = viewer_html(&catalog, &svgs_of(&entries, &out), &prompts);
    if let Err(e) = write(&out.join("catalog.json"), &format!("{}\n", crate::mcp::pretty_json(&catalog))).and_then(|()| write(&out.join("index.html"), &html)) {
        eprintln!("\x1b[31merro\x1b[0m {e}");
        return ExitCode::from(1);
    }
    println!("\x1b[32m✓\x1b[0m {} ({})", out.join("index.html").display(), count_line(&entries));
    if failed { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsinfo::DsCase;

    fn component(name: &str, cases: &[(&str, bool)], slide: bool) -> DsComponent {
        DsComponent {
            name: name.into(),
            display_name: None,
            guide: None,
            props: None,
            description: Some(format!("{name} de teste")),
            category: None,
            tags: Vec::new(),
            when_to_use: vec!["sempre".into()],
            avoid: Vec::new(),
            audience: vec!["executive".into()],
            kind: if slide { crate::dsinfo::Tier::Template } else { crate::dsinfo::Tier::Component },
            slide,
            cases: cases.iter().map(|(n, fill)| DsCase { name: (*n).into(), description: None, fill: *fill, audience: Vec::new(), props: Rc::new(Obj::new()) }).collect(),
        }
    }

    fn ds(components: Vec<DsComponent>) -> DsInfo {
        DsInfo {
            name: "teste-ds".into(),
            version: "1.0.0".into(),
            description: None,
            theme_colors: vec![("light1".into(), "#FFFFFF".into()), ("dark1".into(), "#111111".into())],
            import_from: "./ds.ts".into(),
            named: true,
            has_theme: true,
            background: None,
            ink: None,
            components,
            rules: Vec::new(),
            guide: None,
            schema: None,
            entry: PathBuf::from("/x/ds.ts"),
            by_path: true,
            ds_dir: None,
            tsconfig_extends: None,
        }
    }

    #[test]
    fn deck_has_one_slide_per_case_with_the_right_frame() {
        let info = ds(vec![component("Card", &[("a", false), ("b", true)], false), component("Cover", &[("capa", false)], true), component("Plain", &[], false)]);
        let refs = case_refs(&info);
        assert_eq!(refs, vec![CaseRef { component: 0, case: 0 }, CaseRef { component: 0, case: 1 }, CaseRef { component: 1, case: 0 }]);
        let deck = deck_source(&info, "../../ds.ts", &refs);
        assert!(deck.contains("import * as mod from \"../../ds.ts\";"), "{deck}");
        // Tamanho natural: coluna de grid; fill: direto no Slide; layout: direto no Deck, com ctx.
        assert!(deck.contains("<Box display=\"grid\" gridTemplateColumns=\"minmax(0, 1fr)\">{show(\"Card\", 0)}</Box>"), "{deck}");
        assert!(deck.contains("<Slide padding={36} backgroundColor=\"#FFFFFF\" color=\"#111111\">{show(\"Card\", 1)}</Slide>"), "{deck}");
        assert!(deck.contains("    {show(\"Cover\", 0, { ctx })}"), "{deck}");
        assert_eq!(deck.matches("show(\"").count(), 3);
    }

    #[test]
    fn renders_map_paginated_slides_and_diagnostics_back_to_the_case() {
        let state = crate::js::parse_json(
            r#"{ "sizes": [ { "size": "16x9", "slides": [ { "number": 1, "sourceSlide": 1 }, { "number": 2, "sourceSlide": 2 }, { "number": 3, "sourceSlide": 2 } ] } ] }"#,
        )
        .expect("json");
        let diags = crate::js::parse_json(
            r#"[ { "size": "16x9", "slide": 3, "severity": "error", "code": "text-overflow", "path": "Slide > Text", "message": "m" }, { "size": "16x9", "slide": 0, "severity": "warning", "code": "module-warning", "path": "", "message": "deck" } ]"#,
        )
        .expect("json");
        let r = renders_of(&state, &diags, "x/previews", 2);
        assert_eq!(r[0].previews, vec![("16x9".to_string(), vec!["x/previews/16x9/1.svg".to_string()])]);
        assert_eq!(r[1].previews[0].1, vec!["x/previews/16x9/2.svg".to_string(), "x/previews/16x9/3.svg".to_string()]);
        assert_eq!(r[1].diagnostics.len(), 1);
        assert_eq!(r[1].diagnostics[0].code, "text-overflow");
        assert!(r[0].diagnostics.is_empty(), "o diagnóstico do deck (slide 0) não é de um caso");
    }

    #[test]
    fn catalog_json_carries_the_contract_fields() {
        let info = ds(vec![component("Card", &[("a", false)], false)]);
        let entries = vec![
            Entry { id: "teste".into(), spec: "./ds.ts".into(), info: Ok(info), renders: Vec::new(), pptx: Vec::new(), prompt_file: None, description: None, base: PathBuf::from(".") },
            Entry { id: "quebrado".into(), spec: "./nada.ts".into(), info: Err("arquivo não encontrado".into()), renders: Vec::new(), pptx: Vec::new(), prompt_file: None, description: None, base: PathBuf::from(".") },
        ];
        let mut json = String::new();
        to_json(&catalog_value(&entries, &[crate::js::DeckCtx { name: "16x9".into(), width: 720.0, height: 405.0 }]), &mut json);
        assert!(json.starts_with(r#"{"format":"pptsx.catalog/1""#), "{json}");
        assert!(json.contains(r#""description":"Card de teste""#), "{json}");
        assert!(json.contains(r#""whenToUse":["sempre"]"#), "{json}");
        assert!(json.contains(r#""usage":"<Card />""#), "{json}");
        assert!(json.contains(r##""theme":{"colors":{"light1":"#FFFFFF","dark1":"#111111"}}"##), "{json}");
        assert!(json.contains(r#""error":"arquivo não encontrado""#), "{json}");
    }

    #[test]
    fn viewer_embeds_the_data_without_closing_the_script() {
        let html = viewer_html(&obj(vec![("format", str_val(FORMAT))]), &[("a/1.svg".into(), "<svg></svg>".into())], &[]);
        assert!(!html.contains(DATA_MARK));
        assert!(html.contains(r#""a/1.svg":"<svg><\/svg>""#));
    }
}
