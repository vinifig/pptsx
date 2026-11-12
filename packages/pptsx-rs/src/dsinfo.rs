//! Um design system visto pelo binário (`init --ds`, `skill --ds`, `mcp --ds`): o manifesto que
//! `createDs`/`defineDs` exportam (`ds` nomeado ou default), lido pelo avaliador do módulo do DS, e
//! o schema das props lido dos tipos (dsprops.rs). O módulo pode ser um caminho `.ts`/`.tsx`, uma
//! pasta ou um pacote em `node_modules`.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::js::value::{Obj, Value, to_json};

/// Um caso do catálogo (`meta.catalog[i]`): o nome, as props de exemplo e como ele ocupa o slide.
pub struct DsCase {
    pub name: String,
    /// Uma linha sobre o que o caso mostra (contrato do catálogo; opcional).
    pub description: Option<String>,
    /// `fill: true`: o componente preenche o corpo do slide (sem a coluna de tamanho natural).
    pub fill: bool,
    /// Para quem a variação foi pensada (`audience`, ex.: `executive`, `tactical`); vazio = o do componente.
    pub audience: Vec<String>,
    pub props: Rc<Obj>,
}

/// As camadas de um DS, da peça menor ao slide inteiro (`meta.kind`, docs/ds-catalog.md). O agente
/// monta um slide de cima para baixo: escolhe o template, preenche as estruturas com componentes, e os
/// componentes com widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// Peça atômica, sem filhos de outro componente do DS (um selo, um marcador, um número).
    Widget,
    /// Um bloco de conteúdo com sentido próprio (cartão de KPI, lista, citação).
    Component,
    /// Apresenta dados: gráfico, tabela, Gantt de roadmap, linha do tempo.
    Chart,
    /// Organiza outros componentes numa região do slide (colunas, grade, faixa).
    Structure,
    /// O slide inteiro (capa, conteúdo, encerramento): devolve o `<Slide>`.
    Template,
}

impl Tier {
    pub const ALL: [Tier; 5] = [Tier::Template, Tier::Structure, Tier::Chart, Tier::Component, Tier::Widget];

    pub fn name(self) -> &'static str {
        match self {
            Tier::Widget => "widget",
            Tier::Component => "component",
            Tier::Chart => "chart",
            Tier::Structure => "structure",
            Tier::Template => "template",
        }
    }

    /// O título da seção no prompt e no visualizador.
    pub fn title(self) -> &'static str {
        match self {
            Tier::Template => "Templates de slide",
            Tier::Structure => "Estruturas",
            Tier::Component => "Componentes",
            Tier::Chart => "Gráficos e dados",
            Tier::Widget => "Widgets",
        }
    }

    /// `meta.kind`; sem ele, `slide: true` (ou a tag `layout`, do ds-base anterior ao contrato) é template.
    fn of(meta: &Value) -> Tier {
        match get_str(meta, "kind").as_deref() {
            Some("widget") => return Tier::Widget,
            Some("structure") => return Tier::Structure,
            Some("template") => return Tier::Template,
            Some("chart") => return Tier::Chart,
            Some(_) => return Tier::Component,
            None => {}
        }
        if get(meta, "slide").is_some_and(Value::truthy) { Tier::Template } else { Tier::Component }
    }
}

pub struct DsComponent {
    pub name: String,
    pub display_name: Option<String>,
    pub guide: Option<String>,
    /// Props do 1º caso do catálogo (o exemplo do deck inicial e do ds.md).
    pub props: Option<Rc<Obj>>,
    /// Metadados de prompt e de catálogo (docs/ds-catalog.md): todos opcionais.
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub when_to_use: Vec<String>,
    pub avoid: Vec<String>,
    /// Públicos para os quais o componente serve (`executive`, `strategic`, `tactical`, `operational`…).
    pub audience: Vec<String>,
    /// A camada do componente no DS (`meta.kind`).
    pub kind: Tier,
    /// O componente devolve o `<Slide>` inteiro (todo `template`, ou `slide: true`): o catálogo o põe direto no Deck.
    pub slide: bool,
    pub cases: Vec<DsCase>,
}

