# Contrato do catálogo de design systems

Um design system (DS) do pptsx é um módulo TS que exporta `ds` (ou o default) feito com `createDs` do `@pptsx/ds-kit` (ou `defineDs` do núcleo). Este documento define o que o manifesto e cada componente declaram para aparecer no **catálogo visual** (`pptsx-rs catalog`), no **índice para agentes** (ferramenta `catalog` do MCP) e no **prompt** do DS (`prompt.md` e o `references/ds.md` da skill).

Tudo aqui é opcional e aditivo: um DS antigo continua compilando, e o catálogo diz o que falta como recomendação.

## Por que assim

- **A fonte é o próprio `defineComponent`.** Os metadados moram ao lado do código do componente, e não num arquivo de catálogo à parte que envelhece. O binário lê o manifesto pelo mesmo avaliador do deck; nada de Node.
- **O catálogo renderiza com o motor de verdade.** Cada caso de `meta.catalog` vira um slide de um deck gerado e passa pelo layout, fit e diagnósticos do build. O que aparece na galeria é o que sai no `.pptx`, com os mesmos avisos de estouro.
- **O mesmo dado serve a pessoas e a agentes.** O visualizador, o `catalog.json`, o `prompt.md` e o `catalog` do MCP saem do mesmo manifesto.

## Camadas (`kind`)

O agente monta um slide de cima para baixo: escolhe o **template**, preenche o corpo com **estruturas**, e as estruturas com **gráficos**, **componentes** e **widgets**.

| `kind` | o que é | exemplos no ds-base |
|---|---|---|
| `template` | O slide inteiro: o componente devolve o `<Slide>`. **Recomendado**: o catálogo avisa quando o DS não tem nenhum. | `CoverLayout`, `ContentLayout`, `ClosingLayout` |
| `structure` | Organiza outros componentes numa região do slide (colunas, grade, faixa). | `TwoColumn` |
| `chart` | Apresenta dados: gráfico, tabela, Gantt de roadmap, linha do tempo. | `Gantt` |
| `component` (padrão) | Um bloco de conteúdo com sentido próprio. | `Bullets`, `KpiRow` |
| `widget` | Peça atômica, usada sozinha ou dentro de outros. | `KpiCard` |

Sem `kind`, vale `component`; `slide: true` sem `kind` é lido como `template`.

## Público (`audience`)

Para quem o componente serve: `executive`, `strategic`, `tactical`, `operational` (outros valores valem). Cada **caso** do catálogo pode trazer o seu `audience`: é assim que um componente declara variações, como o Gantt tático (uma linha por tarefa, com dono) e o executivo (uma barra por frente, com o pior status). O visualizador filtra por público, e o `catalog` do MCP aceita `audience` para o agente ver só o que serve à plateia do deck.

## Metadados de um componente

```tsx
export const Gantt = defineComponent<GanttProps>(
  {
    name: 'Gantt',
    kind: 'chart',
    category: 'Roadmap',                       // agrupa dentro da camada
    description: 'Roadmap em Gantt nativo…',   // uma frase: o que é
    audience: ['tactical', 'executive'],
    whenToUse: ['Roadmap, cronograma ou plano de entregas…'],
    avoid: ['Mais de 12 linhas no modo tático…'],
    tags: ['roadmap'],
    catalog: [
      { name: 'tático', audience: ['tactical'], description: 'Uma linha por tarefa…', props: { periods, tasks } },
      { name: 'executivo', audience: ['executive'], props: { periods, tasks, detail: 'executive' } },
    ],
  },
  GanttView,
);
```

| campo | tipo | para quê |
|---|---|---|
| `kind` | `ComponentKind` | a camada (acima) |
| `description` | `string` | 1ª linha do componente no catálogo e no prompt |
| `category` | `string` | subgrupo no visualizador |
| `audience` | `Audience[]` | públicos atendidos |
| `whenToUse` / `avoid` | `string[]` | o que o agente decide primeiro: quando escolher, o que não fazer |
| `guide` | `string` (markdown) | guia longo do componente (vira seção do guia do DS) |
| `catalog[]` | `{ name, props, fill?, description?, audience? }` | os casos renderizados; `fill: true` = o componente preenche o corpo do slide |

As **props** não são declaradas aqui: o binário as lê da interface do `defineComponent<XProps>` (tipo, obrigatória, JSDoc, padrão). Um `schema` passado ao `createDs` vence.

No manifesto (`createDs`), `description` diz para que o DS serve.

## Registro (`pptsx.registry.json`)

A lista dos DS que o catálogo e o MCP enxergam sem `--ds`, achada nesta pasta ou acima. Caminhos são relativos ao arquivo; um nome de pacote também vale.

```json
{
  "designSystems": [
    { "id": "ds-base", "ds": "packages/ds-base" },
    { "id": "aurora", "ds": "examples/third-party-ds/ds.ts" }
  ]
}
```

```bash
pptsx-rs registry add ../marca-ds --id marca     # carrega o DS antes de gravar
pptsx-rs registry list                           # confere que cada um avalia
pptsx-rs registry remove marca
```

É JSON, e não TS, porque não executa nada: só diz onde cada DS mora. O `id` dá o nome da pasta de saída e a âncora do visualizador.

## O catálogo

```bash
pptsx-rs catalog                    # todos os DS do registro → dist/catalog/
pptsx-rs catalog --ds ../marca-ds   # um DS avulso, sem registro
pptsx-rs catalog --json             # só o contrato (pptsx.catalog/1), sem construir nada
pptsx-rs catalog --prompt           # só o prompt de todos os DS
```

```text
dist/catalog/
├── index.html                   o visualizador: um arquivo, SVGs embutidos, abre direto do disco
├── catalog.json                 o contrato exportado (pptsx.catalog/1)
└── <id>/
    ├── prompt.md                o prompt do DS para um agente (o mesmo ds.md da skill)
    ├── catalog.16x9.pptx        um slide por caso, para abrir no PowerPoint/Keynote
    ├── previews/<tam>/N.svg     o SVG de cada slide, com state.json e diagnostics.json do build
    └── deck/catalog.tsx         o deck gerado (não edite: refeito a cada execução)
```

Um caso que não renderiza não derruba o catálogo: o deck é refeito caso a caso e o caso quebrado aparece com o erro. As recomendações (sem template, sem `audience`, sem `description`, sem caso de catálogo) saem no terminal e no topo do DS no visualizador.

### `catalog.json` (pptsx.catalog/1)

```text
{ format, generator, sizes: [{ name, width, height }],
  designSystems: [{ id, source, name, version, description?, theme?: { colors }, tiers: { template, structure, chart, component, widget },
                    notes: [recomendação], rules: [{ id, severity }], guide?, prompt?, pptx?: { <tam>: arquivo }, error?,
                    components: [{ name, displayName?, description?, category?, kind, slide?, tags, audience, whenToUse, avoid,
                                   guide?, signature, schema?, cases: [{ name, description?, fill?, audience, usage,
                                   previews?: { <tam>: [svg] }, diagnostics?: [{ size, severity, code, path, message }], error? }] }] }] }
```

O formato só muda de versão quando um campo existente muda de sentido; campos novos entram na mesma versão.

## Para agentes

- **MCP:** `catalog({ source?, audience?, kind? })` devolve o índice por camada (assinatura, para que serve, público, quando usar, o que evitar e o JSX de cada caso). Depois, `schema` dá as props completas de um componente e `guide` o guia do DS.
- **Sem MCP:** `pptsx-rs catalog --prompt` imprime o mesmo conteúdo do `prompt.md`.
