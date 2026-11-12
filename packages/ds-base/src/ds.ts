import { createDs } from '@pptsx/ds-kit';
import { Bullets, Gantt, KpiCard, KpiRow, TwoColumn } from './components/index.ts';
import { guideParts } from './guide.ts';
import { ClosingLayout, ContentLayout, CoverLayout } from './layouts/index.ts';
import { rules } from './rules.ts';
import { PROPS_SCHEMA } from './schema.generated.ts';
import { theme } from './theme.ts';

/** O manifesto do ds-base: o que o núcleo (lint, MCP) e o catálogo do ds-kit leem. */
export const ds = createDs({
  name: '@pptsx/ds-base',
  version: '0.1.0',
  description: 'Design system de exemplo do pptsx: templates de capa, conteúdo e encerramento, indicadores, listas, colunas e um Gantt de roadmap.',
  components: [Bullets, KpiCard, KpiRow, TwoColumn, Gantt, CoverLayout, ContentLayout, ClosingLayout],
  theme,
  rules,
  guide: guideParts(),
  schema: PROPS_SCHEMA,
  examples: ['examples/demo.tsx', 'examples/showcase.tsx', 'examples/gallery/safra.tsx'],
});

export default ds;
