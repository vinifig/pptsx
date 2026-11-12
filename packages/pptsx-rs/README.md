<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="../pptsx/docs/readme/brand/logo-dark.svg">
    <img src="../pptsx/docs/readme/brand/logo-light.svg" height="64" alt="pptsx">
  </picture>
</p>

<h1 align="center">pptsx-rs</h1>

<p align="center">
  <b>O pptsx num binário só.</b> TSX entra, PPTX editável sai. Sem Node, sem <code>node_modules</code>, sem motor JS embutido.
</p>

<p align="center">
  <img alt="pptsx-rs 1.1.3" src="https://img.shields.io/badge/pptsx--rs-1.1.3-4F5BFF?style=flat-square&labelColor=0B1020">
  <img alt="sem Node nem npm" src="https://img.shields.io/badge/sem%20Node-sem%20npm-14B87F?style=flat-square&labelColor=0B1020">
  <img alt="binário de 6 MB" src="https://img.shields.io/badge/bin%C3%A1rio-6%20MB-C246D8?style=flat-square&labelColor=0B1020">
  <img alt="macOS, Linux e Windows" src="https://img.shields.io/badge/macOS%20%C2%B7%20Linux%20%C2%B7%20Windows-5%20alvos-FF6A3D?style=flat-square&labelColor=0B1020">
  <img alt="Rust 2024" src="https://img.shields.io/badge/Rust-2024-8A93B8?style=flat-square&labelColor=0B1020&logo=rust&logoColor=white">
  <img alt="Licença MIT" src="https://img.shields.io/badge/licen%C3%A7a-MIT-8A93B8?style=flat-square&labelColor=0B1020">
</p>

<p align="center">
  <a href="#instalar"><b>Instalar</b></a> ·
  <a href="#em-30-segundos"><b>Começar</b></a> ·
  <a href="#com-ia-no-projeto"><b>Com IA</b></a> ·
  <a href="#design-systems"><b>Design systems</b></a> ·
  <a href="#a-cli"><b>CLI</b></a> ·
  <a href="#regressão-contra-o-legado-em-ts"><b>Regressão</b></a> ·
  <a href="#como-funciona"><b>Por dentro</b></a> ·
  <a href="#o-que-falta"><b>O que falta</b></a>
</p>

<br>

**Este é o motor principal do pptsx.** Ele nasceu como port do [pacote em TypeScript](../pptsx/README.md), que hoje é legado: o que é novo entra aqui e não precisa ser portado para lá. O binário não depende de Node nem de `npm` para nada do fluxo de quem **usa** ou **cria** um design system: `pptsx` e `@pptsx/ds-kit` são módulos de dentro dele, os tipos do editor saem de `pptsx-rs init`/`types` e a skill de um DS sai de `pptsx-rs skill`. As fixtures de `fixtures/` (a saída do TS em sete decks) continuam em `cargo test` como regressão estática.

<table>
<tr>
<td width="33%" valign="top">
<img src="../pptsx/docs/readme/brand/icon-native.svg" width="44" alt=""><br>
<b>Um arquivo, zero runtime</b><br>
<sub>Baixe o binário do seu sistema e rode. O deck <code>.tsx</code> e o design system são interpretados por um avaliador próprio; os tipos para o editor e o <code>@pptsx/ds-kit</code> vêm de dentro do binário.</sub>
</td>
<td width="33%" valign="top">
<img src="../pptsx/docs/readme/brand/icon-diagnostics.svg" width="44" alt=""><br>
<b>Layout de verdade</b><br>
<sub>Port do Yoga 3.2.1, motor de grid próprio, emissor OOXML direto e diagnósticos com <code>path</code> e causa. Sete decks dourados em <code>cargo test</code> seguram a saída contra regressão.</sub>
</td>
<td width="33%" valign="top">
<img src="../pptsx/docs/readme/brand/icon-agents.svg" width="44" alt=""><br>
<b>Feito para agentes</b><br>
<sub>Servidor MCP com <code>build</code>, <code>diagnose</code>, <code>preview</code>, <code>lint</code>, <code>guide</code> e <code>schema</code>. O <code>mcp.json</code> do Copilot ou do Claude Code aponta direto para o executável.</sub>
</td>
</tr>
</table>

## Instalar

