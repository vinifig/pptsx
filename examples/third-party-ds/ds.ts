import { createDs } from '@pptsx/ds-kit';
import { Hero, Quote, StatCard } from './components.tsx';
import { rules } from './rules.ts';
import { theme } from './tokens.ts';

export const ds = createDs({
  name: 'aurora-ds',
  version: '0.1.0',
  description: 'DS de terceiro de exemplo (Aurora): capa, indicadores e citação, criado só com o guia do ds-kit.',
  components: [Hero, StatCard, Quote],
  theme,
  rules,
  guide: [
    {
      title: 'Hero',
      markdown: 'Capa do deck: `title` grande e `subtitle` opcional. `tone`: `night` (padrão, fundo ink) ou `day` (fundo mist). Preenche o corpo (`flex=1`). Só no slide 1 (regra `aurora-ds/hero-so-na-capa`).',
    },
    {
      title: 'StatCard',
      markdown: 'Número de destaque: `label`, `value` e `delta` opcional (começando com `-` fica vermelho, senão verde). `tone`: `neutral` | `brand` | `accent`; `size`: `md` | `lg` (aceita responsivo). Use 2 a 4 lado a lado numa `Box flexDirection="row"`.',
    },
    {
      title: 'Quote',
      markdown: 'Citação: `text` e `author`; `tone`: `brand` | `accent` (cor da barra). Slot `avatar` opcional antes do autor. Não ponha aspas no `text` — o componente já coloca.',
    },
  ],
});

export default ds;
