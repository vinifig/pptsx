# pptsx — engine TSX → PPTX

Spec do produto: `docs/spec.md`. Organização do trabalho: `docs/work/README.md`. Próximos marcos: `docs/work/milestones/ROADMAP.md`.

**O motor principal é o `packages/pptsx-rs` (Rust).** É nele que o produto evolui: um binário só, sem Node nem `npm` no fluxo de quem usa ou cria um design system. O pacote TypeScript (`packages/pptsx`) foi a referência original e fica como legado: nada do que entra no Rust precisa ser portado para ele.
Mapa dos módulos do Rust: `packages/pptsx-rs/README.md` ("Como funciona"). Contratos do legado em TS: `packages/pptsx/src/core/types.ts` (tipos) + `docs/work/CONTRACTS.md` (funções e regras).

Monorepo: `packages/pptsx-rs` (o motor, workspace Cargo na raiz) e, do legado em Node (npm workspaces, M17), `packages/pptsx` (núcleo em TS), `packages/ds-kit` e `packages/ds-base` (`@pptsx/ds-base`, o DS de exemplo que os decks dourados usam). O `@pptsx/ds-kit` que vale é o de `packages/pptsx-rs/assets/ds-kit/` (embutido no binário). Caminhos de código abaixo (`src/…`, `tests/…`, `examples/…`) são relativos ao pacote. **Se uma funcionalidade só faz sentido com uma marca, não entra no núcleo**; se o núcleo precisa do nome de um componente, falta um gancho. Os DS usam só a API pública de `pptsx` (nada de `pptsx/src/...`).

## Regras invioláveis

- **Não altere `packages/pptsx/src/core/*`** (types, defaults, assets) do legado em TS. Se o contrato não serve, pare e reporte ao orquestrador com a proposta de mudança.
- **Só edite os arquivos/pastas que a sua tarefa possui** (coluna "Dono" no marco). Outros agentes trabalham em paralelo.
- No TS, testes ficam em `tests/` espelhando `src/` e são escritos pelo agente de testes. No Rust, a lógica pura leva testes unitários no próprio arquivo (`#[cfg(test)]`), escritos por quem escreve o código.
- Antes de concluir no Rust: `cargo build --release`, `cargo clippy --release --all-targets -- -D warnings` e `cargo test` sem erro. Mexeu no TS: `npm run typecheck` e `npm test` nos seus arquivos.
- **O Rust é a fonte, o TS não precisa acompanhar.** Mudança de comportamento, de CLI, de guia ou de kit entra em `packages/pptsx-rs` e não é portada para `packages/pptsx`. Os arquivos de `packages/pptsx-rs/assets/` são os originais (`agent-guide.md`, `schema.generated.ts`, `pptsx-types/`, `ds-kit/`): o `build.rs` não copia nada de fora do crate, edite-os ali. As fixtures de `packages/pptsx-rs/fixtures/` são regressão estática do `cargo test` (vieram do TS; `scripts/golden.sh` só serve para conferir contra o legado): se um teste dourado quebrar, investigue a regressão, não regrave a fixture para escondê-la.
- **O binário não depende de Node.** Nada no fluxo de quem usa ou cria um DS pode pedir `npm install`, `npx` ou `node_modules`: `pptsx` e `@pptsx/ds-kit` são módulos do binário, os tipos do editor saem de `pptsx-rs init`/`types`, a skill de `pptsx-rs skill`.

## Convenções de código

- Rust 2024 no motor (`cargo clippy -- -D warnings` limpo). Nos decks, nos DS e no legado: TypeScript ESM, imports relativos com extensão `.ts` (`import { x } from './y.ts'`).
- Programação funcional: funções puras, dados `readonly`/imutáveis, sem classes salvo necessidade real (ex.: erro customizado).
- Early return e guard clauses; nada de `if` aninhado profundo. Funções pequenas com um propósito.
- Toda etapa do pipeline é pura: árvore entra, árvore sai. IO (ler arquivo, fonte, imagem) só nas bordas (no Rust, `main.rs` e os `run` de cada comando; no TS, `render.ts`, `cli.ts` e fábricas como `createFontRegistry`).
- Dependências externas injetadas (ex.: `LayoutEnv`), nunca importadas no meio de uma função pura.
- Design patterns só onde encaixam: Strategy (medidores de texto, políticas de overflow), tabela de despacho por `kind` (emissores), Factory (`definePrimitive`, `create*`).
- Padrões numéricos e de props têm um lugar só (constantes nomeadas no módulo dono, no Rust; `src/core/defaults.ts`, no TS); nunca repita números mágicos.
- Unidade interna: pt (1 pol = 72 pt). Conversão para polegadas só no emissor PPTX; para px só no preview.
- Identificadores em inglês; comentários em português e só para explicar o **porquê**.
- No TS, cada pasta de `src/` expõe sua API pública por `index.ts` e não há `any` exceto onde o contrato já declara (`Component<P = any>`). No Rust, `pub` só no que outro módulo usa.

## Comandos

- `cargo build --release` — o binário (`target/release/pptsx-rs`); `cargo test` e `cargo clippy --release --all-targets -- -D warnings` na raiz
- `target/release/pptsx-rs build <deck.tsx> --size 16x9,4x3 --out dist/` — `.pptx`, PNGs e diagnósticos; `check --json` só diagnostica
- `target/release/pptsx-rs init <pasta> [--ds <caminho|pacote>]` e `types <pasta>` — pasta de deck e tipos do editor, sem npm
- `target/release/pptsx-rs catalog` (DS do `pptsx.registry.json`; `registry add <ds>` registra) — o catálogo visual em `dist/catalog/index.html`; contrato em `docs/ds-catalog.md`
- `target/release/pptsx-rs skill --ds <pasta do DS> --name <nome> --out <pasta>` — a skill autocontida (binário + DS)
- `scripts/keynote-export.sh <arquivo.pptx> <pasta-saida>` — exporta slides via Keynote (validação visual real)
- Legado em TS: `npm test` (vitest), `npm run typecheck` (tsc), `node packages/pptsx/bin/pptsx.mjs build …`, `node packages/ds-kit/bin/pptsx-ds.mjs catalog …`