Cada [release](https://github.com/JonathanSantos/pptsx/releases/latest) traz um pacote por plataforma e o `SHA256SUMS`. O pacote **é a skill**: uma pasta `pptsx/` com o `SKILL.md`, as referências para o agente, os `mcp.json` prontos e o binário em `bin/`. Descompactou, moveu para a pasta de skills, acabou.

| Sistema | Arquivo |
|---|---|
| macOS Apple Silicon | `pptsx-rs-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `pptsx-rs-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `pptsx-rs-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `pptsx-rs-aarch64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `pptsx-rs-x86_64-pc-windows-msvc.zip` |

macOS e Linux, na raiz do repositório em que o agente vai trabalhar:

```bash
alvo=aarch64-apple-darwin   # ou x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
base=https://github.com/JonathanSantos/pptsx/releases/latest/download
curl -LO "$base/pptsx-rs-$alvo.tar.gz" && curl -LO "$base/SHA256SUMS"
shasum -a 256 -c SHA256SUMS --ignore-missing          # confere o pacote baixado
mkdir -p .github/skills && tar -xzf "pptsx-rs-$alvo.tar.gz" -C .github/skills    # Copilot; Claude Code: .claude/skills
cp .github/skills/pptsx/mcp/vscode.mcp.json .vscode/mcp.json                     # Claude Code: mcp/claude.mcp.json → .mcp.json
.github/skills/pptsx/bin/pptsx-rs --version
```

Pronto: o agente lê `.github/skills/pptsx/SKILL.md`, sabe onde está o binário e como iniciar, construir e corrigir um deck. No Windows, extraia o `.zip` em `.github\skills\` e o binário é `bin\pptsx-rs.exe`. No macOS, um pacote baixado pelo navegador pode chegar com a quarentena do Gatekeeper; `xattr -d com.apple.quarantine .github/skills/pptsx/bin/pptsx-rs` libera.

A pasta da skill também sai do próprio binário, para quem compilou do código: `pptsx-rs skill --out pptsx` (gera `SKILL.md`, `references/`, `mcp/` e copia o binário para `bin/`). Com um design system, `pptsx-rs skill --ds <pasta do DS> --name <nome>` gera a skill **da marca**, com o DS dentro ([Design systems](#design-systems)).

Para compilar do código (Rust estável, 1.85+):

```bash
cargo build --release          # na raiz do monorepo (workspace) ou em packages/pptsx-rs
./target/release/pptsx-rs --version
```

## Em 30 segundos

```bash
bin=.github/skills/pptsx/bin/pptsx-rs
$bin init meu-deck                                 # deck.tsx, tsconfig.json, pptsx.config.ts, assets/ e os tipos do pptsx
$bin init meu-deck --ds ../ds/src/index.ts         # o mesmo, com o deck já usando um design system
$bin build meu-deck/deck.tsx --size 16x9,4x3       # .pptx + PNG por slide + overview + diagnósticos
```

```text
dist/
├── deck.16x9.pptx              editável no PowerPoint e no Keynote
├── deck.16x9.overview.png      todos os slides numa grade, com selo vermelho onde há error
├── deck.16x9/slide-01.png      um PNG por slide, desenhado com o mesmo layout do arquivo
├── deck.4x3.pptx …             o mesmo deck no outro formato, sem mudar código
└── deck.diagnostics.json       código, severidade, slide, path do nó, mensagem e dados
```

O `init` grava os `.d.ts` do pptsx em `.pptsx/types/pptsx/` e os do kit em `.pptsx/types/@pptsx/ds-kit/`, como pacotes de tipos, e põe `typeRoots` no `tsconfig.json`: o VS Code confere tipos e autocompleta as props do deck **e de um DS** sem `npm install`. Para o editor, um pacote instalado em `node_modules` vence a cópia; para o build, não (veja [Design systems](#design-systems)). Num projeto que já existe, `pptsx-rs types <pasta>` faz só isso. O build sai com código 1 quando há diagnóstico `error`.

Quando um nó estoura dentro de um componente, o diagnóstico diz **qual componente do autor e com que props** (`no componente <Risks items=[4 itens] status="warn" />`, e `data.component` no JSON): é ali que se mexe, não no path interno do DS. Um `console.warn` do deck ou do DS (um ícone com nome desconhecido, por exemplo) vira o diagnóstico `module-warning`, em vez de sumir.

## Com IA no projeto

O jeito principal de um agente usar o pptsx-rs é pelo **servidor MCP**: o agente lê o guia, escreve o deck, constrói, lê os diagnósticos e vê o PNG, tudo por ferramentas, sem shell. O binário é o servidor: nada de `npx`, nada de runtime.

### Ferramentas

| ferramenta | o que devolve | quando o agente usa |
|---|---|---|
| `guide` | o índice do guia de primitivas e receitas, ou uma seção (`"§6"`, `"grid"`); com um DS registrado, o guia do DS | antes de escrever o primeiro slide |
| `schema` | as props de cada primitiva (ou componente do DS) em JSON Schema, com o tipo TS; num DS, lidas das interfaces de props (tipo, obrigatória, descrição, padrão) | quando não sabe o nome ou o tipo de uma prop |
| `diagnose` | só os diagnósticos, sem gravar arquivo: o caminho rápido para iterar | a cada mudança no deck |
| `build` | grava `.pptx` e PNGs e devolve os diagnósticos e o **overview em PNG** de cada tamanho | quando o deck está sem `error` |
| `preview` | o PNG de um slide num tamanho | para conferir um slide que o diagnóstico apontou |
| `lint` | regras da engine e as do design system da config | antes de entregar |
| `catalog` | o índice dos componentes dos DS registrados, por camada (template, structure, chart, component, widget), com público, quando usar, o que evitar e o JSX de cada caso; filtra por `audience` e `kind` | antes de escrever um deck com um DS |

`build`, `diagnose` e `lint` marcam `isError` quando o deck tem `error` (ou `warning`, no lint): a ferramenta funcionou, o deck é que precisa de correção. Cada diagnóstico diz o slide, o caminho do nó (`Slide > Box[1] > Text` é o `Text` do segundo `Box` do slide), o que aconteceu e o que fazer, e, num estouro, qual nó causou (`causedBy`).

### GitHub Copilot no VS Code

A skill em `.github/skills/pptsx/` já ensina o fluxo e as armadilhas ao agent mode. Para ele usar as ferramentas em vez do shell, `.vscode/mcp.json` na raiz do repositório (é o arquivo `mcp/vscode.mcp.json` da skill):

```json
{
  "servers": {
    "pptsx": {
      "type": "stdio",
      "command": "${workspaceFolder}/.github/skills/pptsx/bin/pptsx-rs",
      "args": ["mcp"]
    }
  }
}
```

Depois, no chat em agent mode: *"Crie um deck de 5 slides sobre o resultado do trimestre em `decks/q3.tsx` e gere o PPTX em 16:9 e 4:3"*. O agente lê o `SKILL.md`, chama `guide` se precisar, escreve o arquivo, chama `build`, lê os diagnósticos e corrige até sair limpo.

### Claude Code

A mesma pasta vai em `.claude/skills/pptsx/`; `.mcp.json` na raiz do repositório é o `mcp/claude.mcp.json` da skill:

```json
{
  "mcpServers": {
    "pptsx": {
      "command": ".claude/skills/pptsx/bin/pptsx-rs",
      "args": ["mcp"]
    }
  }
}
```

No Claude Desktop, a entrada é a mesma em `claude_desktop_config.json`, com o caminho absoluto do binário.

### Sem MCP: o agente na linha de comando

Um agente que só tem shell usa os mesmos comandos de uma pessoa, com `--json` para ler a saída por máquina:

```bash
$bin check deck.tsx --json          # diagnósticos em JSON, sem gravar nada
$bin build deck.tsx --size 16x9     # .pptx, PNGs e dist/deck.diagnostics.json
$bin lint deck.tsx --json           # regras da engine e do DS
```

O ciclo que funciona: `check --json` até zerar os `error`, `build`, abrir o `overview.png` e conferir. Em CI, `build` com código de saída 1 em `error` basta como gate.

### O que o agente recebe de volta

```json
{
  "size": "16x9",
  "slide": 2,
  "severity": "error",
  "code": "text-overflow",
  "path": "Slide > Box > Text",
  "message": "A área disponível para o texto é 0 pt: o contêiner não deu altura a ele (flexGrow={1} em coluna com altura pelo conteúdo…). Remova flexGrow do Text ou dê altura ao contêiner.",
  "data": { "overflowPt": 18.4, "causedBy": "Slide > Box" }
}
```

Mensagens em português, escritas para serem lidas por um modelo: dizem o que mudar. Irmãos com o mesmo problema viram um só item (`Box[0..3]`), com a lista em `data.paths`.

## Design systems

Um design system (DS) é um módulo TS que exporta os componentes e um manifesto `ds`. O deck o importa pelo caminho (`import { Content, Stat } from '../ds/src/index.ts'`) ou pelo nome de um pacote em `node_modules`, e o código do DS roda no mesmo avaliador do deck. O `pptsx.config.ts` registra o DS:

```ts
import { defineConfig } from 'pptsx';
import { ds } from '../ds/src/index.ts';

export default defineConfig({ ds: [ds], theme: ds.theme });   // tema no arquivo, regras no lint, guia e schema no MCP
```

Com isso o `lint` aplica as regras de marca do DS, o `guide` do MCP ganha a fonte do DS e o `schema` responde as props dos componentes dele. O servidor também aceita `pptsx-rs mcp --ds <caminho|pasta|pacote>` para um DS fora da config. Achados `off-theme-color` de nível info dentro de componentes do DS são do DS, não do deck: o `lint` os resume numa linha (o `--json` lista todos).

### Catálogo visual e registro

Cada componente declara no `defineComponent` a **camada** (`kind`: `template`, `structure`, `chart`, `component` ou `widget`), o **público** (`audience`: `executive`, `strategic`, `tactical`, `operational`, também por caso do catálogo), `description`, `whenToUse` e `avoid`. O `pptsx.registry.json` lista os DS do projeto, e `pptsx-rs catalog` renderiza cada caso de catálogo de cada DS com o motor de verdade e gera um visualizador num HTML só, o `catalog.json` (o contrato, `pptsx.catalog/1`), o `prompt.md` de cada DS e um `.pptx` por tamanho. O contrato completo está em [`docs/ds-catalog.md`](../../docs/ds-catalog.md).

```bash
pptsx-rs registry add packages/ds-base --id ds-base   # grava pptsx.registry.json (carrega o DS antes)
pptsx-rs catalog                                      # dist/catalog/index.html, catalog.json, <id>/prompt.md e .pptx
pptsx-rs catalog --json | --prompt                    # só o contrato ou só o prompt, sem construir nada
```

O servidor MCP também lê o registro: os DS dele viram fontes de `guide`, `schema` e `catalog` sem `--ds`.

### `@pptsx/ds-kit` vem no binário
`pptsx-rs init <pasta> --ds @pptsx/ds-base` (ou um caminho, `--ds ../meu-ds/src/index.ts`) cria o deck com um slide por componente do DS, a config que o registra e o tsconfig; `pptsx-rs skill --ds …` gera a skill com `references/ds.md` (componentes, exemplos, props, regras e o guia do DS) e `references/pptsx.config.ts`, para o agente usar a marca sem procurar.

### Regras de lint de um DS

Uma regra é um objeto `{ id, severity, check?, checkDeck? }` (tipos em `lint/rule.d.ts`), avaliado pelo interpretador do binário em cada tamanho.

`check(node, ctx)` roda para cada slide e cada nó. `node.props` traz cada valor com o nome que o autor escreve no TSX e com o alias da 0.x, sempre iguais: `fontSize`/`size`, `fontFamily`/`font`, `fontWeight`/`weight`, `fontStyle` (`'italic'`/`'normal'`)/`italic` (boolean), `textAlign`/`align` em Text e SlideNumber; `fontSize` e `fontFamily` também em Table e Chart; `backgroundColor`/`fill` em Shape e Path; `objectFit`/`fit` em Image. Text, SlideNumber e Table trazem ainda `fittedFontSize`, o corpo depois do fit (menor que `fontSize` quando o `fit="shrink"` atuou).

`checkDeck(deck, ctx)` roda uma vez por tamanho com `deck.slides`, todos os slides resolvidos (`component`, `path`, `children`, `css`, `props` e `sourceSlide`, o número no `<Deck>` do autor). É onde se cobra ritmo, proporção e o que falta no deck. O achado é `{ message, slide?, path?, severity? }`: com `slide`, aponta aquele slide; sem, sai como `[deck] … em Deck` (`"slide": 0` no JSON).

```ts
const rhythm = {
  id: 'meu-ds/rhythm',
  severity: 'warning',
  checkDeck: (deck) => {
    const name = (slide) => slide.component?.name;
    const tripled = deck.slides.filter((slide, i) => i >= 2 && name(slide) !== undefined && name(slide) === name(deck.slides[i - 1]) && name(slide) === name(deck.slides[i - 2]));
    const closing = name(deck.slides.at(-1)) === 'Closing' ? [] : [{ message: 'O deck não termina com um slide de encerramento (Closing).' }];
    return [...tripled.map((slide) => ({ slide: slide.sourceSlide, message: `Terceiro slide seguido com o layout ${name(slide)}; alterne.` })), ...closing];
  },
};
```

O `off-theme-color` separa o que é do deck do que é do DS. Um hex solto que o autor escreveu aparece como sempre, inclusive num filho que ele entregou a um componente do DS. A cor que o DS declara (`colors` ou `palette` no manifesto, `tokens.color`, `theme.colors`) e tudo o que um componente do DS pinta por dentro levam `"origin": "ds"`: ficam fora do resumo em texto e da contagem do cabeçalho, continuam no `--json` com o marcador e aparecem no texto com `--verbose`. O tema do arquivo só tem 10 slots de cor, então um DS com mais tokens sempre teria sobra; essa sobra não é assunto de quem escreve o deck.

Quem **cria** um DS importa as ferramentas de `@pptsx/ds-kit`, e esse módulo é servido de dentro do binário: `defineTokens`, `colorsOf`, `themeOf`, `checkTheme`, `contrastOf`, `AA_TEXT`, `AA_LARGE`, `createVariants`, `slot`, `filledSlots`, `defineRule`, `colorsFromTokens`, `fontsFromTheme`, `maxChildren`, `onlyOnSlide`, `paintedColorsOf`, `createDs`, `componentGuides`, `guideOf`, `metaOfComponent`, `schemaFromMeta`, mais `defineComponent`, `defineDs` e `defineLayout` do núcleo. Um DS que importa de `'@pptsx/ds-kit'` compila numa pasta sem `node_modules`.

- **As fontes ficam em [`assets/ds-kit/`](assets/ds-kit)** (`.js` que o avaliador executa e `.d.ts` que o `init`/`types` gravam para o editor) e são **os originais do kit**: mudou a API, mude ali.
- **A cópia embutida sempre vence** um `node_modules/@pptsx/ds-kit`. O pacote do npm reexporta catálogo, scaffold e skill, que precisam de Node; e um DS tem de compilar igual com ou sem `npm install`. (Subcaminhos como `@pptsx/ds-kit/tsconfig.base.json` continuam indo ao `node_modules`.)
- **Fora do kit embutido:** `catalog`/`compareFrames` (o catálogo visual com fixtures), `schemaOf` (o binário lê os tipos sozinho, abaixo), `initDs` e `writeDsSkill` (`pptsx-rs init --ds` e `pptsx-rs skill --ds` fazem esse papel).

### Schema das props, lido dos tipos

Para cada `defineComponent<XProps>(meta, fn)` dos módulos do DS, o binário acha a interface (ou o `type`) `XProps` e tira de cada prop o nome, o tipo (o texto do TS), se é opcional (`?`), a descrição (o JSDoc `/** … */` acima dela) e o padrão da desestruturação da função (`{ tone = 'default' }`). É o que o `schema` do MCP e o `references/ds.md` da skill mostram, **mesmo para componentes sem caso de catálogo**; os exemplos do catálogo entram como exemplos de cada prop.

- Segue `extends` e interseções (`A & { … }`) entre tipos do DS, inclusive de outro arquivo. O que só o compilador abriria (`Omit<BoxProps, 'x'>`, um tipo de outro pacote) aparece como "aceita também as props de …" (`x-extends` no schema).
- Acha o `defineComponent` numa `const`, no valor de uma chave de objeto e no retorno de uma fábrica; a função pode ser inline, declarada, ou devolvida por uma fábrica (`view(theme)`).
- Sem argumento de tipo, vale a anotação do 1º parâmetro da função. Um `schema` passado ao `createDs` pelo autor do DS vence o lido dos tipos.

### `init --ds` e `skill --ds`

`pptsx-rs init <pasta> --ds <caminho|pasta|pacote>` cria o deck com um slide por componente do DS que tem caso de catálogo, a config que o registra e o tsconfig.

Um DS pode trazer o próprio deck inicial: um `starter.tsx` na raiz dele (ao lado do `package.json`, ou na pasta da entrada) vira o `deck.tsx` do projeto, com `{{ds}}` trocado pelo caminho de import. Sem ele, o `init` gera o catálogo.

`pptsx-rs skill --ds <caminho> --name <nome> [--description <texto>] --out <pasta>` gera a skill **autocontida** de um DS, sem Node:

```text
<pasta>/
├── SKILL.md                 o genérico, ou o do DS (abaixo), com name/description no frontmatter
├── bin/pptsx-rs             este binário
├── ds/…                     o código do DS (sem node_modules, dist, pastas ocultas e a pasta skill/)
├── references/
│   ├── essentials.md, guide.md, props.md      o guia do binário e as props das primitivas
│   ├── ds.md                                   cada componente: assinatura, exemplo, tabela de props, regras e o guia do DS
│   └── *.md                                    as referências próprias do DS
└── mcp/{vscode,claude}.mcp.json                servidor "<nome>", com --ds apontando para ds/ dentro da skill
```

- **O que é empacotado em `ds/`:** a pasta passada em `--ds`; para um arquivo, a pasta dele, ou a de cima quando ela tem `package.json` (`ds/src/index.ts` → `ds/`). Os textos gerados apontam para `<pasta-desta-skill>/ds/…`. Se o DS já mora em `<out>/ds`, nada é copiado. Um DS que é pacote (`--ds @marca/ds`) não é copiado: a skill ganha `references/pptsx.config.ts` e o MCP recebe `--ds @marca/ds`.
- **Conteúdo próprio do DS:** uma pasta `skill/` na raiz do DS. `skill/SKILL.md` substitui o `SKILL.md` genérico: o frontmatter dele dá o nome e a descrição padrão (as flags vencem) e o corpo aceita `{{bin}}` (`<pasta-desta-skill>/bin/pptsx-rs`), `{{ds}}` (a entrada do DS na skill), `{{name}}`, `{{version}}`, `{{ds_name}}` e `{{ds_version}}`. `skill/references/*.md` são copiados para `references/` (um arquivo com o nome de um gerado o substitui).
- `--name` é o nome da pasta, da skill e do servidor nos `mcp/*.json` (letras minúsculas, dígitos e hífens); sem ele, o do frontmatter do DS, senão `pptsx`.

## A CLI

```text
pptsx-rs build <deck.tsx> [--size 16x9,4x3] [--out dist/] [--lang pt-BR] [--line-breaks forced|auto]
               [--preview native|none] [--preview-width 1280] [--embed-fonts] [--watch] [--serve [--port 4173]]
pptsx-rs check <deck.tsx> [--size 16x9] [--debug] [--json]      diagnósticos (e frames em pt, com --debug)
pptsx-rs lint  <deck.tsx> [--size 16x9,4x3] [--json]            regras da engine e dos DS da config
pptsx-rs init  [pasta] [--ds <caminho|pacote>] [--force] [--json]
pptsx-rs lint  <deck.tsx> [--size 16x9,4x3] [--json] [--verbose] regras da engine e dos DS da config
pptsx-rs init  [pasta] [--ds <pacote|caminho>] [--force] [--json]
                                                                deck.tsx, tsconfig.json (+ typeRoots dos tipos), pptsx.config.ts,
                                                                assets/ e .pptsx/types; com --ds, o deck já usa o DS
pptsx-rs types [pasta]                                          (re)grava os tipos do pptsx e do @pptsx/ds-kit e o typeRoots
pptsx-rs skill [--out <pasta>] [--name <nome>] [--description <texto>] [--ds <caminho|pacote>]
               [--bin <binário> | --no-bin]                     a pasta da skill (SKILL.md, references/, mcp/, bin/);
                                                                com --ds <caminho>, o DS vai junto em ds/
pptsx-rs catalog [--ds <caminho|pacote>]... [--registry <arquivo>] [--out dist/catalog] [--size 16x9,4x3] [--json | --prompt]
                                                                o catálogo visual dos DS registrados (index.html, catalog.json,
                                                                prompt.md e .pptx por DS)
pptsx-rs registry list | add <ds> [--id <id>] | remove <id>     o pptsx.registry.json: os DS que catalog e mcp enxergam
pptsx-rs tree  <deck.tsx> [--size 16x9]                         a árvore avaliada
pptsx-rs mcp   [--ds <caminho|pacote>]...                       servidor MCP por stdio
```

| flag | o que faz |
|---|---|
| `--size` | `16x9`, `16x10`, `4x3`, `wide` ou `<L>x<A>in`; sem ela vale o `sizes` da config, senão `16x9` |
| `--line-breaks auto` | grava os parágrafos do autor e deixa o PowerPoint quebrar; o padrão `forced` grava uma quebra por linha calculada |
| `--preview none` | não rasteriza PNGs; `--preview-width` muda a largura (padrão 1280 px) |
| `--embed-fonts` | embute no arquivo as fontes do autor que o deck usa (EOT, como o PowerPoint exige); também `embedFonts: true` na config |
| `--watch` | refaz o build quando o deck, a config ou um módulo importado muda; cada build num processo filho, memória de volta a zero |
| `--serve` | o preview ao vivo do pptsx no navegador (slides em SVG, diagnósticos, clique destaca o nó, atualização por SSE); implica `--watch` |
| `--json` | em `check` e `lint`, só o JSON no stdout |
| `--verbose` | em `lint`, mostra também os achados do design system (`"origin": "ds"`), que o resumo em texto omite |

Os `.pptx` abrem e exportam no Keynote (`scripts/keynote-export.sh` do repositório): tabelas com mesclagem e ícones, gráficos de todos os tipos, `Path`, ícones, sombra, rotação, paginação, imagem de fundo, layouts com placeholders e conteúdo fixo. A validação no PowerPoint fica com o dono do projeto.

## Regressão contra o legado em TS

O motor nasceu reproduzindo a saída do pptsx em TypeScript. Na última comparação completa, os **80 decks de exemplo do repositório** gerados nas duas engines e comparados caixa a caixa (texto com as quebras de linha e frame com tolerância de 1 pt) deram:

| métrica | resultado |
|---|---|
| decks que geram nas duas engines | 80/80, com o mesmo número de slides em todos |
| caixas de texto idênticas | **8617/8617 (100%)** |
| decks 100% idênticos | **80/80** |
| geometrias livres (`custGeom`: Path, ícones, conectores) idênticas | **3496/3496** |
| diagnósticos do TS que o Rust dá no mesmo nó (código, slide, path) | **1704/1704**, sem nenhum a mais |
| achados do lint idênticos | **2074/2074** |
| fontes embutidas (`.fntdata` em EOT) | byte a byte iguais |

Daqui em diante o Rust é a fonte: o TS não acompanha o que é novo, e essa tabela é um retrato, não um contrato. O que continua valendo é a **regressão**: os sete decks de `fixtures/` em `cargo test`.

**Além do TS.** `fitGroup` (grupo de fit) só existe aqui: os `Text` do mesmo slide com o mesmo `fitGroup="nome"` (ou dentro de um `Box`/`Container` com `fitGroup`) encolhem pela mesma escala, a do membro que menos coube, seja por altura, por `nowrap` ou por palavra partida. Uma fileira de indicadores com números `nowrap` de comprimentos diferentes sai toda no mesmo corpo, sem medir nada com `<Container>`. Sem a prop, a saída é a mesma do TS. Testes em `tests/fit_group.rs`.

<details>
<summary><b>Desempenho</b> (Mac arm64; Node 24 rodando o pptsx do repositório; Rust em release; processo inteiro)</summary>
<br>

| deck | Node | Rust | ganho | memória Node | memória Rust |
|---|---:|---:|---:|---:|---:|
| `hello` | 367 ms | 6,4 ms | 58× | 167 MB | 6,7 MB |
| `layout-test` | 381 ms | 8,9 ms | 43× | 169 MB | 7,2 MB |
| `calibration` | 448 ms | 19,4 ms | 23× | 175 MB | 8,7 MB |
| `ds-base/demo` | 681 ms | 14 ms | 48× | 229 MB | 10,7 MB |
| `ds-base/showcase` | 667 ms | 32,4 ms | 21× | 228 MB | 9,6 MB |

O Node aqui transpila o próprio código-fonte do pptsx a cada execução (`tsx`); a versão publicada (`dist/`) começa mais rápido, então o ganho real fica um pouco abaixo. Nos decks com `Container` o ganho é menor: o DS é grande (o avaliador leva ~40 ms só para carregá-lo) e cada `Container` roda a função dele depois da medida (uma vez por medida; o cache evita rodar os de fora de novo quando os de dentro são medidos). Com o preview PNG ligado, o `ds-base/demo` (6 slides) sai em ~80 ms.

Binário: **6,1 MB**: 1,6 MB são as quatro Liberation Sans embutidas, e o resto inclui o `regex` com tabelas Unicode, o `resvg` (ícones complexos viram PNG, como no TS) e o guia e o schema do pptsx (para o MCP).
</details>

<details>
<summary><b>Como a regressão é testada</b> (fixtures douradas)</summary>
<br>

`fixtures/` guarda a saída de referência de sete decks de exemplo (`hello`, `layout-test`, `overflow`, `demo`, `charts`, `showcase`, `long-list`, em 16x9 e 4x3): o `diagnostics.json` e os `slideN.xml`/`chartN.xml` crus de cada `.pptx`, gravados uma vez pelo pptsx em TS. `cargo test` roda o binário nesses decks e compara, em `tests/golden.rs`, forma a forma: frames, caminhos e `srcRect` com tolerância de 2 EMU, texto com as quebras, corpo, peso e cor de cada trecho, preenchimento e contorno, células de tabela com bordas, e os gráficos (valores, título, estilo de texto, linhas dos eixos). Os diagnósticos comparam código, severidade, tamanho, slide, path e mensagem. As fixtures são **estáticas**: o CI não as regenera. Se um teste dourado quebrar, é regressão até prova em contrário; uma mudança de saída intencional regrava a fixture no mesmo commit, com o porquê (`scripts/golden.sh` ainda as gera com o TS, enquanto ele existir no repositório). Os decks `demo` e `showcase` importam `@pptsx/ds-base` pelo `node_modules` do monorepo: é a única coisa do `cargo test` que pede `npm ci` + `npm run build`.
</details>

## Como funciona

```text
deck.tsx ─oxc_parser─► AST ─avaliador do subconjunto─► árvore de elementos
         ─validação (props, CSS sem de/para)─► resolve (CSS, herança, responsivo)
         ─Container: layout mede o espaço, a função roda, resolve de novo (M19)
         ─paginate ─► layout (port do Yoga; grid próprio) + fit (shrink por escopo)─► frames
         ─diagnósticos (estouro, causa, agrupamento)
         ─emissor OOXML─► slides, tabelas, gráficos, notas, fontes ─zip─► .pptx
```

<details>
<summary><b>Os módulos</b> (e o arquivo do legado em TS de que cada um nasceu)</summary>
<br>

| etapa | arquivo | o quê |
|---|---|---|
| frontend | `src/js/` | `oxc` faz só o parse; um avaliador de AST executa o subconjunto (abaixo). `pptsx` é módulo nativo e `@pptsx/ds-kit` é servido das fontes embutidas (`assets/ds-kit/`, `module.rs`) |
| config | `src/config.rs` | `pptsx.config.ts` (fontes, sizes, lang, theme, document, embedFonts), avaliado pelo mesmo avaliador |
| validação | `src/validate.rs` | as tabelas de handlers do `resolve/` do pptsx: prop da 0.x e valor inválido derrubam o build com a mesma mensagem; o resto vira `css-not-emitted` |
| resolve | `src/model/` (`mod`, `style`, `shape`, `resolve`, `data`, `connector`), `src/css.rs` | props → estilo + pintura; responsivo; herança de texto; paths dos nós como o pptsx (`Slide > Box[1] > Text`); footer do Deck; ids de conector com escopo de componente |
| Container | `src/containers.rs` | `containers.ts`: passadas de resolve + layout sem fit até nada pendente (no máximo 3 níveis); resultado de cada função em cache por (slide, path, medida) |
| texto | `src/text.rs` | porta de `src/text/*`: quebra gulosa, `textSafety` 0,97, hhea, shaping com kerning (rustybuzz), pesos numéricos, corpo por trecho |
| layout | `src/yoga.rs`, `src/yoga_layout.rs`, `src/grid.rs`, `src/layout.rs` | port do Yoga 3.2.1 com a configuração do pptsx; árvore montada como `layout/tree.ts`; grid: porta do `grid.ts` (placement §8.5 e track sizing §11 do CSS Grid, com as contribuições min/max-content do `intrinsic.ts`) e cada item numa árvore Yoga do tamanho da área, como `grid-layout.ts`; fit de `fit/search.ts`; paginate; conectores; ordem de pintura CSS |
| diagnósticos | `src/report.rs`, `src/diag.rs` | `fit/detect.ts`, `diagnostics.ts`, `policies.ts`, `table.ts`, `cause.ts` e `group.ts`: mesmos códigos, paths, mensagens e dados |
| vetor | `src/vector.rs` | `d` do SVG → M/L/C/Q/Z (arcos viram cúbicas); ícones SVG → custGeom, ou PNG pelo resvg |
| objetos | `src/objects.rs` | tabela (merges, rich text, ícones, colunas pelo conteúdo) e gráficos (planilha embutida; rótulos com posição e cor por contraste com a série) |
| gradiente | `src/gradient.rs` | `linear-gradient(…)` e `radial-gradient(circle, …)` do CSS → `<a:gradFill>` no arquivo e `<linearGradient>`/`<radialGradient>` no preview (vai além do pptsx em TS) |
| formato numérico | `src/numfmt.rs` | códigos de formato do Excel mais comuns (`0.0`, `0%`, `#,##0`, literais entre aspas) com os separadores do `lang`, para o preview mostrar rótulos e eixos como o arquivo |
| emissão | `src/emit.rs`, `src/package.rs` | XML direto, sem pptxgenjs; tema, seções, notas, hyperlinks, layouts |
| embed | `src/embed.rs` | `fonts/eot.ts` e `emit/embed.ts`: faces do autor usadas no deck viram `ppt/fonts/fontN.fntdata` em EOT, `<p:embeddedFontLst>` e `embedTrueTypeFonts`; `font-not-embeddable` para .otf/.ttc e fsType restrito |
| preview | `src/preview.rs` | SVG por slide (texto em contorno, gráficos, imagens) → PNG pelo `resvg` |
| preview ao vivo | `src/serve.rs` | `--serve`: HTTP + SSE em `std::net`, a página do TS em `assets/preview.html`, estado gravado pelo build filho |
| init | `src/init.rs`, `src/types.rs` | arquivos do projeto novo, mesclas de tsconfig/package/.gitignore; os `.d.ts` do pptsx e do kit em `.pptsx/types/` |
| DS | `src/dsinfo.rs`, `src/dsprops.rs` | o manifesto de um DS lido pelo avaliador (`init --ds`, `skill --ds`, `mcp --ds`, `catalog`), com as camadas (`kind`) e o público (`audience`) dos componentes e o schema das props lido das interfaces TS com o `oxc` |
| catálogo | `src/catalog.rs`, `src/registry.rs`, `assets/catalog.html` | `pptsx-rs catalog`: o deck gerado com um slide por caso de catálogo, construído num processo filho, o `catalog.json` (pptsx.catalog/1), o `prompt.md` e o visualizador; o `pptsx.registry.json` e o comando `registry` |
| skill | `src/skill.rs` | a pasta da skill: `SKILL.md`, referências geradas do guia e do schema, `mcp/*.json`, o binário e, com `--ds`, o DS empacotado e o conteúdo próprio dele |
| lint | `src/lint.rs` | regras da engine e as dos DS (funções JS da config, chamadas pelo avaliador) |
| MCP | `src/mcp.rs` | JSON-RPC por stdio; guia e schema embutidos no binário |

O que o binário embute fica em `assets/`, e **esses arquivos são os originais**: `agent-guide.md` (o guia do `guide` do MCP e das referências da skill), `schema.generated.ts` (as props das primitivas), `pptsx-types/` (os `.d.ts` do editor), `ds-kit/` (o kit) e `fonts/`. O `build.rs` só gera as listas com `include_str!`; não copia nada de fora do crate.
</details>

<details>
<summary><b>O subconjunto de TS/JSX</b> que o avaliador executa</summary>
<br>

Avaliado sem motor JS: `const`/`let`, funções e arrows (closures), desestruturação com padrão e rest, spread, template strings, `?.`/`??`, ternário, `if`/`for…of`/`for`/`while`/`switch`/`try`, JSX com componentes, imports ESM (relativos, `pptsx` e `@pptsx/ds-kit` de dentro do binário, pacotes em `node_modules`, JSON), **regex** (`test`, `exec`, `match`, `split`, `replace` com `$1`/função), arrays mutáveis (`push`, `sort`…), métodos de string e número, `Math`, `Object`, `Array`, `JSON`. Tipos do TS são ignorados (a checagem continua com o `tsc` no editor).

Fora do subconjunto, com erro em `arquivo:linha:coluna`: classes e `async` (definir pode, chamar é erro), `this` dentro de função, atribuição em propriedade de objeto, `Date`, `Math.random`. Um orçamento de passos garante que todo deck termina. Funções passadas em props (`<Table cell={fn}>`) rodam ainda no avaliador; a função filha de um `<Container>` roda depois do layout que mede o espaço, no mesmo avaliador.

Componentes são expandidos quando o JSX é avaliado (o pptsx expande no resolve), mas o elemento guarda o componente e as props de origem: `child.type` e `child.props` se comportam como no TS, e um DS pode recriar o filho com outra prop (`jsx(child.type, { ...child.props, size })`).

Concessão pragmática: `node:fs`, `node:path` e `node:url` existem **só para leitura**, porque os design systems leem guias `.md` e listam ícones ao carregar. Escrita, processos e rede não existem. A leitura (`readFileSync`, `readdirSync`) fica restrita à pasta do pacote de cada módulo carregado (o deck, a config, os DS importados: o ancestral mais próximo com `package.json`) e a qualquer `node_modules` acima deles; fora disso é erro com posição. `existsSync` é livre (não lê conteúdo; um DS sonda fontes do sistema com ele).
</details>

<details>
<summary><b>Tudo o que é suportado</b></summary>
<br>

- **Primitivas**: `Deck`, `Slide`, `Box`, `Text` (com `Span`/`B`/`I`/`Sup`/`Sub`, inclusive `fontSize`/`fontFamily` por trecho), `Shape` (as 24 formas), `Image` (`contain`/`cover`), `Path`, `Icon` (vetor ou PNG), `Connector` (reto/cotovelo, setas, rótulo), `Table`, `Chart` (`bar`, `barH`, `stackedBar`, `stackedBarH`, `stackedBar100`, `line`, `area`, `stackedArea`, `pie`, `doughnut`, `scatter`, `combo`), `SlideNumber` (campo `slidenum`), `Container` (o filho é uma função do espaço medido, com `width`/`height`/`aspect`, `narrow`/`short`, `box.pick({ base, narrow, short, small })` e `box.slide`; contenção de tamanho; aninhado até 3 níveis)
- **Layout**: flexbox e CSS Grid (`repeat`, `minmax`, `fr`, `gridColumn`/`gridRow`, `placeItems`), `position: absolute` relativo ao pai, `%`, `margin: auto`, `alignItems: baseline`, ordem de pintura CSS (camadas por `zIndex`, posição, opacidade e transform)
- **CSS do autor**: padding/margin/gap/inset em shorthand, `border` e bordas por lado, `borderRadius`, `opacity` (herdada), `boxShadow`, `rotate`/`transform` (rotate, translate), herança de texto, valores responsivos e `ctx.pick`
- **Fit**: `shrink` por escopo de componente (Text e Table), disparado só por texto/tabela que não cabe ou por palavra partida, em passos de 1 pt; `truncate` com o relayout do pptsx; `maxLines`; `nowrap`; fonte não registrada medida por estimativa
- **paginate**: `Box` (itens), `Text` (linhas, com corte em parágrafo/órfãs) e `Table` (linhas, com `repeatHeader`)
- **Tabela**: `colSpan`/`rowSpan`, cabeçalho de vários níveis, rich text, ícone na célula, `valign`, `cell={fn}`, listras, bordas, colunas pelo conteúdo
- **Deck**: `theme` (cores e fontes do tema do arquivo), `sections`, `footer`, notas com estilo, slides ocultos, hyperlinks, vários tamanhos num build, `backgroundImage`, `document` da config, `textSafety`, `lineBreaks`, `embedFonts`
- **Diagnósticos**: os do pptsx (`text-overflow`, `text-shrunk`, `text-illegible`, `text-truncated`, `text-word-broken`, `box-overflow` inclusive de célula de grid, `out-of-slide`, `table-overflow`, `font-missing`, `font-weight-fallback`, `font-not-embeddable`, `image-upscaled`, `icon-rasterized`, `slide-continued`, `placeholder-unused`, `css-not-emitted`, `container-unsized`, `container-depth`), a causa de cada estouro e o agrupamento de irmãos
- **Validação**: prop da 0.x, cor/id/href/placeholder inválidos e erros de estrutura derrubam o build com a mensagem do pptsx
- **Layouts** (`defineLayout`): cada layout vira um slideLayout do arquivo; nós com `placeholder` saem ligados ao `<p:ph>`, e o conteúdo `fixed` vai para o layout
- **Tema no arquivo**: cores declaradas viram `<a:schemeClr>` e as famílias `+mj-lt`/`+mn-lt`: trocar o tema no PowerPoint repinta o deck
- **lint**: as regras do `pptsx lint` (`legacy-prop`, `keynote-line-height`, `grid-fr`, `off-theme-color`, `structure`) e as dos DS da config, com o mesmo path, agrupamento e junção entre tamanhos
- **Fontes**: Liberation Sans embutida (Arial); fontes do autor pela config, inclusive `.ttc` pelo nome PostScript e busca no sistema; pesos numéricos; embed em EOT
</details>

## O que falta

- `lint --fix` e `lint --from 0.x` (o codemod da 0.x) não existem no binário: a troca é à mão, pela tabela do guia.
- No MCP, `init` e o parâmetro `ds` do `lint` (os DS vêm da config ou do `--ds` do servidor).
- O catálogo visual existe (`pptsx-rs catalog`), mas a comparação de frames contra fixtures (`catalog --check` do kit em Node) não tem equivalente.
- O schema das props de um DS é lido do texto dos tipos, sem checagem: tipos mapeados ou utilitários (`Omit<…>`, `Partial<…>`) e tipos de outro pacote ficam como referência (`x-extends`), não são abertos.
- O taffy ficou só como o tipo `Style` do modelo; trocá-lo por um `Style` próprio é limpeza, não comportamento.
