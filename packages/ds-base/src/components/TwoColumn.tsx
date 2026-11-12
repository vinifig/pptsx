import { Box, Text, mapResponsive, type Responsive } from 'pptsx';
import { defineComponent, slot, type Slots } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';

/** Slots `left` e `right` (convenção do ds-kit): cada um é um Child; vazio, a coluna fica vazia. */
export interface TwoColumnProps extends Slots<'left' | 'right'> {
  /**
   * true empilha (left em cima, right embaixo); padrão false (lado a lado).
   * Responsivo: `stacked={{ base: false, tall: true }}` empilha em 4:3.
   */
  readonly stacked?: Responsive<boolean>;
  /** Fração da largura da coluna esquerda lado a lado; padrão 0.5. Empilhado, a de cima fica com a altura do conteúdo. */
  readonly ratio?: number;
}

const { space } = tokens;

/**
 * Duas colunas num grid que ocupa o resto do corpo (flex 1).
 * Lado a lado: colunas `ratio fr` e `1 − ratio fr` (com mínimo 0, para uma palavra longa não
 * alargar a coluna além da proporção) numa linha que é a altura toda; cada item
 * estica na célula (o padrão do grid), então um Chart ocupa a coluna inteira e uma lista
 * começa no topo, alinhada com o topo da imagem/gráfico ao lado.
 * Empilhado: uma coluna, linha de cima `auto` (altura do conteúdo) e a de baixo com o resto
 * (`minmax(0, 1fr)`): dividir a altura pela proporção deixaria buracos.
 * Template e gap derivam de stacked chave a chave, então um stacked responsivo continua
 * responsivo até a primitiva e o componente não precisa de ctx.
 */
function TwoColumnView({ left, right, stacked = false, ratio = 0.5 }: TwoColumnProps) {
  const by = <T,>(side: T, column: T): Responsive<T> => mapResponsive(stacked, (s) => (s ? column : side));
  return (
    <Box display="grid" flex={1}
      gridTemplateColumns={by(`minmax(0, ${ratio}fr) minmax(0, ${1 - ratio}fr)`, 'minmax(0, 1fr)')}
      gridTemplateRows={by('minmax(0, 1fr)', 'auto minmax(0, 1fr)')}
      gap={by(space.gap * 1.5, space.gap)}>
      {slot(left)}
      {slot(right)}
    </Box>
  );
}

export const TwoColumn = defineComponent<TwoColumnProps>(
  {
    name: 'TwoColumn',
    ds: '@pptsx/ds-base',
    kind: 'structure',
    category: 'Grade',
    description: 'Duas colunas (ou empilhadas em 4:3), com proporção ajustável: argumento de um lado, evidência do outro.',
    audience: ['executive', 'strategic', 'tactical', 'operational'],
    whenToUse: ['Texto + gráfico, antes/depois, problema/solução.'],
    avoid: ['Três ou mais colunas: use um grid de Box.'],
    tags: ['layout'],
    catalog: [
      { name: 'lado-a-lado', fill: true, props: { left: <Text fontSize={20}>Texto à esquerda, com a lista ou o argumento.</Text>, right: <Box backgroundColor={tokens.color.surface} flex={1} /> } },
      { name: 'empilhado', fill: true, props: { stacked: true, left: <Text fontSize={20}>Em cima, a altura do conteúdo.</Text>, right: <Box backgroundColor={tokens.color.surface} flex={1} /> } },
    ],
  },
  TwoColumnView,
);
