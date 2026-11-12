import { Shape, Slide, Text, type DeckContext } from 'pptsx';
import { defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';
import { circleDiameter } from './decor.ts';

export interface ClosingLayoutProps {
  readonly title: string;
  readonly subtitle?: string;
  readonly contact?: string;
  /** Contexto do deck: mantém a elipse circular e proporcional em qualquer tamanho. */
  readonly ctx?: DeckContext;
}

const { color, font, space } = tokens;

/**
 * Encerramento: fundo da marca, tudo centralizado (textAlign e cor herdados do Slide).
 * O subtítulo tem largura máxima (medida de leitura) e fica no centro, longe do círculo
 * do canto, que assim nunca passa por trás do texto.
 */
function ClosingLayoutView({ title, subtitle, contact, ctx }: ClosingLayoutProps) {
  const diameter = circleDiameter(ctx, 0.75);
  return (
    <Slide backgroundColor={color.brand} padding={space.page * 1.5} justifyContent="center" gap={space.gap}
      textAlign="center" color={color.accentSoft}>
      <Shape kind="ellipse" position="absolute" left={-diameter * 0.4} bottom={-diameter * 0.4}
        width={diameter} height={diameter} backgroundColor={color.brandDark} zIndex={-1} />
      <Text fontSize={font.display} fontWeight="bold" color={color.white} minSize={32}>{title}</Text>
      <Shape kind="rect" width={56} height={6} backgroundColor={color.accent} alignSelf="center" flexShrink={0} />
      {subtitle ? (
        <Text fontSize={font.h2} fit="truncate" maxLines={2} width="80%" alignSelf="center">{subtitle}</Text>
      ) : null}
      {contact ? <Text fontSize={font.small} marginTop={space.tight}>{contact}</Text> : null}
    </Slide>
  );
}

export const ClosingLayout = defineComponent<ClosingLayoutProps>({
    name: 'ClosingLayout',
    ds: '@pptsx/ds-base',
    kind: 'template',
    category: 'Slides',
    description: 'Encerramento: fundo da marca, mensagem centralizada e contato.',
    audience: ['executive', 'strategic', 'tactical', 'operational'],
    whenToUse: ['Último slide do deck: agradecimento, próximos passos ou contato.'],
    avoid: ['Conclusões com dados: ponha num ContentLayout antes do encerramento.'],
    tags: ['layout'],
    catalog: [{ name: 'encerramento', props: { title: 'Obrigado', subtitle: 'Próximos passos no canal #resultados', contact: 'time-dados@empresa.com' } }],
  }, ClosingLayoutView);
