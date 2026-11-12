import type { TokenSource } from './tokens.ts';
import { type Component, type ComponentCatalogCase, type ComponentMeta, type DsManifest, type LintRule, type Theme } from 'pptsx';
/**
 * `createDs`: monta o `DsManifest` do núcleo a partir das funções dos componentes (o núcleo só guarda os
 * metas; o catálogo precisa das funções, que ficam em `implementations`). O guia junta as partes em
 * markdown; o tema passa pelo `checkTheme` (os avisos ficam em `warnings`).
 */
/** Um caso do catálogo (o do núcleo): nome, props do componente e `fill?`. Fica em `meta.catalog`. */
export type CatalogCase<P = Readonly<Record<string, unknown>>> = ComponentCatalogCase<P>;
export interface DsInput {
    readonly name: string;
    readonly version: string;
    /** Uma ou duas frases: para que o DS serve (o catálogo e o prompt abrem com ela). */
    readonly description?: string;
    readonly components: readonly Component[];
    readonly theme?: Theme;
    readonly rules?: readonly LintRule[];
    /** Markdown do guia: um texto ou partes (cada uma vira uma seção `##`). */
    readonly guide?: string | readonly GuidePart[];
    readonly schema?: Readonly<Record<string, unknown>>;
    readonly examples?: readonly string[];
    /** Os tokens do DS: as cores deles vão para `colors` do manifesto, e o lint deixa de apontá-las como fora do tema. */
    readonly tokens?: TokenSource;
}
export interface GuidePart {
    readonly title: string;
    readonly markdown: string;
}
export interface Ds extends DsManifest {
    /** Funções dos componentes, na ordem de `components` (o catálogo renderiza com elas). */
    readonly implementations: readonly Component[];
    /** Avisos de montagem (contraste do tema). */
    readonly warnings: readonly string[];
}
export declare const metaOfComponent: (component: Component) => ComponentMeta;
/** Guia em partes → um markdown com uma seção `##` por parte (o `guide` do MCP navega por elas). */
export declare const guideOf: (title: string, parts: readonly GuidePart[]) => string;
/**
 * Schema de props sem ler os tipos: o que os casos do catálogo e os `defaults` mostram (nome, tipo do valor,
 * exemplos; obrigatória = presente em todo caso). Fica no formato do `schema` do núcleo. O pptsx-rs completa
 * com os tipos lidos das interfaces de props (`defineComponent<XProps>`); um `schema` passado ao `createDs` vence.
 */
export declare const schemaFromMeta: (meta: ComponentMeta) => Readonly<Record<string, unknown>>;
/** As seções do guia que os componentes trazem (`meta.guide`): para juntar com partes escritas à mão. */
export declare const componentGuides: (components: readonly Component[]) => readonly GuidePart[];
export declare const createDs: (input: DsInput) => Ds;
