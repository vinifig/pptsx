import { COMPONENT_META, defineDs } from 'pptsx';
import { checkTheme } from './theme.js';
import { colorsOf } from './tokens.js';
export const metaOfComponent = (component) => {
    const meta = component[COMPONENT_META];
    if (meta === undefined)
        throw new Error(`createDs: "${component.name || 'anônimo'}" não foi criado com defineComponent (o DS precisa da identidade dele)`);
    return meta;
};
/** Guia em partes → um markdown com uma seção `##` por parte (o `guide` do MCP navega por elas). */
export const guideOf = (title, parts) => [`# ${title}`, ...parts.map((part) => `## ${part.title}\n\n${part.markdown.trim()}`)].join('\n\n') + '\n';
const typeOfValue = (value) => {
    if (Array.isArray(value))
        return 'array';
    if (value === null)
        return 'null';
    if (typeof value === 'object')
        return '$$typeof' in value || 'type' in value ? 'element' : 'object';
    return typeof value;
};
/**
 * Schema de props sem ler os tipos: o que os casos do catálogo e os `defaults` mostram (nome, tipo do valor,
 * exemplos; obrigatória = presente em todo caso). Fica no formato do `schema` do núcleo. O pptsx-rs completa
 * com os tipos lidos das interfaces de props (`defineComponent<XProps>`); um `schema` passado ao `createDs` vence.
 */
export const schemaFromMeta = (meta) => {
    const cases = (Array.isArray(meta.catalog) ? meta.catalog : []);
    const defaults = (meta.defaults ?? {});
    const samples = [...cases.map((item) => item.props), defaults];
    const names = [...new Set(samples.flatMap((props) => Object.keys(props)))];
    const properties = Object.fromEntries(names.map((name) => {
        const values = samples.map((props) => props[name]).filter((value) => value !== undefined);
        const types = [...new Set(values.map(typeOfValue))];
        const examples = values.filter((value) => typeOfValue(value) !== 'element' && typeOfValue(value) !== 'object').slice(0, 3);
        return [name, { type: types.join(' | '), examples: [...new Set(examples.map((value) => JSON.stringify(value)))].map((text) => JSON.parse(text)) }];
    }));
    const required = cases.length === 0 ? [] : names.filter((name) => cases.every((item) => item.props[name] !== undefined) && !(name in defaults));
    return { type: 'object', properties, required, 'x-source': 'catálogo e defaults' };
};
const guidePartsOf = (metas) => metas.flatMap((meta) => (typeof meta.guide === 'string' ? [{ title: meta.displayName ?? meta.name, markdown: meta.guide }] : []));
/** As seções do guia que os componentes trazem (`meta.guide`): para juntar com partes escritas à mão. */
export const componentGuides = (components) => guidePartsOf(components.map(metaOfComponent));
/** Guia montado dos `meta.guide` (markdown) dos componentes, uma seção por componente. */
const guideFromMetas = (name, metas) => {
    const parts = guidePartsOf(metas);
    return parts.length === 0 ? undefined : guideOf(name, parts);
};
export const createDs = (input) => {
    const metas = input.components.map(metaOfComponent);
    const guide = input.guide === undefined ? guideFromMetas(input.name, metas) : typeof input.guide === 'string' ? input.guide : guideOf(input.name, input.guide);
    const schema = input.schema ?? Object.fromEntries(metas.map((meta) => [meta.name, schemaFromMeta(meta)]));
    const manifest = defineDs({
        name: input.name,
        version: input.version,
        ...(input.description === undefined ? {} : { description: input.description }),
        components: metas.map((meta) => ({ ...meta, ds: meta.ds ?? input.name })),
        rules: input.rules ?? [],
        ...(input.theme === undefined ? {} : { theme: input.theme }),
        ...(guide === undefined ? {} : { guide }),
        schema,
        ...(input.examples === undefined ? {} : { examples: input.examples }),
    });
    return Object.freeze({ ...manifest, ...(input.tokens === undefined ? {} : { colors: colorsOf(input.tokens) }), implementations: input.components, warnings: input.theme === undefined ? [] : checkTheme(input.theme) });
};
