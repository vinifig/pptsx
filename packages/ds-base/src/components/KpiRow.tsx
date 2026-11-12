import { Box, type Responsive } from 'pptsx';
import { defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';
import { KpiCard, type KpiCardProps, type KpiSize } from './KpiCard.tsx';

export interface KpiRowProps {
  readonly items: readonly KpiCardProps[];
  /** Tamanho de todos os cartões (o size de um item vence); padrão 'md'. Aceita `{ base: 'md', wide: 'lg' }`. */
  readonly size?: Responsive<KpiSize>;
}

/**
 * Linha de KPI cards de larguras iguais: um grid de uma linha com uma coluna por item.
 * `minmax(0, 1fr)` e não `1fr`: o `1fr` do CSS nunca fica menor que a palavra mais longa
 * (min-content), então um valor como "R$ 52.400.000,00" alargaria a coluna e empurraria os
 * cartões para fora do slide; com mínimo 0 as colunas ficam iguais e o texto encolhe nelas.
 */
function KpiRowView({ items, size }: KpiRowProps) {
  return (
    <Box display="grid" gridTemplateColumns={`repeat(${items.length}, minmax(0, 1fr))`} gap={tokens.space.gap}>
      {items.map((item) => (
        <KpiCard {...(size ? { size } : {})} {...item} />
      ))}
    </Box>
  );
}

const SAMPLE: readonly KpiCardProps[] = [
  { label: 'Receita (R$ mi)', value: '52,4', delta: '+14,2%', trend: 'up' },
  { label: 'Custo por consulta', value: '−22%', delta: 'menor do ano', trend: 'down', sentiment: 'positive' },
  { label: 'NPS interno', value: '62', delta: '−4 pts', trend: 'down' },
  { label: 'Chamados', value: '3.412', delta: '+8%', trend: 'up', sentiment: 'negative' },
];

export const KpiRow = defineComponent<KpiRowProps>(
  {
    name: 'KpiRow',
    ds: '@pptsx/ds-base',
    kind: 'component',
    category: 'Indicadores',
    description: 'Linha de 2 a 4 KpiCards de larguras iguais, num grid de uma linha.',
    audience: ['executive', 'strategic'],
    whenToUse: ['Painel de resultado: os 3 ou 4 números do período.'],
    avoid: ['Mais de 4 itens (regra ds-base/kpi-row-max-4).'],
    tags: ['kpi'],
    catalog: [
      { name: '3-md', audience: ['executive'], props: { items: SAMPLE.slice(0, 3) } },
      { name: '4-sm', audience: ['tactical'], props: { items: SAMPLE, size: 'sm' } },
    ],
  },
  KpiRowView,
);
