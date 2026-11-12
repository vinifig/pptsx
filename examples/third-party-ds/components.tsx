import { Box, Text, type Responsive } from 'pptsx';
import { createVariants, defineComponent, slot, type Slots } from '@pptsx/ds-kit';
import { tokens } from './tokens.ts';

const DS = 'aurora-ds';
const c = tokens.color;

// Hero ----------------------------------------------------------------------

const hero = createVariants({
  base: { padding: tokens.space.page, gap: tokens.space.gap, justifyContent: 'center' },
  variants: {
    tone: {
      night: { backgroundColor: c.ink, color: c.paper },
      day: { backgroundColor: c.mist, color: c.ink },
    },
  },
  defaults: { tone: 'night' },
});

interface HeroProps {
  readonly title: string;
  readonly subtitle?: string;
  readonly tone?: 'night' | 'day';
}

export const Hero = defineComponent<HeroProps>(
  {
    name: 'Hero',
    ds: DS,
    kind: 'component',
    category: 'Abertura',
    description: 'Bloco de abertura: título grande e subtítulo sobre o fundo da marca (noite) ou claro (dia).',
    audience: ['executive', 'strategic'],
    whenToUse: ['Capa do deck, dentro de um <Slide> sem padding.'],
    avoid: ['Fora do slide 1 (regra aurora-ds/hero-so-na-capa).'],
    catalog: [
      { name: 'noite', props: { title: 'Aurora 2026', subtitle: 'Resultados do ano' }, fill: true },
      { name: 'dia', props: { title: 'Plano de produto', subtitle: 'Q1 a Q4', tone: 'day' }, fill: true },
    ],
  },
  ({ title, subtitle, tone }) => {
    const text = tone === 'day' ? c.ink : c.paper;
    return (
      <Box {...hero({ tone })} flex={1}>
        <Box width={96} height={8} backgroundColor={c.glow} />
        <Text fontSize={tokens.font.hero} fontWeight="bold" color={text}>{title}</Text>
        {subtitle ? <Text fontSize={tokens.font.title} color={text}>{subtitle}</Text> : null}
      </Box>
    );
  },
);

// StatCard ------------------------------------------------------------------

const card = createVariants({
  base: { padding: tokens.space.inner, borderRadius: 10, gap: 6 },
  variants: {
    tone: {
      neutral: { backgroundColor: c.mist, color: c.ink },
      brand: { backgroundColor: c.aurora, color: c.paper },
      accent: { backgroundColor: c.violet, color: c.paper },
    },
    size: { md: { padding: tokens.space.inner }, lg: { padding: tokens.space.page / 2 } },
  },
  defaults: { tone: 'neutral', size: 'md' },
});

type StatTone = 'neutral' | 'brand' | 'accent';

interface StatCardProps {
  readonly label: string;
  readonly value: string;
  readonly delta?: string;
  readonly tone?: StatTone;
  readonly size?: Responsive<'md' | 'lg'>;
}

// delta começa com '-' = queda; no card colorido o delta herda o branco para manter contraste
const deltaColor = (delta: string, tone: StatTone): string => {
  if (tone !== 'neutral') return c.paper;
  return delta.trim().startsWith('-') ? c.down : c.up;
};

export const StatCard = defineComponent<StatCardProps>(
  {
    name: 'StatCard',
    ds: DS,
    kind: 'widget',
    category: 'Indicadores',
    description: 'Número de destaque com rótulo e variação colorida pelo sinal.',
    audience: ['executive', 'strategic', 'tactical'],
    whenToUse: ['De 2 a 4 lado a lado numa Box em row.'],
    catalog: [
      { name: 'neutro', props: { label: 'Receita', value: 'R$ 52,4 mi', delta: '+12%' } },
      { name: 'queda', props: { label: 'Churn', value: '3,1%', delta: '-0,4 pp' } },
      { name: 'marca-lg', props: { label: 'NPS', value: '62', tone: 'brand', size: 'lg' } },
    ],
  },
  ({ label, value, delta, tone = 'neutral', size }) => {
    const ink = tone === 'neutral' ? c.ink : c.paper;
    return (
      <Box {...card({ tone, size })} flexGrow={1}>
        <Text fontSize={tokens.font.small} color={ink}>{label}</Text>
        <Text fontSize={tokens.font.stat} fontWeight="bold" color={ink}>{value}</Text>
        {delta ? <Text fontSize={tokens.font.body} fontWeight="bold" color={deltaColor(delta, tone)}>{delta}</Text> : null}
      </Box>
    );
  },
  { tone: 'neutral', size: 'md' },
);

// Quote ---------------------------------------------------------------------

// barra lateral como Box: borda só de um lado não apareceu no render (borderLeft* foi ignorado em silêncio)
const quoteBar = createVariants({
  base: { width: 6, borderRadius: 3 },
  variants: {
    tone: {
      brand: { backgroundColor: c.aurora },
      accent: { backgroundColor: c.violet },
    },
  },
  defaults: { tone: 'brand' },
});

interface QuoteProps extends Slots<'avatar'> {
  readonly text: string;
  readonly author: string;
  readonly tone?: 'brand' | 'accent';
}

export const Quote = defineComponent<QuoteProps>(
  {
    name: 'Quote',
    ds: DS,
    kind: 'component',
    category: 'Texto',
    description: 'Citação com barra de cor, autor e avatar opcional.',
    audience: ['executive', 'strategic'],
    avoid: ['Aspas no text: o componente já as coloca.'],
    catalog: [
      { name: 'marca', props: { text: 'O melhor ano da nossa história.', author: 'Ana Lima, CEO' } },
      { name: 'acento', props: { text: 'Os clientes voltaram porque o produto melhorou.', author: 'Bruno Costa, CPO', tone: 'accent' } },
      { name: 'com-avatar', props: { text: 'Simples assim.', author: 'Carla Dias', avatar: <Box width={28} height={28} borderRadius={14} backgroundColor={c.glow} /> } },
    ],
  },
  ({ text, author, tone, avatar }) => (
    <Box flexDirection="row" gap={tokens.space.gap} alignItems="stretch">
      <Box {...quoteBar({ tone })} />
      <Box gap={10} flex={1}>
        <Text fontSize={tokens.font.title} fontStyle="italic" color={c.ink}>{`“${text}”`}</Text>
        <Box flexDirection="row" gap={10} alignItems="center">
          {slot(avatar)}
          <Text fontSize={tokens.font.body} color={c.violet}>{`— ${author}`}</Text>
        </Box>
      </Box>
    </Box>
  ),
);
