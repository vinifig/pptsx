import { Box, Text, type Responsive } from 'pptsx';
import { createVariants, defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';

export type KpiSize = 'sm' | 'md' | 'lg';
export type KpiTrend = 'up' | 'down';
export type KpiSentiment = 'positive' | 'negative' | 'neutral';

export interface KpiCardProps {
  readonly label: string;
  readonly value: string;
  readonly delta?: string;
  /** Direção da variação: só define a seta (▲/▼). */
  readonly trend?: KpiTrend;
  /** Se a variação é boa ou ruim: define a cor. Padrão: up → positive, down → negative, sem trend → neutral. */
  readonly sentiment?: KpiSentiment;
  /**
   * Escala do cartão; padrão 'md'. Use 'sm' para 4+ cartões ou valores longos, 'lg' com largura/altura sobrando.
   * Aceita `{ base: 'md', tall: 'lg' }`.
   */
  readonly size?: Responsive<KpiSize>;
  /**
   * Variação na mesma linha do valor, alinhada pela baseline ("52,4  ▲ +14,2%").
   * Padrão false (variação embaixo). Use com variações curtas: a linha precisa caber valor + variação.
   */
  readonly inlineDelta?: boolean;
}

const { color, font, space } = tokens;

/** Medidas por tamanho (pt). md é o cartão original; sm e lg escalam padding e tipografia juntos. */
type KpiMetrics = { padding: number; gap: number; label: number; value: number; minValue: number };
const SIZES: Readonly<Record<KpiSize, KpiMetrics>> = {
  sm: { padding: space.tight + 4, gap: space.tight * 0.6, label: font.caption + 1, value: font.h1, minValue: 22 },
  md: { padding: space.gap, gap: space.tight, label: font.small, value: font.display, minValue: 28 },
  lg: { padding: space.gap, gap: space.tight * 1.2, label: font.small + 2, value: font.display + 12, minValue: 36 },
};

/** Uma opção por tamanho, a partir das medidas. */
const perSize = <T,>(pick: (metrics: KpiMetrics) => T): Readonly<Record<KpiSize, T>> => ({ sm: pick(SIZES.sm), md: pick(SIZES.md), lg: pick(SIZES.lg) });

/** Variantes de tamanho: as medidas do cartão (padding, gap, corpo do label) e as do valor (um Text). */
const cardBySize = createVariants({
  variants: { size: perSize((m) => ({ padding: m.padding, gap: m.gap, fontSize: m.label })) },
  defaults: { size: 'md' },
});
const valueBySize = createVariants({
  for: 'text',
  variants: { size: perSize((m) => ({ fontSize: m.value, minSize: m.minValue })) },
  defaults: { size: 'md' },
});

const SENTIMENT_COLOR: Readonly<Record<KpiSentiment, string>> = {
  positive: color.positive,
  negative: color.negative,
  neutral: color.muted,
};

const defaultSentiment = (trend: KpiTrend | undefined): KpiSentiment =>
  trend === 'up' ? 'positive' : trend === 'down' ? 'negative' : 'neutral';

const trendMark = (trend: KpiTrend | undefined): string =>
  trend === 'down' ? '▼ ' : trend === 'up' ? '▲ ' : '';

/**
 * Cartão de indicador: label, valor grande e variação.
 * A seta vem de trend e a cor de sentiment: "Custo ▼ −22%" pode ser verde e
 * "Chamados ▲ +22%" vermelho.
 * O cartão define fontSize (o do label) e cor (muted) uma vez; os textos herdam e só dizem
 * o que muda. `flex={1}`: numa row flex os cartões dividem a largura; numa célula de grid
 * (KpiRow) não faz nada e o cartão ocupa a célula.
 */
function KpiCardView({ label, value, delta, trend, sentiment, size = 'md', inlineDelta = false }: KpiCardProps) {
  // As variantes derivam de size chave a chave: um size responsivo continua responsivo até a primitiva.
  const card = cardBySize({ size });
  const valueStyle = valueBySize({ size });
  const gap = card.gap;
  const valueText = (
    <Text fontSize={valueStyle.fontSize} fontWeight="bold" color={color.ink} minSize={valueStyle.minSize}>{value}</Text>
  );
  const deltaText = delta ? (
    <Text fontWeight="bold" color={SENTIMENT_COLOR[sentiment ?? defaultSentiment(trend)]}>{trendMark(trend) + delta}</Text>
  ) : null;
  // baseline: a 1ª linha da variação pequena senta na mesma linha de base do valor grande,
  // sem calcular deslocamento a partir dos tamanhos (que mudam quando o fit encolhe o valor).
  const body = inlineDelta && deltaText ? (
    <Box display="flex" alignItems="baseline" gap={gap}>{valueText}{deltaText}</Box>
  ) : [valueText, deltaText];
  return (
    <Box flex={1} backgroundColor={color.surface} borderRadius={8} padding={card.padding} gap={gap}
      fontSize={card.fontSize} color={color.muted}>
      <Text>{label}</Text>
      {body}
    </Box>
  );
}

export const KpiCard = defineComponent<KpiCardProps>(
  {
    name: 'KpiCard',
    ds: '@pptsx/ds-base',
    kind: 'widget',
    category: 'Indicadores',
    description: 'Um indicador: rótulo, valor grande e variação com seta e cor pelo sentimento.',
    audience: ['executive', 'strategic', 'tactical'],
    whenToUse: ['Um número que é a mensagem do slide.', 'Peça de uma linha de indicadores (KpiRow) ou de uma coluna do TwoColumn.'],
    avoid: ['Mais de 4 lado a lado: use KpiRow com size="sm" ou uma tabela.'],
    tags: ['kpi'],
    catalog: [
      { name: 'md', audience: ['executive'], props: { label: 'Receita (R$ mi)', value: '52,4', delta: '+14,2%', trend: 'up' } },
      { name: 'sm-custo', audience: ['tactical'], props: { label: 'Custo por consulta', value: '−22%', delta: 'menor do ano', trend: 'down', sentiment: 'positive', size: 'sm' } },
      { name: 'lg-inline', props: { label: 'NPS interno', value: '62', delta: '−4 pts', trend: 'down', size: 'lg', inlineDelta: true } },
    ],
  },
  KpiCardView,
);