pub struct DsInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    /// As cores do tema do arquivo (`theme.colors`), na ordem declarada, em `#RRGGBB`.
    pub theme_colors: Vec<(String, String)>,
    /// O especificador que o deck e a config importam (o pacote, ou um caminho relativo com extensão).
    pub import_from: String,
    /// `import { ds } from …` (nomeado) ou `import ds from …` (default).
    pub named: bool,
    pub has_theme: bool,
    /// `light1` e `dark1` do tema, para o fundo e o texto do deck inicial.
    pub background: Option<String>,
    pub ink: Option<String>,
    pub components: Vec<DsComponent>,
    pub rules: Vec<(String, String)>,
    pub guide: Option<String>,
    pub schema: Option<Value>,
    /// O arquivo de entrada do DS (canônico) e se ele veio de um caminho local (e não de um pacote).
    pub entry: PathBuf,
    pub by_path: bool,
    /// Pasta do DS relativa ao deck quando ele é um caminho `.ts`/`.tsx` fora da pasta (entra no `include`).
    pub ds_dir: Option<String>,
    /// `extends` do tsconfig quando o deck fica dentro do monorepo do pptsx.
    pub tsconfig_extends: Option<String>,
}

const DS_ENTRIES: [&str; 7] = ["index.ts", "index.tsx", "ds.ts", "index.js", "src/index.ts", "src/index.tsx", "src/ds.ts"];

fn get<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(o) => o.get(key).filter(|v| !v.is_nullish()),
        _ => None,
    }
}

fn get_str(v: &Value, key: &str) -> Option<String> {
    get(v, key).map(Value::to_js_string).filter(|s| !s.is_empty())
}

/// Caminho relativo com `/`, começando por `./` ou `../`.
/// O caminho absoluto e sem `.`/`..`, com os links simbólicos resolvidos até onde o caminho existe. Uma pasta
/// que o `init` ainda vai criar não existe para o `canonicalize`: sem isto, `init ../../decks/novo` contava os
/// `..` como pastas de verdade e o import do DS saía com níveis a mais.
fn resolved(path: &Path) -> PathBuf {
    if let Ok(real) = std::fs::canonicalize(path) {
        return real;
    }
    let absolute = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(path) };
    let mut clean = PathBuf::new();
    for part in absolute.components() {
        match part {
            std::path::Component::ParentDir => {
                clean.pop();
            }
            std::path::Component::CurDir => {}
            other => clean.push(other.as_os_str()),
        }
    }
    // O ancestral que já existe pode estar atrás de um link (/tmp → /private/tmp): resolve-o e recoloca o resto.
    let existing = clean.ancestors().find(|dir| dir.exists()).map(Path::to_path_buf).unwrap_or_default();
    let rest = clean.strip_prefix(&existing).map(Path::to_path_buf).unwrap_or_default();
    std::fs::canonicalize(&existing).unwrap_or(existing).join(rest)
}

/// Caminho relativo de `from` (pasta) até `to`, com `/`.
pub fn relative_path(from: &Path, to: &Path) -> String {
    relative_import(from, to)
}

fn relative_import(from: &Path, to: &Path) -> String {
    let (from, to) = (resolved(from), resolved(to));
    let f: Vec<_> = from.components().collect();
    let t: Vec<_> = to.components().collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = (common..f.len()).map(|_| "..".to_string()).collect();
    parts.extend(t[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()));
    let rel = parts.join("/");
    if rel.starts_with("..") { rel } else { format!("./{rel}") }
}

fn is_path_spec(spec: &str, cwd: &Path) -> bool {
    spec.starts_with('.') || Path::new(spec).is_absolute() || cwd.join(spec).exists()
}

/// Caminho → o arquivo de entrada (pasta: index.ts, ds.ts…); pacote → resolvido como o deck resolveria.
pub fn entry_of(spec: &str, cwd: &Path) -> Result<(PathBuf, bool), String> {
    if !is_path_spec(spec, cwd) {
        let file = crate::js::resolve_import(cwd, spec).ok_or_else(|| format!("--ds {spec}: pacote não encontrado em node_modules a partir de {}", cwd.display()))?;
        return Ok((file, false));
    }
    let target = if Path::new(spec).is_absolute() { PathBuf::from(spec) } else { cwd.join(spec) };
    if target.is_dir() {
        let file = DS_ENTRIES.iter().map(|e| target.join(e)).find(|p| p.is_file()).ok_or_else(|| format!("--ds {spec}: a pasta não tem {}", DS_ENTRIES.join(", ")))?;
        return Ok((file, true));
    }
    if !target.is_file() {
        return Err(format!("--ds {spec}: arquivo não encontrado"));
    }
    Ok((target, true))
}

