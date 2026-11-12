import { Shape, Slide, Text, type DeckContext } from 'pptsx';
import { defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';
import { circleDiameter } from './decor.ts';

export interface CoverLayoutProps {
  readonly title: string;
  readonly subtitle?: string;
  /** Linha pequena no topo (área, data, evento). */
  readonly caption?: string;
  /** Contexto do deck: mantém a elipse circular e proporcional em qualquer tamanho. */
  readonly ctx?: DeckContext;
}

const { color, font, space } = tokens;

/**
 * Capa: fundo da marca, círculo decorativo no canto superior direito e o bloco de
 * título ancorado embaixo à esquerda. A diagonal círculo ↔ título equilibra o slide
 * em qualquer proporção (em 4:3 a altura extra fica entre os dois, não embaixo).
 * `marginTop="auto"` na barra empurra o bloco para baixo sem um Box só para isso;
 * a cor branca desce do Slide para os textos (herança).
 */
function CoverLayoutView({ title, subtitle, caption, ctx }: CoverLayoutProps) {
  const diameter = circleDiameter(ctx, 0.9);
  return (
    <Slide
      backgroundColor={color.brand}
      padding={`${space.page * 1.25} ${space.page * 1.5} ${space.page * 1.5}`}
      color={color.white}
    >
      {/* zIndex -1: decoração absoluta pinta atrás do fluxo (sem z, pintaria por cima). */}
      <Shape kind="ellipse" position="absolute" right={-diameter * 0.25} top={-diameter * 0.35}
        width={diameter} height={diameter} backgroundColor={color.brandDark} zIndex={-1} />
      {caption ? (
        <Text fontSize={font.small} fontWeight="bold" color={color.accentSoft} maxWidth="55%">{caption}</Text>
      ) : null}
      <Shape kind="rect" width={56} height={6} backgroundColor={color.accent} flexShrink={0}
        marginTop="auto" marginBottom={space.gap} />
      <Text fontSize={font.display} fontWeight="bold" minSize={32}>{title}</Text>
      {subtitle ? (
        <Text fontSize={font.h2} color={color.accentSoft} marginTop={space.gap} fit="truncate" maxLines={2}>{subtitle}</Text>
      ) : null}
    </Slide>
  );
}

export const CoverLayout = defineComponent<CoverLayoutProps>({
    name: 'CoverLayout',
    ds: '@pptsx/ds-base',
    kind: 'template',
    category: 'Slides',
    description: 'Capa: fundo da marca, título grande embaixo à esquerda e linha de contexto no topo.',
    audience: ['executive', 'strategic', 'tactical', 'operational'],
    whenToUse: ['Primeiro slide do deck (regra ds-base/cover-only-first).'],
    avoid: ['No meio do deck: para separar partes, use ContentLayout com um título curto.'],
    tags: ['layout'],
    catalog: [{ name: 'capa', props: { caption: 'Relatório trimestral · Q3 2026', title: 'Resultado do trimestre', subtitle: 'Receita, custo e o que muda no Q4' } }],
  }, CoverLayoutView);
