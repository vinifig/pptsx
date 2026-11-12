import type { ComponentMeta, Theme } from './core/types.ts';
import type { LintRule } from './lint/rule.ts';
/**
 * M17: o contrato do que um design system entrega ao núcleo. O núcleo só lê: as regras vão para o
 * `pptsx lint`, o guia e o schema para o MCP (`guide`/`schema` com `source`), o tema pode virar o da
 * config. Montar o manifesto (concatenar guias, gerar schema) é trabalho do `@pptsx/ds-kit`.
 */
export interface DsManifest {
    readonly name: string;
    readonly version: string;
    /** Uma ou duas frases: para que o DS serve (o catálogo e o prompt abrem com ela). */
    readonly description?: string;
    readonly components: readonly ComponentMeta[];
    readonly theme?: Theme;
    readonly rules: readonly LintRule[];
    /**
     * As cores do DS (lista ou árvore de tokens, em qualquer formato de cor CSS). O `off-theme-color` do
     * lint não cobra do autor do deck uma cor que está aqui: o tema do arquivo só tem 10 slots, e um DS
     * com mais tokens sempre sobra. `palette` é sinônimo; `theme.colors` já conta.
     */
    readonly colors?: readonly string[] | Readonly<Record<string, unknown>>;
    readonly palette?: readonly string[] | Readonly<Record<string, unknown>>;
    /** Markdown com `##`/`###` (o `guide` do MCP navega por seção). */
    readonly guide?: string;
    /** Props por componente, no formato do `schema` do núcleo. */
    readonly schema?: Readonly<Record<string, unknown>>;
    /** Decks de exemplo (caminhos), para quem quer ver o DS em uso. */
    readonly examples?: readonly string[];
}
/** Identidade tipada (como `defineConfig`), com as checagens que valem para qualquer DS. */
export declare const defineDs: (manifest: DsManifest) => DsManifest;