fn monorepo_tsconfig(deck_dir: &Path) -> Option<String> {
    let mut dir = std::fs::canonicalize(deck_dir).ok()?;
    loop {
        let candidate = dir.join("tsconfig.monorepo.json");
        if candidate.is_file() {
            return Some(relative_import(deck_dir, &candidate));
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn strings_of(v: &Value, key: &str) -> Vec<String> {
    match get(v, key) {
        Some(Value::Array(list)) => list.borrow().iter().map(Value::to_js_string).filter(|s| !s.is_empty()).collect(),
        Some(Value::Str(s)) if !s.is_empty() => vec![s.to_string()],
        _ => Vec::new(),
    }
}

fn cases_of(meta: &Value) -> Vec<DsCase> {
    let Some(Value::Array(cases)) = get(meta, "catalog") else { return Vec::new() };
    cases
        .borrow()
        .iter()
        .enumerate()
        .filter_map(|(i, case)| {
            let Some(Value::Object(props)) = get(case, "props") else { return None };
            Some(DsCase {
                name: get_str(case, "name").unwrap_or_else(|| format!("caso {}", i + 1)),
                description: get_str(case, "description"),
                fill: get(case, "fill").is_some_and(Value::truthy),
                audience: strings_of(case, "audience"),
                props: props.clone(),
            })
        })
        .collect()
}

/// Os componentes do manifesto (`components`, os metas do `defineComponent`).
pub fn components_of(manifest: &Value) -> Vec<DsComponent> {
    let Some(Value::Array(list)) = get(manifest, "components") else { return Vec::new() };
    list.borrow()
        .iter()
        .filter_map(|meta| {
            let name = get_str(meta, "name")?;
            let cases = cases_of(meta);
            let kind = Tier::of(meta);
            Some(DsComponent {
                name,
                display_name: get_str(meta, "displayName"),
                guide: get_str(meta, "guide"),
                props: cases.first().map(|c| c.props.clone()),
                description: get_str(meta, "description"),
                category: get_str(meta, "category"),
                tags: strings_of(meta, "tags"),
                when_to_use: strings_of(meta, "whenToUse"),
                avoid: strings_of(meta, "avoid"),
                audience: strings_of(meta, "audience"),
                kind,
                slide: kind == Tier::Template || get(meta, "slide").is_some_and(Value::truthy),
                cases,
            })
        })
        .collect()
}

pub fn rules_of(manifest: &Value) -> Vec<(String, String)> {
    let Some(Value::Array(list)) = get(manifest, "rules") else { return Vec::new() };
    list.borrow().iter().filter_map(|r| Some((get_str(r, "id")?, get_str(r, "severity").unwrap_or_else(|| "warning".into())))).collect()
}

/// O manifesto entre os exports de um módulo de DS: `ds` nomeado (true) ou o default (false).
pub fn manifest_of(exports: &Obj) -> Option<(Value, bool)> {
    let named = |key: &str| exports.get(key).filter(|v| get_str(v, "name").is_some()).cloned();
    named("ds").map(|v| (v, true)).or_else(|| named("default").map(|v| (v, false)))
}

/// Carrega o módulo do DS e lê o manifesto. `deck_dir` é a pasta do deck que vai importá-lo.
pub fn load(spec: &str, deck_dir: &Path, cwd: &Path) -> Result<DsInfo, String> {
    let (file, by_path) = entry_of(spec, cwd)?;
    let (interp, exports) = crate::js::evaluate_exports(&file)?;
    let Some((manifest, named)) = manifest_of(&exports) else {
        return Err(format!("--ds {spec}: o módulo precisa exportar \"ds\" (ou default) um defineDs/createDs"));
    };
    let is_ts = file.extension().is_some_and(|e| e == "ts" || e == "tsx");
    let import_from = if by_path { relative_import(deck_dir, &file) } else { spec.to_string() };
    let inside = std::fs::canonicalize(&file).ok().zip(std::fs::canonicalize(deck_dir).ok()).is_some_and(|(f, d)| f.starts_with(&d));
    let ds_dir = (by_path && is_ts && !inside).then(|| file.parent().map(|p| relative_import(deck_dir, p)).unwrap_or_else(|| ".".into()));
    let theme = get(&manifest, "theme").and_then(crate::config::theme_of);
    let color = |slot: &str| theme.as_ref().and_then(|t| t.colors.iter().find(|(k, _)| k == slot).map(|(_, hex)| format!("#{hex}")));
    Ok(DsInfo {
        name: get_str(&manifest, "name").unwrap_or_else(|| spec.to_string()),
        version: get_str(&manifest, "version").unwrap_or_else(|| "0.0.0".into()),
        description: get_str(&manifest, "description"),
        theme_colors: theme.as_ref().map(|t| t.colors.iter().map(|(k, hex)| (k.clone(), format!("#{hex}"))).collect()).unwrap_or_default(),
        import_from,
        named,
        has_theme: theme.is_some(),
        background: color("light1"),
        ink: color("dark1"),
        components: components_of(&manifest),
        rules: rules_of(&manifest),
        guide: get_str(&manifest, "guide"),
        schema: crate::dsprops::schema_of(&interp, &manifest),
        entry: std::fs::canonicalize(&file).unwrap_or(file),
        by_path,
        ds_dir,
        tsconfig_extends: monorepo_tsconfig(deck_dir),
    })
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn is_safe_string(s: &str) -> bool {
    !s.chars().any(|c| matches!(c, '"' | '{' | '}' | '<' | '>' | '\\' | '\n'))
}

/// O componente em JSX com as props de um caso do catálogo (ds-scaffold.ts componentUsageOf): um
/// elemento (slot) vira `{<Text>nome</Text>}` para o autor trocar.
pub fn usage_of(component: &DsComponent) -> String {
    usage_with(&component.name, component.props.as_deref())
}

/// O JSX de um componente com estas props (um caso do catálogo).
pub fn usage_with(name: &str, props: Option<&Obj>) -> String {
    let mut attrs: Vec<String> = Vec::new();
    // `children` vai entre as tags: um texto simples como está, o resto como um `<Text>…</Text>` para o autor trocar.
    let children = props.and_then(|p| p.get("children")).filter(|v| !v.is_nullish()).map(|v| match v {
        Value::Str(s) if is_safe_string(s) => s.to_string(),
        _ => "<Text>…</Text>".to_string(),
    });
    if let Some(props) = props {
        for (name, value) in props.iter() {
            if !is_identifier(name) || &**name == "children" {
                continue;
            }
            let attr = match value {
                Value::Undefined => continue,
                Value::Element(_) => format!("{name}={{<Text>{name}</Text>}}"),
                Value::Str(s) if is_safe_string(s) => format!("{name}=\"{s}\""),
                Value::Bool(true) => name.to_string(),
                other => {
                    let mut json = String::new();
                    to_json(other, &mut json);
                    format!("{name}={{{json}}}")
                }
            };
            attrs.push(attr);
        }
    }
    let open = if attrs.is_empty() { format!("<{name}") } else { format!("<{name} {}", attrs.join(" ")) };
    match children {
        Some(body) => format!("{open}>{body}</{name}>"),
        None => format!("{open} />"),
    }
}

/// Nome do deck inicial que um DS pode trazer na própria raiz.
pub const STARTER_FILE: &str = "starter.tsx";

/// O deck inicial do próprio DS: `starter.tsx` na pasta da entrada ou na de cima (a raiz do pacote), com
/// `{{ds}}` trocado pelo especificador de import. Um DS com layouts de slide sabe começar um deck melhor do
/// que um catálogo em `<Slide>` cru, e o agente parte de um exemplo que segue as regras do DS.
fn starter_of(ds: &DsInfo) -> Option<String> {
    let own = ds.entry.parent()?;
    let template = [Some(own), own.parent()].into_iter().flatten().find_map(|dir| std::fs::read_to_string(dir.join(STARTER_FILE)).ok())?;
    Some(template.replace("{{ds}}", &ds.import_from))
}

/// Deck inicial com o DS: o `starter.tsx` dele, se houver; senão um slide por componente com caso de catálogo.
pub fn deck_content(ds: &DsInfo) -> String {
    if let Some(starter) = starter_of(ds) {
        return starter;
    }
    catalog_deck(ds)
}

/// Deck de catálogo (ds-scaffold.ts dsDeckContent): um slide por componente com caso de catálogo.
fn catalog_deck(ds: &DsInfo) -> String {
    let mut slide_props = vec!["padding={40}".to_string(), "gap={16}".to_string()];
    if let Some(bg) = &ds.background {
        slide_props.push(format!("backgroundColor=\"{bg}\""));
    }
    if let Some(ink) = &ds.ink {
        slide_props.push(format!("color=\"{ink}\""));
    }
    let slide_props = slide_props.join(" ");
    let shown: Vec<&DsComponent> = ds.components.iter().filter(|c| c.props.is_some()).collect();
    let names: Vec<&str> = shown.iter().map(|c| c.name.as_str()).collect();
    let slides: Vec<String> = if shown.is_empty() {
        vec![format!("    <Slide {slide_props}>\n      <Text fontSize={{24}}>Nenhum componente do {} tem caso de catálogo.</Text>\n    </Slide>", ds.name)]
    } else {
        shown
            .iter()
            .map(|c| {
                // Um template já é o slide inteiro: entra direto no Deck (ctx para os que medem o slide).
                if c.slide {
                    return format!("    {}", usage_of(c).replacen(&format!("<{}", c.name), &format!("<{} ctx={{ctx}}", c.name), 1));
                }
                format!("    <Slide {slide_props}>\n      <Text fontSize={{24}} fontWeight=\"bold\">{}</Text>\n      {}\n    </Slide>", c.name, usage_of(c))
            })
            .collect()
    };
    let mut out = vec![
        format!("// Deck criado por `pptsx-rs init --ds`: um slide por componente do {}, com as props do catálogo.", ds.name),
        "//   pptsx-rs build deck.tsx --size 16x9,4x3".to_string(),
        "//   pptsx-rs lint deck.tsx   (regras do núcleo + as do DS registrado no pptsx.config.ts)".to_string(),
        "import { Deck, Slide, Text, defineDeck } from 'pptsx';".to_string(),
    ];
    if !names.is_empty() {
        out.push(format!("import {{ {} }} from '{}';", names.join(", "), ds.import_from));
    }
    out.push(String::new());
    out.push(if shown.iter().any(|c| c.slide) { "export default defineDeck((ctx) => (".into() } else { "export default defineDeck(() => (".into() });
    out.push("  <Deck>".into());
    out.push(slides.join("\n\n"));
    out.push("  </Deck>".into());
    out.push("));".into());
    out.push(String::new());
    out.join("\n")
}

/// `pptsx.config.ts` que registra o DS (regras no lint, tema no arquivo).
pub fn config_content(ds: &DsInfo) -> String {
    let import_line = if ds.named { format!("import {{ ds }} from '{}';", ds.import_from) } else { format!("import ds from '{}';", ds.import_from) };
    let theme = if ds.has_theme { ", theme: ds.theme" } else { "" };
    format!(
        "import {{ defineConfig }} from 'pptsx';\n{import_line}\n\n// O {} no lint (regras) e no arquivo (tema). Outras opções: sizes, lang, fonts, textSafety, lineBreaks, embedFonts.\nexport default defineConfig({{ ds: [ds]{theme} }});\n",
        ds.name
    )
}

/// Onde o agente acha o DS, dito no `references/ds.md`: o especificador do import e como registrar.
pub struct DsLocation {
    /// O que vai no `from '…'` do deck (um pacote, um caminho, ou um caminho a completar).
    pub import_from: String,
    /// Frase sobre o `pptsx.config.ts` (onde está pronto, ou quem o cria).
    pub config_hint: String,
}

fn schema_of_component<'a>(ds: &'a DsInfo, name: &str) -> Option<&'a Value> {
    ds.schema.as_ref().and_then(|schema| get(schema, name))
}

fn prop_names(schema: &Value) -> Vec<String> {
    match get(schema, "properties") {
        Some(Value::Object(p)) => p.keys().map(|k| k.to_string()).collect(),
        _ => Vec::new(),
    }
}

fn required_names(schema: &Value) -> Vec<String> {
    match get(schema, "required") {
        Some(Value::Array(a)) => a.borrow().iter().map(Value::to_js_string).collect(),
        _ => Vec::new(),
    }
}

/// A assinatura numa linha: obrigatórias primeiro, opcionais com `?`, e os filhos quando há `children`.
pub fn signature_of(name: &str, schema: &Value) -> String {
    let required = required_names(schema);
    let names = prop_names(schema);
    let attrs = |wanted: bool| names.iter().filter(|n| n.as_str() != "children" && required.contains(n) == wanted).map(|n| if wanted { n.clone() } else { format!("{n}?") }).collect::<Vec<_>>();
    let all: Vec<String> = attrs(true).into_iter().chain(attrs(false)).collect();
    let open = if all.is_empty() { format!("<{name}") } else { format!("<{name} {}", all.join(" ")) };
    if names.iter().any(|n| n == "children") { format!("{open}>…</{name}>") } else { format!("{open} />") }
}

/// O que as props herdam e o binário não abriu (`x-extends` do schema lido dos tipos).
fn extends_line(schema: &Value) -> Option<String> {
    let Some(Value::Array(list)) = get(schema, "x-extends") else { return None };
    let names: Vec<String> = list.borrow().iter().map(|v| format!("`{}`", v.to_js_string())).collect();
    (!names.is_empty()).then(|| format!("Aceita também as props de {}.", names.join(", ")))
}

fn component_section(ds: &DsInfo, c: &DsComponent, props_table: &impl Fn(&Value) -> String) -> Vec<String> {
    let mut out = vec![format!("### {}", c.display_name.as_deref().unwrap_or(&c.name)), String::new()];
    if let Some(d) = &c.description {
        out.push(d.trim().to_string());
        out.push(String::new());
    }
    if !c.audience.is_empty() {
        out.push(format!("Público: {}.", c.audience.join(", ")));
        out.push(String::new());
    }
    // Variações por público: o agente escolhe o caso que casa com a plateia do deck.
    let variants: Vec<String> = c.cases.iter().filter(|k| !k.audience.is_empty()).map(|k| format!("- `{}` ({}): `{}`", k.name, k.audience.join(", "), usage_with(&c.name, Some(&k.props)))).collect();
    if !variants.is_empty() {
        out.push("Variações por público:".into());
        out.extend(variants);
        out.push(String::new());
    }
    // As listas de prompt (`whenToUse`/`avoid`) vêm antes do guia: são o que o agente decide primeiro.
    for (title, items) in [("Use quando", &c.when_to_use), ("Evite", &c.avoid)] {
        if items.is_empty() {
            continue;
        }
        out.push(format!("{title}:"));
        out.extend(items.iter().map(|item| format!("- {}", item.trim())));
        out.push(String::new());
    }
    if let Some(g) = &c.guide {
        out.push(g.trim().to_string());
        out.push(String::new());
    }
    let schema = schema_of_component(ds, &c.name);
    if let Some(schema) = schema {
        out.push(format!("Assinatura: `{}`", signature_of(&c.name, schema)));
        out.push(String::new());
    }
    if c.props.is_some() {
        out.push("```tsx".into());
        out.push(usage_of(c));
        out.push("```".into());
        out.push(String::new());
    }
    let Some(schema) = schema else { return out };
    if !prop_names(schema).is_empty() {
        out.push(props_table(schema).trim_end().to_string());
        out.push(String::new());
    }
    if let Some(line) = extends_line(schema) {
        out.push(line);
        out.push(String::new());
    }
    out
}

/// `references/ds.md` da skill: import, cada componente com assinatura, exemplo e props (dos tipos do
/// DS, mesmo sem catálogo), as regras e o guia. `props_table` desenha a tabela de um componente.
pub fn markdown(ds: &DsInfo, location: &DsLocation, props_table: impl Fn(&Value) -> String) -> String {
    let mut out = vec![format!("# {} {}", ds.name, ds.version), String::new()];
    if let Some(d) = &ds.description {
        out.push(d.trim().to_string());
        out.push(String::new());
    }
    let first: Vec<&str> = ds.components.iter().take(3).map(|c| c.name.as_str()).collect();
    out.push(format!(
        "Importe de `{}` (`import {{ {} }} from '{}';`) e registre o DS no `pptsx.config.ts` da pasta do deck ({}).",
        location.import_from,
        first.join(", "),
        location.import_from,
        location.config_hint
    ));
    out.push(String::new());
    out.push("## Componentes".into());
    out.push(String::new());
    out.push("`*` = prop obrigatória. O tipo é o do TypeScript do DS; a nota traz a descrição, o padrão e exemplos do catálogo.".into());
    out.push(String::new());
    // Por camada, do slide inteiro à peça menor: é a ordem em que o agente decide. Um DS só de
    // componentes sai sem subtítulos, como antes do contrato.
    let tiers: Vec<Tier> = Tier::ALL.into_iter().filter(|t| ds.components.iter().any(|c| c.kind == *t)).collect();
    if !tiers.contains(&Tier::Template) && tiers.len() > 1 {
        out.push("Este DS não traz templates de slide: monte cada slide com `<Slide>` e as estruturas abaixo.".into());
        out.push(String::new());
    }
    for tier in &tiers {
        if tiers.len() > 1 || *tier != Tier::Component {
            out.push(format!("#### {} (`kind: \"{}\"`)", tier.title(), tier.name()));
            out.push(String::new());
        }
        for c in ds.components.iter().filter(|c| c.kind == *tier) {
            out.extend(component_section(ds, c, &props_table));
        }
    }
    if !ds.rules.is_empty() {
        out.push("## Regras de lint".into());
        out.push(String::new());
        out.push("O `lint` e o `build` apontam estas regras do DS, com o caminho do nó:".into());
        out.push(String::new());
        out.extend(ds.rules.iter().map(|(id, sev)| format!("- `{id}` ({sev})")));
        out.push(String::new());
    }
    if let Some(guide) = &ds.guide {
        out.push("## Guia do design system".into());
        out.push(String::new());
        let body = guide.strip_prefix('#').map_or(guide.as_str(), |rest| rest.split_once('\n').map_or("", |(_, b)| b));
        out.push(body.trim().to_string());
        out.push(String::new());
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_import_resolves_parent_segments_of_a_folder_that_does_not_exist_yet() {
        let base = std::env::temp_dir().join(format!("pptsx-rel-{}", std::process::id()));
        std::fs::create_dir_all(base.join("a/b")).unwrap();
        std::fs::create_dir_all(base.join("ds/src")).unwrap();
        std::fs::write(base.join("ds/src/index.ts"), "").unwrap();
        // A pasta do deck ainda não existe e chega com `..`, como em `init ../../out/novo` rodado de a/b.
        let deck_dir = base.join("a/b/../../out/novo");
        assert_eq!(relative_import(&deck_dir, &base.join("ds/src/index.ts")), "../../ds/src/index.ts");
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn usage_puts_children_between_the_tags() {
        let mut props = Obj::new();
        props.insert(Rc::from("title"), crate::js::value::str_val("Resultado"));
        props.insert(Rc::from("children"), crate::js::value::str_val("corpo"));
        assert_eq!(usage_with("Content", Some(&props)), "<Content title=\"Resultado\">corpo</Content>");
        props.insert(Rc::from("children"), crate::js::value::arr_val(vec![]));
        assert_eq!(usage_with("Content", Some(&props)), "<Content title=\"Resultado\"><Text>…</Text></Content>");
        assert_eq!(usage_with("Divider", None), "<Divider />");
    }

    #[test]
    fn tier_reads_kind_and_falls_back_to_slide() {
        let parse = |s: &str| crate::js::parse_json(s).expect("json");
        assert_eq!(Tier::of(&parse(r#"{ "kind": "chart" }"#)), Tier::Chart);
        assert_eq!(Tier::of(&parse(r#"{ "slide": true }"#)), Tier::Template);
        assert_eq!(Tier::of(&parse(r#"{ "kind": "outra" }"#)), Tier::Component);
        assert_eq!(Tier::of(&parse(r#"{}"#)), Tier::Component);
    }

    #[test]
    fn signature_lists_required_first_and_marks_optional() {
        let schema = crate::js::parse_json(r#"{ "properties": { "kicker": {}, "title": {}, "children": {} }, "required": ["title"] }"#).expect("json");
        assert_eq!(signature_of("Content", &schema), "<Content title kicker?>…</Content>");
        let empty = crate::js::parse_json(r#"{ "properties": {}, "required": [] }"#).expect("json");
        assert_eq!(signature_of("Divider", &empty), "<Divider />");
    }
}
