//! `pptsx-rs skill [--out <pasta>] [--name <nome>] [--description <texto>] [--ds <caminho|pacote>]
//! [--bin <binário> | --no-bin]`: uma skill para agentes (formato Agent Skills: `SKILL.md`,
//! `references/`), gerada do guia e do schema embutidos, com o próprio binário em `bin/`. É o que a
//! release empacota: descompactou em `.github/skills/` (Copilot) ou `.claude/skills/` (Claude Code), o
//! agente se vira. O `SKILL.md` é curto de propósito (economia de tokens): o essencial fica em
//! `references/essentials.md` e o guia inteiro, com índice de linhas, em `references/guide.md`.
//!
//! Com `--ds <caminho>` a skill é **autocontida**: o código do DS é copiado para `ds/` e os textos
//! apontam para `<pasta-desta-skill>/ds/…` (binário + DS, sem Node). O DS pode trazer o conteúdo
//! próprio da skill numa pasta `skill/` na raiz dele: `skill/SKILL.md` substitui o genérico (o
//! frontmatter dele dá o nome e a descrição padrão; `{{bin}}`, `{{ds}}`, `{{name}}`, `{{version}}`,
//! `{{ds_name}}` e `{{ds_version}}` são trocados) e `skill/references/*.md` entram em `references/`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::js::value::{Value, to_json};

/// Nome padrão da pasta, da skill e do servidor MCP (`--name` troca).
pub const NAME: &str = "pptsx";
const SKILL_ROOT: &str = "<pasta-desta-skill>";
const BIN: &str = "<pasta-desta-skill>/bin/pptsx-rs";
const DEFAULT_DESCRIPTION: &str = "Cria apresentações PowerPoint (.pptx) editáveis escrevendo os slides em TSX com layout automático (CSS como props, flexbox e grid), com PNG de cada slide para conferir. Use quando pedirem apresentação, slides, deck, pptx, PowerPoint, Keynote ou Google Slides.";
/// Pasta, na raiz do DS, com o conteúdo próprio da skill (`SKILL.md`, `references/*.md`).
const DS_SKILL_DIR: &str = "skill";
/// Onde o código do DS fica dentro da skill gerada.
const DS_BUNDLE_DIR: &str = "ds";
/// Pastas que não são código do DS: não entram na cópia.
const BUNDLE_SKIP: [&str; 3] = ["node_modules", "dist", "target"];
const MAX_NAME: usize = 64;

/// Nome e descrição da skill (o frontmatter do `SKILL.md`).
struct Identity {
    name: String,
    description: String,
}

/// O DS da skill: o manifesto e, quando ele veio de um caminho local, a entrada dele dentro da
/// skill (`ds/src/index.ts`).
struct SkillDs<'a> {
    info: &'a crate::dsinfo::DsInfo,
    bundled_entry: Option<String>,
    /// O `--ds` como foi passado (o pacote, quando não é caminho).
    spec: &'a str,
}

impl SkillDs<'_> {
    /// O DS como o agente o encontra: dentro da skill, ou o pacote.
    fn location(&self) -> String {
        match &self.bundled_entry {
            Some(entry) => format!("{SKILL_ROOT}/{entry}"),
            None => self.info.import_from.clone(),
        }
    }
}

const MODEL_DECK: &str = r##"```tsx
import { Box, Deck, Slide, Text, defineDeck } from 'pptsx';

