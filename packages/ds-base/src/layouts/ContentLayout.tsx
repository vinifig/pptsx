import { Box, Slide, Text, type Children, type SlideProps } from 'pptsx';
import { defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';

export interface ContentLayoutProps {
  readonly title: string;
  /** Posição vertical do conteúdo no corpo; padrão 'start' (logo abaixo do título). */
  readonly justify?: 'start' | 'center';
  /** Notas do apresentador (texto ou rich text); numa paginação vão para todas as partes. */
  readonly notes?: SlideProps['notes'];
  /** M15: slide oculto (reserva). */
  readonly hidden?: boolean;
  readonly children?: Children;
}

const { color, font, space } = tokens;

/** Barra do título: 6 pt de borda + 14 pt até o texto, a mesma geometria da antiga barra-Shape. */
const TITLE_BAR = { width: 6, gap: 14 } as const;

/**
 * Slide de conteúdo: título com barra de destaque (borderLeft) e corpo flexível que ocupa o resto.
 * justify="center" centraliza conteúdo curto (ex.: uma lista) na altura que sobra,
 * em vez de deixar a metade de baixo vazia em slides mais altos (4:3).
 * A cor do texto (ink) desce do Slide para todo o conteúdo.
 */
function ContentLayoutView({ title, justify = 'start', notes, hidden, children }: ContentLayoutProps) {
  return (
    <Slide backgroundColor={color.white} padding={space.page} gap={space.gap * 1.5} color={color.ink} {...(notes ? { notes } : {})} {...(hidden ? { hidden } : {})}>
      <Text fontSize={font.h1} fontWeight="bold" minSize={24}
        borderLeft={`${TITLE_BAR.width}px solid ${color.accent}`} paddingLeft={TITLE_BAR.gap}>{title}</Text>
      <Box flex={1} justifyContent={justify === 'center' ? 'center' : 'flex-start'}>{children}</Box>
    </Slide>
  );
}

export const ContentLayout = defineComponent<ContentLayoutProps>({
    name: 'ContentLayout',
    ds: '@pptsx/ds-base',
    kind: 'template',
    category: 'Slides',
    description: 'Slide de conteúdo: título com barra de destaque e corpo flexível para os componentes.',
    audience: ['executive', 'strategic', 'tactical', 'operational'],
    whenToUse: ['Todo slide de conteúdo: o corpo recebe estruturas, gráficos e componentes.'],
    avoid: ['Conteúdo que já tem título próprio grande (uma capa de seção): use CoverLayout.'],
    tags: ['layout'],
    catalog: [{ name: 'conteúdo', props: { title: 'O que mudou no trimestre', children: <Text fontSize={20}>O corpo recebe a estrutura do slide: TwoColumn, KpiRow, Gantt ou Bullets.</Text> } }],
  }, ContentLayoutView);