export default defineDeck(() => (
  <Deck>
    <Slide padding={40} gap={16} backgroundColor="#FFFFFF" color="#1F2933">
      <Text fontSize={32} fontWeight="bold">Resultados do trimestre</Text>
      <Box display="grid" gridTemplateColumns={{ base: 'repeat(3, minmax(0, 1fr))', tall: 'minmax(0, 1fr)' }} gap={16} flex={1}>
        <Box backgroundColor="#EEF2FF" borderRadius={8} padding={16} gap={6}>
          <Text fontSize={14} color="#52606D">Receita</Text>
          <Text fontSize={28} fontWeight="bold">R$ 52,4 mi</Text>
        </Box>
      </Box>
      <Text fontSize={11} color="#52606D" marginTop="auto">Fonte: relatório trimestral</Text>
    </Slide>
  </Deck>
));
```"##;

/// Escalar YAML de uma linha: entre aspas quando o texto puro quebraria o frontmatter.
fn yaml_scalar(text: &str) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let plain = !line.is_empty() && !line.contains(": ") && !line.contains(" #") && !line.ends_with(':') && line.chars().next().is_some_and(|c| c.is_alphanumeric());
    if plain { line } else { format!("\"{}\"", line.replace('\\', "\\\\").replace('"', "\\\"")) }
}

fn frontmatter(identity: &Identity) -> String {
    format!("---\nname: {}\ndescription: {}\n---\n", identity.name, yaml_scalar(&identity.description))
}

fn ds_section(ds: &SkillDs) -> String {
    let name = &ds.info.name;
    let location = ds.location();
    let where_it_is = match &ds.bundled_entry {
        Some(_) => format!("Ele vem dentro desta skill, em `{SKILL_ROOT}/{DS_BUNDLE_DIR}/` (entrada: `{location}`): no deck, importe-o pelo caminho relativo da pasta do deck até lá. `{BIN} init <pasta> --ds {location}` cria um deck que já o importa e o `pptsx.config.ts` que o registra (regras no lint, tema no arquivo)."),
        None => format!("Ele é o pacote `{location}`: precisa estar no `node_modules` da pasta do deck. `references/pptsx.config.ts` é a config que o registra (regras no lint, tema no arquivo): copie para a pasta do deck. `{BIN} init <pasta> --ds {location}` cria um deck que já o importa e registra."),
    };
    format!("## Design system: {name}\n\nEsta skill usa o DS **{name}**. {where_it_is} Prefira os componentes dele às primitivas: `references/ds.md` tem cada componente com a assinatura, um exemplo, as props (com tipo, padrão e descrição) e as regras.\n\n")
}

fn skill_md(version: &str, identity: &Identity, ds: Option<&SkillDs>, extra_references: &[String]) -> String {
    let ds_section = ds.map(ds_section).unwrap_or_default();
    let mut ds_refs = String::new();
    if let Some(ds) = ds {
        ds_refs.push_str("- `references/ds.md`: os componentes, as props e as regras do design system.\n");
        if ds.bundled_entry.is_none() {
            ds_refs.push_str("- `references/pptsx.config.ts`: a config que registra o design system.\n");
        }
    }
    for name in extra_references {
        ds_refs.push_str(&format!("- `references/{name}`: do design system.\n"));
    }
    let front = frontmatter(identity);
    let name = &identity.name;
    let title = if name == NAME { format!("pptsx {version}: apresentações em TSX") } else { format!("{name}: apresentações em TSX (pptsx {version})") };
    format!(
        r#"{front}
# {title}

Você descreve a **estrutura** dos slides num `.tsx`; o pptsx calcula o layout, mede o texto com a fonte real e grava um `.pptx` editável, os PNGs de cada slide e os diagnósticos. Não existem coordenadas x/y.

O motor é o binário desta skill, sem Node nem instalação: `{BIN}` (Windows: `bin\pptsx-rs.exe`). Se o servidor MCP `{name}` estiver conectado, prefira as ferramentas dele (`diagnose`, `build`, `preview`, `lint`, `guide`, `schema`) ao shell; é o mesmo binário (`bin/pptsx-rs mcp`; os `mcp/*.json` prontos estão nesta pasta).

## Fluxo (meta: entregar em até 2 builds)

1. **Pasta do deck:** `{BIN} init <pasta>` cria `deck.tsx` de exemplo, `tsconfig.json` e `pptsx.config.ts`. Ou escreva só o `deck.tsx` como no modelo abaixo (a config só é precisa para fontes próprias, tema ou um design system).
2. **Escreva os slides** seguindo as regras abaixo. Antes do primeiro deck, leia `references/essentials.md` (≈ 420 linhas: primitivas, `Container`, texto, paginação, diagnósticos e armadilhas). O resto está em `references/guide.md` (o índice no topo dá a linha de cada seção; §6 receitas) e `references/props.md` (todas as props, com o tipo).
3. **Itere com** `{BIN} check <pasta>/deck.tsx --json` (só diagnósticos, sem gravar) até zerar os `error`.
4. **Compile:** `{BIN} build <pasta>/deck.tsx --size 16x9,4x3` (`--size 16x9` se pedirem um formato só). Sai em `dist/`: `.pptx` por tamanho, `overview.png` (todos os slides numa grade, selo vermelho onde há error), `slide-NN.png` e `diagnostics.json`.
5. **Leia os diagnósticos:** cada um tem o slide, o caminho do nó (`Slide > Box[1] > Text` é o Text do 2º Box do slide), a mensagem com o conserto e, num estouro, `causedBy`. Abra o overview PNG para ver o layout.
6. **Entregue** o `.pptx` e conte o que sobrou de diagnóstico. Código de saída 1 = ainda há `error`.

## Modelo

{MODEL_DECK}

## Regras que evitam o segundo build

- `export default defineDeck((ctx) => <Deck>…</Deck>)`; os filhos do `Deck` são só `Slide`.
- **Todo texto dentro de `<Text>`** (string solta num Box é erro). Negrito e itálico no meio do texto: `<B>`, `<I>`, `<Span color=…>` dentro do Text.
- O layout é **CSS em camelCase** como props: `padding`, `gap`, `margin`, `width`, `height`, `flex`, `alignItems`, `justifyContent`, `backgroundColor`, `border`, `borderRadius`, `fontSize`, `fontWeight`, `color`, `textAlign`, `lineHeight`. Números são pt (o slide 16:9 tem 720 × 405 pt; o 4:3, 720 × 540).
- `Box` sem `display` é **coluna**; `display="flex"` é **linha**. Colunas iguais: `display="grid" gridTemplateColumns="repeat(3, minmax(0, 1fr))"` (sempre `minmax(0, 1fr)`, nunca `1fr` solto) ou `flex={{1}}` em cada filho de uma linha.
- Ocupar a altura que sobra: `flex={{1}}`. Rodapé no fim: `marginTop="auto"` no último filho.
- Fonte e cor declaradas no `Slide` ou num `Box` **descem** para os `Text` de dentro: defina uma vez e escreva só o que muda.
- O 4:3 é mais **alto** que o 16:9 (mesma largura). Valor por formato: `{{ base: …, tall: … }}` (`tall` = 4:3; o 16:10 cai no `base`), nunca `if` sobre o formato.
- Lista ou tabela longa: `<Box paginate flex={{1}}>` (ou `Table paginate`) divide o conteúdo em vários slides. Texto que não cabe encolhe até um mínimo e, depois disso, vira `error` com o conserto.
- Gráfico nativo: `<Chart kind="bar" data={{{{ categories: [...], series: [{{ name, values: [...] }}] }}}} labels flex={{1}} />` (precisa de altura: `flex={{1}}` ou `height`). Tabela: `<Table columns={{[{{ header: "Área" }}, {{ header: "Q3", align: "right" }}]}} rows={{[["Varejo", "22,6"]]}} />` (números como texto mantêm a vírgula).
- Imagem: `<Image src="./logo.png" width={{120}} />`, com o caminho relativo ao deck.
- Degradê é nativo: `backgroundImage="linear-gradient(135deg, #2A2F9E, #0B1020)"` em Slide, Box e Shape. O que o PPTX não tem (`overflow: hidden`, `filter`…) vira o aviso `css-not-emitted`: troque por algo que exista. Não invente props; na dúvida, `references/props.md`.
- Design system: é um módulo TS (uma pasta ou um pacote) que exporta `ds`. Importe os componentes dele, registre no `pptsx.config.ts` (`defineConfig({{ ds: [ds] }})`) e prefira-os às primitivas; `{BIN} lint deck.tsx` aplica as regras dele. `{BIN} init <pasta> --ds <caminho|pacote>` cria o deck já com os componentes. O `@pptsx/ds-kit` (o kit de quem escreve um DS) vem dentro do binário: não precisa de `npm install`.
- Tipos: o `init` grava os `.d.ts` do pptsx em `.pptsx/types/` e aponta o `tsconfig.json` para lá; o editor entende `pptsx` e `@pptsx/ds-kit` sem `npm install`. Se o editor não achar `pptsx`, `{BIN} types <pasta>`.
- Lendo diagnósticos: `data.component` diz o componente do autor e as props dele (`<Risks items=[4 itens] status="warn" />`) quando o nó estourado fica dentro de um componente; é ali que se mexe, não no path interno. O `lint` não mostra `off-theme-color` de cores do design system (só com `--verbose`): o que aparece é seu.
- `module-warning` (slide 0) é um `console.warn` do deck ou do DS, como um ícone com nome desconhecido: corrija o que ele diz.

{ds_section}## Referências

- `references/essentials.md`: o essencial do guia; leia inteiro antes do primeiro deck.
- `references/guide.md`: o guia completo (primitivas, texto, receitas, diagnósticos, armadilhas), com índice de linhas.
- `references/props.md`: as props de cada primitiva, com o tipo.
{ds_refs}- `mcp/vscode.mcp.json` e `mcp/claude.mcp.json`: a entrada do servidor MCP para copiar em `.vscode/mcp.json` ou `.mcp.json`.
"#
    )
}

/// O guia com um índice com a linha de cada seção: ele é longo demais para uma leitura só, e o
/// índice deixa ler só a seção que importa.
fn guide_reference(guide: &str) -> String {
    let lines: Vec<&str> = guide.split('\n').collect();
    let headings: Vec<(usize, &str)> = lines.iter().enumerate().filter(|(_, l)| l.starts_with("## ") || l.starts_with("### ")).map(|(i, l)| (i, *l)).collect();
    let header = |offset: usize| -> Vec<String> {
        let mut h = vec!["**Índice** (linha de cada seção neste arquivo; leia só a que precisar):".to_string(), String::new()];
        h.extend(headings.iter().map(|(i, l)| format!("- linha {}: {}", i + 1 + offset, l.trim_start_matches('#').trim_start())));
        h.push(String::new());
        h
    };
    let offset = header(0).len();
    let mut out = header(offset);
    out.extend(lines.iter().map(|l| (*l).to_string()));
    out.join("\n")
}

/// Seções do guia que entram no essencial: a `##` e quais `###` dela (None = todas; vazio = só a introdução).
const ESSENTIAL_SECTIONS: [(&str, Option<&[&str]>); 6] = [
    ("## 1.", None),
    ("## 2.", Some(&["### Breakpoints"])),
    ("## 3.", Some(&["### Por primitiva", "### Paginação", "### Regras de flex e grid", "### Container"])),
    ("## 4.", Some(&["### Rich text", "### NBSP"])),
    ("## 8.", Some(&["### Como ler o `path`", "### Códigos", "### Checklist"])),
    ("## 9.", None),
];

/// O essencial: as seções acima, na ordem do guia. Um `###` entra se a sua `##` pede todas ou se bate
/// com um dos prefixos; a introdução da `##` (antes do 1º `###`) entra sempre.
fn essentials(guide: &str) -> String {
    let mut out = vec![
        "# pptsx: o essencial".to_string(),
        String::new(),
        "> Gerado do guia completo (`references/guide.md`): só o que um agente precisa antes do primeiro deck. Receitas (§6), gráficos, tabelas, diagramas, texto rico avançado, tema do arquivo e design systems ficam no guia; o índice no topo dele dá a linha de cada seção.".into(),
        String::new(),
    ];
    let mut rule: Option<Option<&[&str]>> = None;
    let mut keep = false;
    for line in guide.split('\n') {
        if line.starts_with("## ") {
            rule = ESSENTIAL_SECTIONS.iter().find(|(prefix, _)| line.starts_with(prefix)).map(|(_, subs)| *subs);
            keep = rule.is_some();
        } else if line.starts_with("### ") {
            keep = match rule {
                None => false,
                Some(None) => true,
                Some(Some(subs)) => subs.iter().any(|s| line.starts_with(s)),
            };
        }
        if keep {
            out.push(line.to_string());
        }
    }
    let text = out.join("\n");
    // Sem três quebras seguidas (as seções puladas deixam linhas vazias).
    let mut compact = String::with_capacity(text.len());
    let mut blank = 0;
    for l in text.split('\n') {
        if l.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        compact.push_str(l);
        compact.push('\n');
    }
    compact
}

const PRIMITIVE_TITLES: [(&str, &str); 14] = [
    ("deck", "Deck"),
    ("slide", "Slide"),
    ("box", "Box"),
    ("text", "Text"),
    ("span", "Span (B, I, Sup, Sub)"),
    ("image", "Image"),
    ("shape", "Shape"),
    ("chart", "Chart"),
    ("table", "Table"),
    ("path", "Path"),
    ("icon", "Icon"),
    ("connector", "Connector"),
    ("slideNumber", "SlideNumber"),
    ("container", "Container"),
];

const MAX_TYPE: usize = 140;

fn str_of(v: &Value, key: &str) -> Option<String> {
    match v {
        Value::Object(o) => match o.get(key) {
            Some(Value::Str(s)) => Some(s.to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// O responsivo vale para toda prop e repete o tipo inteiro: a tabela mostra só o valor fixo.
fn short_type(ty: &str) -> String {
    let fixed = ty.find("| ResponsiveValue<").map_or(ty, |i| &ty[..i]);
    let fixed: String = fixed.split_whitespace().collect::<Vec<_>>().join(" ");
    if fixed.chars().count() <= MAX_TYPE { fixed } else { format!("{}…", fixed.chars().take(MAX_TYPE - 1).collect::<String>()) }
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").split('\n').filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
}

/// A 1ª frase basta na tabela; o guia tem o resto. Ponto, interrogação ou exclamação só fecham a frase fora
/// de crases, aspas e parênteses e quando o que vem depois começa com maiúscula (ou é o fim): "vs. Q2",
/// `{ text? }` e ("R$ 52,4 mi") não cortam a nota no meio.
fn first_sentence(text: &str) -> String {
    let t = text.trim();
    let (mut depth, mut code, mut quoted) = (0usize, false, false);
    for (i, c) in t.char_indices() {
        match c {
            '`' => code = !code,
            '"' if !code => quoted = !quoted,
            '(' if !code && !quoted => depth += 1,
            ')' if !code && !quoted => depth = depth.saturating_sub(1),
            '.' | '!' | '?' if !code && !quoted && depth == 0 => {
                let rest = &t[i + c.len_utf8()..];
                let starts_sentence = rest.trim_start().chars().next().is_none_or(char::is_uppercase);
                if starts_sentence && rest.chars().next().is_none_or(char::is_whitespace) {
                    return t[..i + c.len_utf8()].trim().to_string();
                }
            }
            _ => {}
        }
    }
    t.to_string()
}

fn examples_note(prop: &Value) -> String {
    let Value::Object(p) = prop else { return String::new() };
    let Some(Value::Array(a)) = p.get("examples") else { return String::new() };
    if a.borrow().is_empty() {
        return String::new();
    }
    let items: Vec<String> = a
        .borrow()
        .iter()
        .map(|v| {
            let mut s = String::new();
            to_json(v, &mut s);
            format!("`{s}`")
        })
        .collect();
    format!(" Ex.: {}.", items.join(", "))
}

/// A nota de uma prop: a 1ª frase da descrição, o padrão (dos tipos de um DS) e os exemplos.
fn prop_note(prop: &Value) -> String {
    let description = str_of(prop, "description").map(|d| first_sentence(&d)).unwrap_or_default();
    let default = str_of(prop, "default").map(|d| format!(" Padrão: `{d}`.")).unwrap_or_default();
    format!("{description}{default}{}", examples_note(prop)).trim().to_string()
}

/// A tabela `| prop | tipo | nota |` de um schema `{ properties, required }`. `sorted` ordena por
/// nome (as primitivas têm dezenas de props); sem ele vale a ordem da declaração (um DS).
fn props_table(schema: &Value, sorted: bool) -> String {
    let Value::Object(o) = schema else { return String::new() };
    let required: Vec<String> = match o.get("required") {
        Some(Value::Array(a)) => a.borrow().iter().map(Value::to_js_string).collect(),
        _ => Vec::new(),
    };
    let mut props: Vec<(String, Value)> = match o.get("properties") {
        Some(Value::Object(p)) => p.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        _ => Vec::new(),
    };
    if sorted {
        props.sort_by(|a, b| a.0.cmp(&b.0));
    }
    let mut out = String::from("| prop | tipo | nota |\n|---|---|---|\n");
    for (name, prop) in props {
        let star = if required.contains(&name) { " *" } else { "" };
        out.push_str(&format!("| `{name}`{star} | `{}` | {} |\n", cell(&short_type(&str_of(&prop, "type").unwrap_or_else(|| "unknown".into()))), cell(&prop_note(&prop))));
    }
    out
}

fn props_section(title: &str, schema: &Value) -> String {
    format!("## {title}\n\n{}\n", props_table(schema, true))
}

/// Tabelas de props de um schema `{ nome: { properties, required } }` com um título.
fn props_sections(heading: &str, schema: &Value) -> String {
    let mut out = format!("# {heading}\n\nGerado do schema do pptsx (o mesmo do `schema()` no MCP). `*` = obrigatória. Toda prop aceita também um valor por breakpoint, `{{ base, tall, wide }}` (guia §2).\n\n");
    if let Value::Object(o) = schema {
        for (key, value) in o.iter() {
            let title = PRIMITIVE_TITLES.iter().find(|(k, _)| *k == &**key).map_or(key.to_string(), |(_, t)| (*t).to_string());
            out.push_str(&props_section(&title, value));
        }
    }
    out
}

/// `references/props.md`, gerado do schema embutido (o mesmo do `schema()` do MCP), nunca à mão.
fn props_md(schema: &Value) -> String {
    props_sections("Props das primitivas do pptsx", schema)
}

/// A entrada do servidor MCP: o binário da skill e, com um DS, `--ds` apontando para ele.
fn mcp_json(name: &str, bin: &str, ds: Option<&str>, vscode: bool) -> String {
    let quoted = |text: &str| {
        let mut out = String::new();
        to_json(&Value::Str(text.into()), &mut out);
        out
    };
    let mut args = vec![quoted("mcp")];
    if let Some(ds) = ds {
        args.push(quoted("--ds"));
        args.push(quoted(ds));
    }
    let args = args.join(", ");
    if vscode {
        format!("{{\n  \"servers\": {{\n    {}: {{\n      \"type\": \"stdio\",\n      \"command\": {},\n      \"args\": [{args}]\n    }}\n  }}\n}}\n", quoted(name), quoted(&format!("${{workspaceFolder}}/{bin}")))
    } else {
        format!("{{\n  \"mcpServers\": {{\n    {}: {{\n      \"command\": {},\n      \"args\": [{args}]\n    }}\n  }}\n}}\n", quoted(name), quoted(bin))
    }
}

fn readme(version: &str, name: &str, ds: Option<&SkillDs>) -> String {
    let ds_line = match ds {
        Some(SkillDs { info, bundled_entry: Some(entry), .. }) => format!(" O design system **{}** vem junto, em `{entry}`.", info.name),
        _ => String::new(),
    };
    format!(
        r#"# Skill {name} (pptsx-rs {version})

Uma skill de apresentações para agentes, com o motor dentro (`bin/pptsx-rs`): nada de Node, nada para instalar.{ds_line}

1. Mova esta pasta inteira para a pasta de skills do repositório em que o agente vai trabalhar:
   - **GitHub Copilot no VS Code (agent mode):** `.github/skills/{name}/`
   - **Claude Code:** `.claude/skills/{name}/` (ou `~/.claude/skills/{name}/` para todos os projetos)
2. Para o agente usar as ferramentas MCP em vez do shell, copie `mcp/vscode.mcp.json` para `.vscode/mcp.json` ou `mcp/claude.mcp.json` para `.mcp.json` na raiz do repositório (o caminho já aponta para o `bin/` desta pasta).
3. Peça o deck. O agente lê o `SKILL.md`, escreve o `.tsx`, compila e corrige pelos diagnósticos.

No macOS, se o sistema bloquear o binário baixado: `xattr -d com.apple.quarantine bin/pptsx-rs`. O `init` grava os tipos do pptsx em `.pptsx/types/` e aponta o `tsconfig.json` para lá: o editor confere o deck sem `npm install` (num projeto que já existe, `bin/pptsx-rs types <pasta>`).
"#
    )
}

// ───────────── conteúdo que o DS traz ─────────────

/// O `SKILL.md` do DS: nome e descrição do frontmatter (quando há) e o corpo sem ele.
struct DsSkillMd {
    name: Option<String>,
    description: Option<String>,
    body: String,
}

fn unquote(value: &str) -> String {
    let v = value.trim();
    let quoted = v.len() >= 2 && ((v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')));
    if quoted { v[1..v.len() - 1].to_string() } else { v.to_string() }
}

/// Separa o frontmatter (`---` … `---`) do corpo; só `name` e `description` interessam.
fn parse_skill_md(text: &str) -> DsSkillMd {
    let whole = || DsSkillMd { name: None, description: None, body: text.to_string() };
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else { return whole() };
    let Some(end) = rest.find("\n---") else { return whole() };
    let field = |key: &str| rest[..end].lines().find_map(|l| l.strip_prefix(key).and_then(|r| r.strip_prefix(':'))).map(unquote).filter(|v| !v.is_empty());
    let body = rest[end + 4..].split_once('\n').map_or("", |(_, b)| b);
    DsSkillMd { name: field("name"), description: field("description"), body: body.trim_start_matches(['\n', '\r']).to_string() }
}

/// Troca `{{chave}}` pelo valor; chave desconhecida fica como está.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |text, (key, value)| text.replace(&format!("{{{{{key}}}}}"), value))
}

/// Agent Skills: letras minúsculas, dígitos e hífens.
fn check_name(name: &str) -> Result<(), String> {
    let valid = !name.is_empty() && name.len() <= MAX_NAME && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') && !name.starts_with('-') && !name.ends_with('-');
    if valid { Ok(()) } else { Err(format!("--name {name}: use letras minúsculas, dígitos e hífens (até {MAX_NAME} caracteres)")) }
}

/// A raiz do DS (o que é copiado para `ds/` e onde fica `skill/`): a pasta passada em `--ds`; para
/// um arquivo, a pasta dele ou a de cima quando ela tem `package.json` (`ds/src/index.ts` → `ds/`).
/// Não sobe mais que isso: o `package.json` seguinte costuma ser o do projeto inteiro.
fn ds_root(entry: &Path, spec_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = spec_dir {
        return dir.to_path_buf();
    }
    let own = entry.parent().unwrap_or(Path::new(".")).to_path_buf();
    let has_package = |dir: &Path| dir.join("package.json").is_file();
    if has_package(&own) {
        return own;
    }
    own.parent().filter(|up| has_package(up)).map_or(own.clone(), Path::to_path_buf)
}

fn relative_slashed(root: &Path, file: &Path) -> Option<String> {
    Some(file.strip_prefix(root).ok()?.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/"))
}

fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Copia a árvore do DS, sem pastas ocultas, `node_modules`, saídas de build, links simbólicos, a
/// pasta `skill/` da raiz (o conteúdo dela vira a própria skill) e `exclude` (a pasta de saída,
/// quando fica dentro do DS). Devolve quantos arquivos copiou.
fn copy_tree(from: &Path, to: &Path, exclude: &Path, top: bool) -> Result<usize, String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut copied = 0;
    let mut entries: Vec<_> = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?.flatten().collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let source = entry.path();
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_symlink() || name.starts_with('.') || BUNDLE_SKIP.contains(&name.as_str()) || (top && name == DS_SKILL_DIR) || canonical(&source) == exclude {
            continue;
        }
        if kind.is_dir() {
            copied += copy_tree(&source, &to.join(&name), exclude, false)?;
            continue;
        }
        std::fs::copy(&source, to.join(&name)).map_err(|e| format!("{}: {e}", source.display()))?;
        copied += 1;
    }
    Ok(copied)
}

/// Põe o DS em `<out>/ds/`. Quando ele já mora lá (a skill é montada em volta do DS), nada é copiado.
fn bundle_ds(root: &Path, out: &Path) -> Result<Option<usize>, String> {
    let target = out.join(DS_BUNDLE_DIR);
    let (root, target_real) = (canonical(root), canonical(&target));
    if target_real == root {
        return Ok(None);
    }
    // Apagar a cópia antiga apagaria o próprio DS.
    if root.starts_with(&target_real) {
        return Err(format!("o DS fica dentro de {}: passe essa pasta em --ds (ou dê um package.json a ela)", target.display()));
    }
    let _ = std::fs::remove_dir_all(&target);
    copy_tree(&root, &target, &canonical(out), true).map(Some)
}

/// Os `.md` de `skill/references/` do DS: (nome, conteúdo), em ordem.
fn ds_references(skill_dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(skill_dir.join("references")) else { return Vec::new() };
    let mut out: Vec<(String, String)> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| Some((e.file_name().to_string_lossy().into_owned(), std::fs::read_to_string(e.path()).ok()?)))
        .collect();
    out.sort();
    out
}

// ───────────── comando ─────────────

const USAGE: &str = "uso: pptsx-rs skill [--out <pasta>] [--name <nome>] [--description <texto>] [--ds <caminho|pacote>] [--bin <binário> | --no-bin]";

struct Args {
    out: Option<PathBuf>,
    bin: Option<PathBuf>,
    no_bin: bool,
    ds: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args { out: None, bin: None, no_bin: false, ds: None, name: None, description: None };
    let mut it = argv.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--out" => args.out = Some(PathBuf::from(it.next().ok_or("--out precisa de uma pasta")?)),
            "--bin" => args.bin = Some(PathBuf::from(it.next().ok_or("--bin precisa de um arquivo")?)),
            "--no-bin" => args.no_bin = true,
            "--ds" => args.ds = Some(it.next().ok_or("--ds precisa de um pacote ou caminho")?.to_string()),
            "--name" => args.name = Some(it.next().ok_or("--name precisa de um nome")?.to_string()),
            "--description" => args.description = Some(it.next().ok_or("--description precisa de um texto")?.to_string()),
            other => return Err(format!("opção desconhecida: {other}\n{USAGE}")),
        }
    }
    Ok(args)
}

fn write(root: &Path, rel: &str, content: &str) -> Result<(), String> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))
}

fn copy_bin(root: &Path, source: &Path) -> Result<String, String> {
    let name = if source.extension().is_some_and(|e| e == "exe") { "pptsx-rs.exe" } else { "pptsx-rs" };
    let target = root.join("bin").join(name);
    std::fs::create_dir_all(root.join("bin")).map_err(|e| e.to_string())?;
    if canonical(source) == canonical(&target) {
        return Ok(format!("bin/{name}"));
    }
    std::fs::copy(source, &target).map_err(|e| format!("{} → {}: {e}", source.display(), target.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755));
    }
    Ok(format!("bin/{name}"))
}

/// A tabela de props de um componente do DS no `ds.md` (ordem da declaração).
pub(crate) fn ds_props_table(schema: &Value) -> String {
    props_table(schema, false)
}

fn generate(args: &Args, cwd: &Path) -> Result<(PathBuf, String, Vec<String>), String> {
    let version = env!("CARGO_PKG_VERSION");
    let schema = crate::mcp::core_schema().ok_or("o schema embutido não pôde ser lido")?;
    let info = args.ds.as_deref().map(|spec| crate::dsinfo::load(spec, cwd, cwd)).transpose()?;
    // A raiz do DS local: o que é empacotado em ds/ e onde fica o conteúdo próprio da skill.
    let root = info.as_ref().zip(args.ds.as_deref()).filter(|(i, _)| i.by_path).map(|(i, spec)| {
        let target = if Path::new(spec).is_absolute() { PathBuf::from(spec) } else { cwd.join(spec) };
        canonical(&ds_root(&i.entry, target.is_dir().then_some(target.as_path())))
    });
    // Um pacote traz a pasta skill/ na raiz dele (o ancestral com package.json).
    let skill_dir = match (&root, &info) {
        (Some(root), _) => Some(root.join(DS_SKILL_DIR)),
        (None, Some(i)) => i.entry.ancestors().find(|a| a.join("package.json").is_file()).map(|a| a.join(DS_SKILL_DIR)),
        (None, None) => None,
    };
    let own_md = skill_dir.as_ref().and_then(|d| std::fs::read_to_string(d.join("SKILL.md")).ok()).map(|text| parse_skill_md(&text));
    let extra = skill_dir.as_deref().map(ds_references).unwrap_or_default();

    let name = args.name.clone().or_else(|| own_md.as_ref().and_then(|m| m.name.clone())).unwrap_or_else(|| NAME.to_string());
    check_name(&name)?;
    let description = args.description.clone().or_else(|| own_md.as_ref().and_then(|m| m.description.clone())).unwrap_or_else(|| DEFAULT_DESCRIPTION.to_string());
    let identity = Identity { name: name.clone(), description };
    let out = args.out.clone().unwrap_or_else(|| PathBuf::from(&name));
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;

    let mut written: Vec<String> = Vec::new();
    let bundled_entry = match (&root, &info) {
        (Some(root), Some(i)) => {
            let rel = relative_slashed(root, &i.entry).ok_or_else(|| format!("{} fica fora de {}", i.entry.display(), root.display()))?;
            match bundle_ds(root, &out)? {
                Some(n) => written.push(format!("{DS_BUNDLE_DIR}/ ({n} arquivos do {})", i.name)),
                None => written.push(format!("{DS_BUNDLE_DIR}/ (o {} já está aqui)", i.name)),
            }
            Some(format!("{DS_BUNDLE_DIR}/{rel}"))
        }
        _ => None,
    };
    let ds = info.as_ref().zip(args.ds.as_deref()).map(|(info, spec)| SkillDs { info, bundled_entry, spec });

    let extra_names: Vec<String> = extra.iter().map(|(n, _)| n.clone()).collect();
    let ds_location = ds.as_ref().map(SkillDs::location).unwrap_or_default();
    let skill_text = match &own_md {
        Some(md) => {
            let vars = [("bin", BIN), ("ds", ds_location.as_str()), ("name", name.as_str()), ("version", version), ("ds_name", info.as_ref().map_or("", |i| i.name.as_str())), ("ds_version", info.as_ref().map_or("", |i| i.version.as_str()))];
            format!("{}\n{}", frontmatter(&identity), fill(&md.body, &vars))
        }
        None => skill_md(version, &identity, ds.as_ref(), &extra_names),
    };
    // O que o servidor MCP recebe em --ds: o DS dentro da skill instalada, ou o pacote.
    let mcp_ds = |prefix: &str| ds.as_ref().map(|d| d.bundled_entry.as_ref().map_or(d.spec.to_string(), |entry| format!("{prefix}/{entry}")));
    let (vscode_dir, claude_dir) = (format!(".github/skills/{name}"), format!(".claude/skills/{name}"));
    let mut files = vec![
        ("SKILL.md".to_string(), skill_text),
        ("references/essentials.md".to_string(), essentials(crate::mcp::GUIDE)),
        ("references/guide.md".to_string(), guide_reference(crate::mcp::GUIDE)),
        ("references/props.md".to_string(), props_md(&schema)),
        ("mcp/vscode.mcp.json".to_string(), mcp_json(&name, &format!("{vscode_dir}/bin/pptsx-rs"), mcp_ds(&format!("${{workspaceFolder}}/{vscode_dir}")).as_deref(), true)),
        ("mcp/claude.mcp.json".to_string(), mcp_json(&name, &format!("{claude_dir}/bin/pptsx-rs"), mcp_ds(&claude_dir).as_deref(), false)),
        ("README.md".to_string(), readme(version, &name, ds.as_ref())),
    ];
    if let Some(ds) = &ds {
        let location = match &ds.bundled_entry {
            Some(entry) => crate::dsinfo::DsLocation { import_from: format!("<caminho relativo do deck até a skill>/{entry}"), config_hint: format!("`{BIN} init <pasta> --ds {SKILL_ROOT}/{entry}` cria o deck e esse arquivo com o caminho certo") },
            None => crate::dsinfo::DsLocation { import_from: ds.info.import_from.clone(), config_hint: "o arquivo pronto está em `references/pptsx.config.ts`".into() },
        };
        files.push(("references/ds.md".to_string(), crate::dsinfo::markdown(ds.info, &location, ds_props_table)));
        if ds.bundled_entry.is_none() {
            files.push(("references/pptsx.config.ts".to_string(), crate::dsinfo::config_content(ds.info)));
        }
    }
    // As referências do DS entram por último: um arquivo com o nome de um gerado o substitui.
    for (file, content) in extra {
        let rel = format!("references/{file}");
        files.retain(|(existing, _)| existing != &rel);
        files.push((rel, content));
    }
    for (rel, content) in &files {
        write(&out, rel, content)?;
        written.push(rel.clone());
    }
    if !args.no_bin {
        let source = match &args.bin {
            Some(b) => b.clone(),
            None => std::env::current_exe().map_err(|e| format!("não achei o próprio binário: {e} (use --bin ou --no-bin)"))?,
        };
        written.push(copy_bin(&out, &source)?);
    }
    Ok((out, name, written))
}

pub fn run(argv: &[String]) -> ExitCode {
    let args = match parse(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    let (out, name, written) = match generate(&args, &cwd) {
        Ok(done) => done,
        Err(e) => {
            eprintln!("\x1b[31merro\x1b[0m {e}");
            return ExitCode::from(1);
        }
    };
    println!("Skill {name} (pptsx-rs {}) em {}:", env!("CARGO_PKG_VERSION"), out.display());
    for w in written {
        println!("  {w}");
    }
    println!("\nMova a pasta para .github/skills/{name} (Copilot) ou .claude/skills/{name} (Claude Code); mcp/ tem a entrada do servidor MCP.");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sentence_ignores_dots_inside_code_quotes_and_parentheses() {
        assert_eq!(first_sentence("Variação (\"+14% vs. Q2\"). Cor pelo sentimento."), "Variação (\"+14% vs. Q2\").");
        assert_eq!(first_sentence("De 3 a 6 tópicos: `{ title, text? }`. Em ordem."), "De 3 a 6 tópicos: `{ title, text? }`.");
        assert_eq!(first_sentence("Custo em R$ 3.5 mi por cidade"), "Custo em R$ 3.5 mi por cidade");
        assert_eq!(first_sentence("Uma frase. Outra frase."), "Uma frase.");
        assert_eq!(first_sentence("Sem ponto final"), "Sem ponto final");
    }

    #[test]
    fn reads_name_and_description_from_the_ds_skill_frontmatter() {
        let md = parse_skill_md("---\nname: acme-deck\ndescription: \"Decks da Acme: capa e conteúdo.\"\n---\n\n# Título\n\ncorpo com {{bin}}\n");
        assert_eq!(md.name.as_deref(), Some("acme-deck"));
        assert_eq!(md.description.as_deref(), Some("Decks da Acme: capa e conteúdo."));
        assert_eq!(md.body, "# Título\n\ncorpo com {{bin}}\n");
        let plain = parse_skill_md("# Sem frontmatter\n");
        assert!(plain.name.is_none() && plain.body.starts_with("# Sem"));
    }

    #[test]
    fn fills_known_placeholders_only() {
        assert_eq!(fill("{{bin}} build; {{outra}}", &[("bin", "x/bin/pptsx-rs")]), "x/bin/pptsx-rs build; {{outra}}");
    }

    #[test]
    fn frontmatter_quotes_a_description_that_would_break_yaml() {
        let front = frontmatter(&Identity { name: "x".into(), description: "Decks: capa e \"fecho\"".into() });
        assert_eq!(front, "---\nname: x\ndescription: \"Decks: capa e \\\"fecho\\\"\"\n---\n");
        assert!(frontmatter(&Identity { name: "x".into(), description: "Cria decks.".into() }).contains("description: Cria decks.\n"));
    }

    #[test]
    fn skill_names_follow_the_agent_skills_rule() {
        assert!(check_name("acme-deck").is_ok());
        assert!(check_name("Acme Deck").is_err());
        assert!(check_name("-x").is_err());
    }

    #[test]
    fn mcp_entry_carries_the_name_and_the_ds() {
        let claude = mcp_json("acme-deck", ".claude/skills/acme-deck/bin/pptsx-rs", Some(".claude/skills/acme-deck/ds/src/index.ts"), false);
        assert!(claude.contains("\"acme-deck\": {") && claude.contains("\"args\": [\"mcp\", \"--ds\", \".claude/skills/acme-deck/ds/src/index.ts\"]"), "{claude}");
        let vscode = mcp_json("pptsx", ".github/skills/pptsx/bin/pptsx-rs", None, true);
        assert!(vscode.contains("\"command\": \"${workspaceFolder}/.github/skills/pptsx/bin/pptsx-rs\"") && vscode.contains("\"args\": [\"mcp\"]"), "{vscode}");
    }

    #[test]
    fn the_guide_ships_without_repository_residue() {
        let guide = crate::mcp::GUIDE;
        for residue in ["packages/", "npx ", "pptsx-mcp", "@pptsx/ds-base", "node packages"] {
            assert!(!guide.contains(residue), "o guia ainda fala de `{residue}`");
        }
        let essentials = essentials(guide);
        assert!(essentials.contains("### Container"), "o essencial precisa da seção do Container");
        assert!(guide_reference(guide).starts_with("**Índice**"));
    }

    #[test]
    fn ds_root_stops_at_the_package_next_to_the_entry() {
        let base = std::env::temp_dir().join(format!("pptsx-rs-skill-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("ds/src")).expect("pastas");
        std::fs::write(base.join("package.json"), "{}").expect("package do projeto");
        assert_eq!(ds_root(&base.join("ds/src/index.ts"), None), base.join("ds/src"), "sem package.json no DS, só a pasta da entrada");
        std::fs::write(base.join("ds/package.json"), "{}").expect("package do DS");
        assert_eq!(ds_root(&base.join("ds/src/index.ts"), None), base.join("ds"));
        assert_eq!(ds_root(&base.join("ds/src/index.ts"), Some(&base.join("ds"))), base.join("ds"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn bundling_skips_what_is_not_ds_code() {
        let base = std::env::temp_dir().join(format!("pptsx-rs-skill-bundle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        for dir in ["ds/src", "ds/node_modules/x", "ds/skill/references", "ds/.git", "ds/src/skill"] {
            std::fs::create_dir_all(base.join(dir)).expect("pastas");
        }
        for file in ["ds/package.json", "ds/src/index.ts", "ds/node_modules/x/index.js", "ds/skill/SKILL.md", "ds/.git/HEAD", "ds/src/skill/keep.ts"] {
            std::fs::write(base.join(file), "x").expect("arquivo");
        }
        let out = base.join("ds/out");
        std::fs::create_dir_all(&out).expect("saída dentro do DS");
        assert_eq!(bundle_ds(&canonical(&base.join("ds")), &out), Ok(Some(3)));
        assert!(out.join("ds/src/index.ts").is_file() && out.join("ds/src/skill/keep.ts").is_file());
        assert!(!out.join("ds/node_modules").exists() && !out.join("ds/skill").exists() && !out.join("ds/.git").exists() && !out.join("ds/out").exists());
        // Rodar de novo não copia a cópia para dentro dela.
        assert_eq!(bundle_ds(&canonical(&base.join("ds")), &out), Ok(Some(3)));
        let _ = std::fs::remove_dir_all(&base);
    }
}
