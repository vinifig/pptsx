/**
 * CONTRATOS DO PIPELINE — fonte da verdade compartilhada entre os módulos.
 *
 * Regra: nenhum módulo redefine estes tipos. Mudança aqui só pelo orquestrador
 * (ver docs/work/CONTRACTS.md). Unidade interna é sempre pt (1 pol = 72 pt).
 *
 * Fluxo por tamanho de slide:
 *   DeckDefinition(ctx) -> SlideElement            (jsx)
 *   SlideElement        -> ResolvedDeck            (resolve)
 *   ResolvedSlide       -> PositionedSlide         (layout, com LayoutEnv)
 *   loop layout <-> ajuste                         (fit) -> FitResult
 *   PositionedDeck      -> Buffer .pptx            (emit)
 *   PositionedSlide     -> SVG -> PNG / overview   (preview)
 */
import type { CssBoxPropsAuthor, ConnectorPropsAuthor, CssChartPropsAuthor, CssIconPropsAuthor, CssImagePropsAuthor, CssPathPropsAuthor, CssSlideNumberPropsAuthor, CssContainerQueryPropsAuthor, CssShapePropsAuthor, CssSlideProps, CssSpanProps, CssTablePropsAuthor, CssTextPropsAuthor } from './css-props.ts';
/** Pontos tipográficos. 1 pol = 72 pt. */
export type Pt = number;
/** Número = pt; string = percentual do pai (ex.: "50%"). */
export type Length = Pt | `${number}%`;
/** Cor hex '#RRGGBB'. */
export type Color = string;
export interface Rect {
    readonly x: Pt;
    readonly y: Pt;
    readonly width: Pt;
    readonly height: Pt;
}
export interface Dimensions {
    readonly width: number;
    readonly height: number;
}
export type SizePresetName = '16x9' | '16x10' | '4x3' | 'wide';
/** Preset ou tamanho custom em POLEGADAS. */
export type SizeSpec = SizePresetName | {
    readonly width: number;
    readonly height: number;
};
export interface SlideSize {
    /** Nome do preset ou 'custom-<w>x<h>' (ex.: 'custom-10x7.5'). */
    readonly name: string;
    readonly width: Pt;
    readonly height: Pt;
}
/** Contexto que o deck recebe; um por tamanho renderizado. */
export interface DeckContext {
    readonly name: string;
    readonly width: Pt;
    readonly height: Pt;
    /** largura / altura */
    readonly aspect: number;
    /** min(width/720, height/405) — relativo ao 16:9 de referência. */
    readonly scale: number;
    /**
     * M9: resolve um valor responsivo em código (contagem de itens, textos alternativos).
     * Mesma regra de precedência das props: nome do tamanho > wide > tall > base.
     */
    readonly pick: <T>(value: Responsive<T>) => T;
}
/**
 * Breakpoints nomeados, derivados de ctx (ver core/responsive.ts):
 *  - 'tall'  ⇔ ctx.aspect < DEFAULTS.breakpoints.tallAspect (4:3, 16:10 e customs "quadrados");
 *  - 'wide'  ⇔ ctx.scale  > DEFAULTS.breakpoints.wideScale  (o preset 'wide' e customs maiores);
 *  - nome exato do tamanho ('16x9', '4x3', 'custom-10x7.5') ⇔ ctx.name.
 * Precedência: nome exato > wide > tall > base ('wide' mexe em tipografia, 'tall' em estrutura).
 */
export type Breakpoint = 'tall' | 'wide' | SizePresetName | `custom-${string}`;
/** Objeto responsivo: `base` obrigatória; é o que o distingue de `padding: { top }` e `background: { image }`. */
export type ResponsiveValue<T> = {
    readonly base: T;
} & {
    readonly [name in Breakpoint]?: T;
};
/** Um valor concreto ou um objeto por breakpoint. Resolvido no resolve; a árvore resolvida só tem valores concretos. */
export type Responsive<T> = T | ResponsiveValue<T>;
/** Props que nunca são responsivas (estrutura e identidade do nó). */
export type NonResponsiveKey = 'children' | 'src' | 'kind' | 'notes';
/** Versão responsiva de uma interface de props: toda chave fora de NonResponsiveKey aceita Responsive<X>. */
export type ResponsiveProps<P> = {
    readonly [K in keyof P]: K extends NonResponsiveKey ? P[K] : Responsive<Exclude<P[K], undefined>>;
};
export type Spacing = Pt | {
    readonly top?: Pt;
    readonly right?: Pt;
    readonly bottom?: Pt;
    readonly left?: Pt;
};
export type FlexDirection = 'row' | 'column';
/** 'baseline' (M8) só vale em row: alinha pela baseline da 1ª linha de texto; em column vira 'start'. */
export type Align = 'start' | 'center' | 'end' | 'stretch' | 'baseline';
export type Justify = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
/** Props de item flex aceitas por Box, Text, Image e Shape. */
export interface FlexItemProps {
    readonly width?: Length;
    readonly height?: Length;
    readonly minWidth?: Length;
    readonly maxWidth?: Length;
    readonly minHeight?: Length;
    readonly maxHeight?: Length;
    readonly grow?: number;
    readonly shrink?: number;
    readonly alignSelf?: Align;
    /** 'absolute' sai do fluxo flex; usado para decoração de fundo. */
    readonly position?: 'relative' | 'absolute';
    readonly top?: Length;
    readonly right?: Length;
    readonly bottom?: Length;
    readonly left?: Length;
    /**
     * M12: identificador do nó para `Connector from/to`. Único no slide (duplicado → ResolveError);
     * não muda layout nem pintura.
     */
    readonly id?: string;
}
/** Props de contêiner flex aceitas por Box e Slide. */
export interface ContainerProps {
    /** padrão 'column' */
    readonly direction?: FlexDirection;
    readonly gap?: Pt;
    readonly padding?: Spacing;
    /** align-items; padrão 'stretch' */
    readonly align?: Align;
    /** justify-content; padrão 'start' */
    readonly justify?: Justify;
}
export interface StrokeProps {
    readonly fill?: Color;
    readonly stroke?: Color;
    /** padrão 1 pt quando stroke definido */
    readonly strokeWidth?: Pt;
}
/** M15: seção nomeada do arquivo; começa no slide `from` do autor (base 1) e vai até a próxima. */
export interface DeckSection {
    readonly title: string;
    readonly from: number;
}
export interface DeckProps {
    /**
     * M15: seções do arquivo (painel de slides do PowerPoint), em ordem crescente de `from`. Os números
     * são do <Deck> do autor; a paginação é traduzida. Slides antes da 1ª seção vão numa seção padrão.
     */
    readonly sections?: readonly DeckSection[];
    /**
     * M13: conteúdo repetido em todo slide (rodapé, número), entra como último filho de cada slide
     * com `position: absolute` (o autor posiciona com bottom/left/right). Repete nas partes paginadas.
     */
    readonly footer?: Children;
    readonly children?: Children;
}
export interface SlideProps extends ContainerProps {
    /** Notas do apresentador. M15: rich text (`B`/`I`/`Span`, links externos; `\n` = parágrafo) além de string. */
    readonly notes?: string | TextChildren;
    /** M15: slide oculto (pulado na apresentação; continua no arquivo). Padrão false. */
    readonly hidden?: boolean;
    readonly children?: Children;
}
/**
 * M15: papel de um nó no layout do slide (`defineLayout`). Os nomes conhecidos viram o tipo de
 * placeholder do PowerPoint (title, subTitle, body, ftr, pic); outro nome vira um `body` com esse nome.
 */
export type PlaceholderName = 'title' | 'subtitle' | 'body' | 'footer' | 'pictures' | (string & {});
export interface BoxProps extends FlexItemProps, ContainerProps, StrokeProps {
    /** Raio dos cantos em pt; > 0 emite roundRect. */
    readonly radius?: Pt;
    /**
     * M9: Box cujos filhos em fluxo, quando não cabem, continuam num slide clonado
     * (ver CONTRACTS §M9 "Slide de continuação"). Só um Box paginate por slide. Padrão false.
     */
    readonly paginate?: boolean;
    /** Grupo de fit herdado: vale para todo Text de dentro que não declarar o próprio `fitGroup` (ver TextProps). */
    readonly fitGroup?: string;
    /**
     * M12: geometria do fundo/borda da caixa (padrão 'rect'; com borderRadius vira 'roundRect').
     * Os filhos continuam posicionados pelo padding, como numa caixa retangular: a forma não
     * recalcula o layout. 'line' não é aceito.
     */
    readonly shape?: ShapeKind;
    /** M13: hyperlink no fundo da caixa: URL externa ou `slide:N` (N = slide do autor, base 1). */
    readonly href?: string;
    /** M15: a área desta caixa vira um placeholder do layout (o conteúdo continua como objetos do slide). */
    readonly placeholder?: PlaceholderName;
    readonly children?: Children;
}
/**
 * M13: 'regular' (= 400) e 'bold' (= 700) são as formas canônicas; os outros pesos (1–1000, como
 * no CSS) ficam numéricos. O resolve normaliza: 400 → 'regular', 700 → 'bold'.
 */
export type FontWeight = 'regular' | 'bold' | number;
/** M13: posição vertical de um trecho (sobrescrito/subscrito). */
export type VerticalAlign = 'baseline' | 'super' | 'sub';
export type TextAlign = 'left' | 'center' | 'right';
export type FitPolicy = 'shrink' | 'truncate' | 'error';
export interface TextStyleProps {
    /** Família registrada; padrão 'Arial'. */
    readonly font?: string;
    /** pt; padrão 18 */
    readonly size?: Pt;
    readonly weight?: FontWeight;
    readonly italic?: boolean;
    /** padrão '#000000' */
    readonly color?: Color;
    /**
     * M11 (semântica CSS): 'normal' (padrão) = altura natural da face (ascent − descent + lineGap);
     * número = múltiplo do fontSize (1.2 em 20 pt = 24 pt por linha), com meia-entrelinha acima e abaixo.
     */
    readonly lineHeight?: number | 'normal';
    readonly align?: TextAlign;
}
export interface TextProps extends FlexItemProps, TextStyleProps {
    /** padrão 'shrink' */
    readonly fit?: FitPolicy;
    /** M8: nunca quebra linha (um parágrafo = uma linha); estouro de largura vira diagnóstico. */
    readonly nowrap?: boolean;
    /** padrão: 75% de size (arredondado para baixo) */
    readonly minSize?: Pt;
    readonly maxLines?: number;
    /**
     * Grupo de fit: os Text do mesmo slide com o mesmo nome (e `fit="shrink"`) encolhem pela mesma
     * escala, a do membro que menos coube (altura, `nowrap` ou palavra partida). Corpos de partida
     * iguais terminam no mesmo corpo; diferentes mantêm a proporção. O piso é o do membro que menos
     * pode encolher (o maior minSize/fontSize). Herdável: `fitGroup` num Box vale para os Text de dentro.
     */
    readonly fitGroup?: string;
    /** M13: hyperlink no texto inteiro (URL ou `slide:N`); um Span pode ter o próprio. */
    readonly href?: string;
    /**
     * M13: o texto que não cabe continua num clone do slide (divide as linhas já quebradas).
     * Só um nó paginate (Box ou Text) por slide; enquanto pagina, este Text não encolhe. Padrão false.
     */
    readonly paginate?: boolean;
    /** M15: o texto sai ligado ao placeholder deste nome do layout do slide (`defineLayout`). */
    readonly placeholder?: PlaceholderName;
    /** Conteúdo: strings/números e Span/B/I (M8); concatenados na resolução. */
    readonly children?: TextChildren;
}
/**
 * M8 — trecho inline dentro de Text. `B` e `I` são Span com weight/italic fixos.
 * M13-7: `fontSize`/`fontFamily` por Span (CssSpanProps) mudam a altura da linha em que o trecho
 * cai (line box do CSS: a maior ascendente e a maior descendente da linha); `lineHeight` continua do Text.
 */
export interface SpanProps {
    readonly weight?: FontWeight;
    readonly italic?: boolean;
    readonly color?: Color;
    /** M13: sobrescrito/subscrito: corpo × DEFAULTS.text.superscriptRatio, sem mudar a altura da linha. */
    readonly verticalAlign?: VerticalAlign;
    /** M13: hyperlink do trecho (URL ou `slide:N`). */
    readonly href?: string;
    readonly children?: TextChildren;
}
/** M13: formato do número de slide; `total` = slides do deck de saída (depois da paginação). */
export type SlideNumberFormat = 'n' | 'n / total';
/**
 * M13: número do slide como campo nativo (`slidenum`, atualiza se o usuário reordenar); o total
 * sai como texto fixo. É um Text de uma linha, medido no pior caso ("88 / 88").
 */
export interface SlideNumberProps extends FlexItemProps, TextStyleProps {
    /** Padrão 'n'. */
    readonly format?: SlideNumberFormat;
}
export type ImageFit = 'contain' | 'cover';
export interface ImageProps extends FlexItemProps {
    readonly src: string;
    /** padrão 'contain' */
    readonly fit?: ImageFit;
    /** M13: hyperlink da imagem (URL ou `slide:N`). */
    readonly href?: string;
    /** M15: a imagem sai ligada ao placeholder deste nome do layout do slide. */
    readonly placeholder?: PlaceholderName;
}
/**
 * M12: lista curada; cada kind tem `prstGeom` próprio (DEFAULTS.shape.pptx) e desenho fiel no
 * preview (src/preview/shapes.ts). Fora da lista: `Path`.
 */
export type ShapeKind = 'rect' | 'roundRect' | 'ellipse' | 'line' | 'triangle' | 'rightTriangle' | 'diamond' | 'parallelogram' | 'trapezoid' | 'pentagon' | 'hexagon' | 'octagon' | 'chevron' | 'arrowRight' | 'arrowLeft' | 'arrowUp' | 'arrowDown' | 'doubleArrow' | 'pill' | 'star' | 'cloud' | 'callout' | 'bracketPair' | 'cylinder';
/** M12: traço do contorno (PPTX `prstDash`: solid, dash, sysDot). */
export type LineStyle = 'solid' | 'dashed' | 'dotted';
/** M12: ponta do traço (PPTX `cap`: flat, rnd). */
export type LineCap = 'flat' | 'round';
/** M12: junção dos segmentos (PPTX `<a:miter/>`, `<a:round/>`, `<a:bevel/>`). */
export type LineJoin = 'miter' | 'round' | 'bevel';
/** M12: ponta de seta nativa (PPTX `headEnd`/`tailEnd`). */
export type ArrowType = 'none' | 'arrow' | 'triangle' | 'stealth' | 'diamond' | 'oval';
export interface ShapeProps extends FlexItemProps, StrokeProps {
    readonly kind: ShapeKind;
    /** Raio para roundRect, em pt. */
    readonly radius?: Pt;
    /** M12: espelha a forma no eixo vertical (horizontal) — `flipH`/`flipV` do PPTX. */
    readonly flipH?: boolean;
    readonly flipV?: boolean;
    /** M12: padrão 'solid'. */
    readonly lineStyle?: LineStyle;
    /** M12: padrão 'flat'. */
    readonly lineCap?: LineCap;
    /** M12: pontas de seta; só em `kind="line"` (senão ResolveError). Padrão 'none'. */
    readonly startArrow?: ArrowType;
    readonly endArrow?: ArrowType;
}
/**
 * Geometria livre por um subconjunto do `d` do SVG: M L H V C Q Z (absolutos e relativos) e A
 * (convertido em cúbicas). O `viewBox` é esticado até o frame (como a geometria de uma forma);
 * sem width/height o tamanho é o do viewBox em pt; com um só, o outro sai da proporção.
 */
export interface PathProps extends FlexItemProps, StrokeProps {
    readonly d: string;
    /** [largura, altura] com origem em 0,0; padrão: a caixa envolvente do `d`. */
    readonly viewBox?: readonly [number, number];
    /** Padrão 'flat' e 'miter' (os do PPTX). */
    readonly lineCap?: LineCap;
    readonly lineJoin?: LineJoin;
    readonly lineStyle?: LineStyle;
}
/** M12: comando de path normalizado: absoluto, sem H/V/S/T/A (só moveTo, lineTo, cúbica, quadrática, close). */
export type PathCommand = {
    readonly op: 'M';
    readonly x: number;
    readonly y: number;
} | {
    readonly op: 'L';
    readonly x: number;
    readonly y: number;
} | {
    readonly op: 'C';
    readonly x1: number;
    readonly y1: number;
    readonly x2: number;
    readonly y2: number;
    readonly x: number;
    readonly y: number;
} | {
    readonly op: 'Q';
    readonly x1: number;
    readonly y1: number;
    readonly x: number;
    readonly y: number;
} | {
    readonly op: 'Z';
};
/**
 * M12: ícone SVG. `render="auto"` (padrão) vira formas nativas quando o SVG é só
 * path/rect/circle/ellipse/line/polygon/polyline com cores sólidas; senão imagem (resvg) com
 * `icon-rasterized` (info). Altura padrão `size`, largura pela proporção do viewBox.
 */
export interface IconProps extends FlexItemProps {
    readonly src: string;
    /** Altura em pt quando o autor não dá width/height; padrão DEFAULTS.icon.size. */
    readonly size?: Pt;
    /** Cor que substitui currentColor (e preto, com recolor); padrão: a cor de texto herdada. */
    readonly color?: Color;
    /** Troca também fills/strokes pretos por `color`; padrão true. */
    readonly recolor?: boolean;
    /** 'shape' exige vetor (senão ResolveError); 'image' força a imagem. Padrão 'auto'. */
    readonly render?: 'auto' | 'shape' | 'image';
}
/** M12: um traço de ícone já pronto para desenhar (coordenadas e strokeWidth no viewBox do ícone). */
export interface VectorPath {
    readonly commands: readonly PathCommand[];
    readonly fill?: Color;
    readonly stroke?: Color;
    readonly strokeWidth?: number;
    readonly lineCap?: LineCap;
    readonly lineJoin?: LineJoin;
}
export type ConnectorSide = 'top' | 'right' | 'bottom' | 'left' | 'auto';
export type ConnectorRoute = 'straight' | 'elbow';
export interface ConnectorLabelStyle {
    readonly font?: string;
    /** pt; padrão DEFAULTS.connector.label.size */
    readonly fontSize?: Pt;
    readonly color?: Color;
    /** Fundo do rótulo (retângulo atrás do texto); ausente = sem fundo. */
    readonly background?: Color;
}
/**
 * M12: liga dois nós do mesmo slide pelo `id`. Pode ser filho do Slide ou de qualquer Box, mas
 * não entra no layout: é calculado depois do ajuste, em coordenadas do slide, e pintado por cima
 * dos nós. Sem desvio de obstáculos; o autor escolhe os lados.
 */
export interface ConnectorProps {
    readonly from: string;
    readonly to: string;
    /** Padrão 'auto': o par de lados (pontos médios) mais próximo. */
    readonly fromSide?: ConnectorSide;
    readonly toSide?: ConnectorSide;
    /** Padrão 'straight'. */
    readonly route?: ConnectorRoute;
    /** Padrão 'none' e DEFAULTS.connector.endArrow. */
    readonly startArrow?: ArrowType;
    readonly endArrow?: ArrowType;
    /** Padrão DEFAULTS.connector.stroke / strokeWidth. */
    readonly stroke?: Color;
    readonly strokeWidth?: Pt;
    readonly lineStyle?: LineStyle;
    /** Texto simples ou Span/B/I, como num Text; centrado no segmento mais longo. */
    readonly label?: TextChildren;
    readonly labelStyle?: ConnectorLabelStyle;
}
/**
 * M14: um `Chart`, vários kinds. Empilhados são kinds próprios (mudam dados válidos e o desenho);
 * `combo` mistura barras e linhas por série (`series[i].as`), com eixo secundário opcional.
 */
export type ChartKind = 'bar' | 'barH' | 'stackedBar' | 'stackedBarH' | 'stackedBar100' | 'line' | 'area' | 'stackedArea' | 'pie' | 'doughnut' | 'scatter' | 'combo';
export interface ChartSeries {
    readonly name: string;
    /** Um valor por categoria (mesmo tamanho de `categories`); números finitos. Obrigatório fora do scatter. */
    readonly values?: readonly number[];
    /** M14 (scatter): pares [x, y]; dispensa `categories`. */
    readonly points?: readonly (readonly [number, number])[];
    /** M14 (combo): como a série é desenhada; obrigatório no combo. */
    readonly as?: 'bar' | 'line';
    /** M14 (combo): eixo de valores; 'secondary' cria o eixo da direita. Padrão 'primary'. */
    readonly axis?: 'primary' | 'secondary';
    /** M14: cor da série (vence `colors[i]`). */
    readonly color?: Color;
}
export interface ChartData {
    /** Obrigatório fora do scatter. */
    readonly categories?: readonly string[];
    /** `pie`/`doughnut` aceitam exatamente uma série. */
    readonly series: readonly ChartSeries[];
}
/** M14: eixo do gráfico; tudo opcional (padrões do renderizador). */
export interface ChartAxis {
    readonly title?: string;
    readonly min?: number;
    readonly max?: number;
    /** Distância entre marcas (unidade principal). */
    readonly step?: number;
    /** Código de formato numérico do Excel ('0%', '#,##0', '0.0'). */
    readonly format?: string;
    /** Linhas de grade do eixo; padrão: só no eixo de valores. */
    readonly grid?: boolean;
    readonly hidden?: boolean;
}
export type ChartLegendPosition = 'bottom' | 'top' | 'left' | 'right';
export type ChartLabelPosition = 'outside' | 'inside' | 'center';
/** M14: rótulos de valor; `true` = padrão por kind. */
export interface ChartLabels {
    /** Código de formato do Excel; padrão: as casas decimais dos dados. */
    readonly format?: string;
    readonly position?: ChartLabelPosition;
    /**
     * Cor fixa de todos os rótulos. Sem ela, o rótulo desenhado dentro da barra/fatia (empilhado,
     * rosca, `position: 'inside' | 'center'`) troca para branco ou preto quando a cor do texto do
     * gráfico não se lê sobre a cor da série.
     */
    readonly color?: Color;
}
/**
 * Gráfico nativo (editável no PowerPoint). Folha sem tamanho intrínseco: precisa de
 * `width`/`height`/`grow` (ou `position="absolute"`), senão ResolveError.
 */
export interface ChartProps extends FlexItemProps {
    readonly kind: ChartKind;
    readonly data: ChartData;
    /** Por série (bar/line) ou por categoria (pie/doughnut); padrão DEFAULTS.chart.palette (cíclica). */
    readonly colors?: readonly Color[];
    /** Padrão: `series.length > 1` (pie/doughnut: true). M14: objeto escolhe a posição (padrão 'bottom'). */
    readonly legend?: boolean | {
        readonly position: ChartLegendPosition;
    };
    /** M14: rótulos de valor sobre as barras/pontos/fatias (true, ou formato, posição e cor); padrão false. */
    readonly labels?: boolean | ChartLabels;
    /** M14: eixo de categorias (no barH, o vertical), eixo de valores e o secundário (combo). */
    readonly xAxis?: ChartAxis;
    readonly yAxis?: ChartAxis;
    readonly y2Axis?: ChartAxis;
    /** M14 (doughnut): furo em % do diâmetro; padrão DEFAULTS.chart.holeSize. */
    readonly holeSize?: number;
    /** M14 (line/area/combo): linha suavizada. */
    readonly smooth?: boolean;
    /** M14 (line/scatter/combo): marcadores nos pontos; padrão: só no scatter. */
    readonly markers?: boolean;
    readonly title?: string;
    /** Tipografia de rótulos, eixos, legenda e título; padrão DEFAULTS.chart (família = DEFAULTS.text.font). */
    readonly font?: string;
    readonly size?: Pt;
    readonly color?: Color;
}
/** M14: alinhamento vertical na célula. */
export type CellVerticalAlign = 'top' | 'middle' | 'bottom';
/** M14: ícone no início da célula (forma sobre a tabela; o texto começa depois dele). */
export interface TableCellIcon {
    readonly src: string;
    /** Padrão: a cor do texto da célula. */
    readonly color?: Color;
    /** Lado em pt; padrão o corpo do texto da tabela. */
    readonly size?: Pt;
}
/**
 * M14: célula como objeto, quando precisa de merge, estilo ou rich text. `children` é rich text
 * (Span/B/I/Sup/Sub, como no Text) e vence `text`.
 */
export interface TableCellSpec {
    readonly text?: string | number;
    readonly children?: TextChildren;
    readonly colSpan?: number;
    readonly rowSpan?: number;
    readonly fill?: Color;
    readonly color?: Color;
    readonly fontWeight?: FontWeight;
    readonly fontStyle?: 'normal' | 'italic';
    readonly align?: TextAlign;
    readonly valign?: CellVerticalAlign;
    readonly icon?: TableCellIcon;
}
export type TableCellValue = string | number | TableCellSpec;
/** M14: o que `cell()` recebe para decidir o estilo de uma célula. */
export interface TableCellContext {
    /** Índice da linha de dados (0 = 1ª linha de `rows`); nas linhas de cabeçalho, o índice entre elas. */
    readonly row: number;
    readonly col: number;
    /** O valor como o autor escreveu (string, número ou objeto). */
    readonly value: TableCellValue;
    readonly header: boolean;
}
export interface TableColumn {
    readonly header: string;
    /** pt ou % da largura da tabela; ausente = divide o que sobra igualmente. */
    readonly width?: Length;
    /** Alinhamento das células da coluna (e do cabeçalho); padrão 'left'. */
    readonly align?: TextAlign;
    /** M14: alinhamento vertical das células da coluna; padrão o da tabela. */
    readonly valign?: CellVerticalAlign;
}
export interface TableHeaderStyle {
    readonly fill?: Color;
    readonly color?: Color;
    readonly weight?: FontWeight;
}
/** `truncate` não se aplica a tabela: ou encolhe ou é erro. */
export type TableFitPolicy = 'shrink' | 'error';
/**
 * Tabela nativa medida pela engine: largura vem do flex (como Box), altura = soma das
 * linhas, cada linha = célula mais alta + 2 × cellPadding. Participa do shrink como
 * UMA unidade (todas as células com o mesmo size).
 */
export interface TableProps extends FlexItemProps, Pick<TextStyleProps, 'font' | 'size' | 'color' | 'lineHeight'> {
    readonly columns: readonly TableColumn[];
    /** rows[i].length === columns.length; números viram texto na resolução. */
    readonly rows: readonly (readonly TableCellValue[])[];
    /** Estilo da linha de cabeçalho (padrão DEFAULTS.table.header); `false` = sem cabeçalho. */
    readonly header?: TableHeaderStyle | false;
    /** Fundo das linhas de dados pares (2ª, 4ª…); ausente = sem zebra. */
    readonly stripe?: Color;
    /** Cor das bordas (todas); ausente = sem borda. */
    readonly border?: Color;
    /** Margem interna da célula em pt; padrão DEFAULTS.table.cellPadding. */
    readonly cellPadding?: Pt;
    /** padrão 'shrink' */
    readonly fit?: TableFitPolicy;
    /** padrão: como no Text (75% de size, piso minLegibleSize) */
    readonly minSize?: Pt;
    /**
     * M14: linhas de cabeçalho acima da linha de `columns[].header` (cabeçalho de dois níveis): cada
     * linha cobre todas as colunas (com `colSpan`). Estilo do cabeçalho; repetem nas partes paginadas.
     */
    readonly headerRows?: readonly (readonly TableCellValue[])[];
    /** M14: estilo por célula calculado (cor por valor); o que a célula declara vence. */
    readonly cell?: (ctx: TableCellContext) => Partial<TableCellSpec> | undefined;
    /** M14: linhas que não cabem continuam num clone do slide (corte entre linhas, nunca num rowSpan). */
    readonly paginate?: boolean;
    /** M14: repete o cabeçalho em cada parte paginada; padrão true. */
    readonly repeatHeader?: boolean;
    /** M14: alinhamento vertical padrão das células; padrão 'top'. */
    readonly valign?: CellVerticalAlign;
}
/** Comprimento normalizado: número = pt; string = percentual do pai. */
export type Len = number | `${number}%`;
export type MarginLen = Len | 'auto';
export interface Sides<T> {
    readonly top: T;
    readonly right: T;
    readonly bottom: T;
    readonly left: T;
}
export type Side = keyof Sides<unknown>;
export type BorderStyle = 'solid' | 'dashed' | 'dotted' | 'none';
export interface BorderSide {
    readonly width: Pt;
    readonly style: BorderStyle;
    /** Hex '#RRGGBB'; undefined = currentColor (resolvido para a cor de texto herdada). */
    readonly color: Color;
    /** 0..1, de rgba(); 1 = opaca. */
    readonly alpha: number;
}
export interface Shadow {
    readonly x: Pt;
    readonly y: Pt;
    readonly blur: Pt;
    readonly spread: Pt;
    readonly color: Color;
    readonly alpha: number;
    readonly inset: boolean;
}
export type Display = 'flex' | 'block' | 'grid' | 'none';
export type CssPosition = 'static' | 'relative' | 'absolute';
export type CssFlexDirection = 'row' | 'column' | 'row-reverse' | 'column-reverse';
export type CssFlexWrap = 'nowrap' | 'wrap' | 'wrap-reverse';
export type CssJustify = 'flex-start' | 'center' | 'flex-end' | 'space-between' | 'space-around' | 'space-evenly';
export type CssAlign = 'flex-start' | 'center' | 'flex-end' | 'stretch' | 'baseline';
export type CssAlignContent = CssJustify | 'stretch';
/** Alinhamento de item de grid na célula (eixo de linha ou de coluna). */
export type CssPlace = 'start' | 'center' | 'end' | 'stretch';
/**
 * De onde vem o flex-basis: 'author' (flexBasis/flex 'a b c') ou 'flex' (`flex: n` → 0, salvo
 * Text/Table em column, que emula o min-height:auto do browser).
 */
export type BasisSource = 'none' | 'author' | 'flex';
export interface TextDecoration {
    readonly underline: boolean;
    readonly lineThrough: boolean;
}
export type TextTransform = 'none' | 'uppercase' | 'lowercase' | 'capitalize';
/** Propriedade que chegou ao nó e não tem de/para para o PPTX (vira css-not-emitted). */
export interface NotEmitted {
    readonly property: string;
    readonly value: string;
    /** Frase curta para a mensagem: o que acontece com ela. */
    readonly reason: string;
}
/** Linha de grid: número (1-based, negativo conta do fim), span n ou auto. */
export type GridLine = {
    readonly kind: 'line';
    readonly index: number;
} | {
    readonly kind: 'span';
    readonly count: number;
} | {
    readonly kind: 'auto';
};
export interface GridPlacement {
    readonly start: GridLine;
    readonly end: GridLine;
}
/** Estilo CSS normalizado de um nó (sem shorthands, sem responsivo, já com herança). */
export interface CssStyle {
    readonly display: Display;
    readonly position: CssPosition;
    readonly inset: Partial<Sides<Len>>;
    readonly width?: Len;
    readonly height?: Len;
    readonly minWidth?: Len;
    readonly maxWidth?: Len;
    readonly minHeight?: Len;
    readonly maxHeight?: Len;
    readonly aspectRatio?: number;
    readonly flexDirection: CssFlexDirection;
    readonly flexWrap: CssFlexWrap;
    readonly justifyContent: CssJustify;
    readonly alignItems: CssAlign;
    readonly alignContent: CssAlignContent;
    readonly alignSelf?: CssAlign;
    readonly justifySelf?: CssPlace;
    readonly justifyItems?: CssPlace;
    readonly flexGrow: number;
    readonly flexShrink: number;
    readonly flexBasis: Len | 'auto';
    readonly basisSource: BasisSource;
    readonly order: number;
    readonly rowGap: Pt;
    readonly columnGap: Pt;
    readonly margin: Sides<MarginLen>;
    readonly padding: Sides<Len>;
    readonly border: Sides<BorderSide>;
    readonly borderRadius?: Pt;
    readonly backgroundColor?: Color;
    readonly backgroundAlpha: number;
    readonly backgroundImage?: string;
    /** Açúcar antigo `stroke`: contorno que não ocupa espaço (como `outline`), centrado na borda. */
    readonly outline?: {
        readonly color: Color;
        readonly width: Pt;
    };
    readonly boxShadow?: Shadow;
    readonly opacity: number;
    /** Graus, sentido horário (como CSS e PPTX). */
    readonly rotate: number;
    readonly translate: {
        readonly x: Pt;
        readonly y: Pt;
    };
    readonly zIndex?: number;
    readonly overflow: 'visible' | 'hidden';
    readonly gridTemplateColumns?: string;
    readonly gridTemplateRows?: string;
    readonly gridAutoRows?: string;
    readonly gridAutoColumns?: string;
    readonly gridColumn: GridPlacement;
    readonly gridRow: GridPlacement;
    readonly textDecoration: TextDecoration;
    readonly letterSpacing: Pt;
    readonly textTransform: TextTransform;
    readonly notEmitted: readonly NotEmitted[];
}
/** M11: caixa resolvida pelo layout (pt; % e auto já calculados). */
export interface BoxMetrics {
    readonly margin: Sides<Pt>;
    readonly border: Sides<Pt>;
    readonly padding: Sides<Pt>;
}
/** M19: breakpoints de contêiner: `small` = estreito e baixo ao mesmo tempo. */
export type ContainerBreakpoint = 'narrow' | 'short' | 'small';
export type ContainerResponsiveValue<T> = {
    readonly base: T;
} & {
    readonly [name in ContainerBreakpoint]?: T;
};
/** M19: valor que depende do espaço do contêiner (como Responsive<T> depende do slide). */
export type ContainerResponsive<T> = T | ContainerResponsiveValue<T>;
/** M19: o espaço medido de um `<Container>` (content box, em pt), entregue à função filha. */
export interface ContainerBox {
    readonly width: Pt;
    readonly height: Pt;
    /** width / height (0 quando a altura é 0). */
    readonly aspect: number;
    /** width < DEFAULTS.containerQuery.narrow */
    readonly narrow: boolean;
    /** height < DEFAULTS.containerQuery.short */
    readonly short: boolean;
    /**
     * Escolhe pelo espaço: small (estreito e baixo) > narrow / short > base. Duas assinaturas, a responsiva primeiro:
     * com uma só, `pick('texto')` inferia T pelo `String.prototype.small()` do JS e não compilava.
     */
    readonly pick: {
        <T>(value: ContainerResponsiveValue<T>): T;
        <T>(value: T): T;
    };
    /** O contexto do slide, para combinar com tall/wide. */
    readonly slide: DeckContext;
}
/**
 * M19: um Box cujo conteúdo é uma função do espaço que ele recebeu. Contenção de tamanho: o tamanho vem do pai
 * e das props (flex, width/height, grid), nunca do conteúdo; por isso a medição é determinística (ver CONTRACTS).
 */
export interface ContainerQueryProps extends Omit<BoxProps, 'children' | 'paginate' | 'placeholder'> {
    readonly children: (box: ContainerBox) => Child;
}
export type PrimitiveKind = 'deck' | 'slide' | 'box' | 'text' | 'image' | 'shape' | 'span' | 'chart' | 'table' | 'path' | 'icon' | 'connector' | 'slideNumber' | 'container';
/** Props concretas por primitiva (o que a árvore resolvida carrega). */
export interface PrimitivePropsMap {
    readonly deck: DeckProps;
    readonly slide: SlideProps;
    readonly box: BoxProps;
    readonly text: TextProps;
    readonly image: ImageProps;
    readonly shape: ShapeProps;
    readonly span: SpanProps;
    readonly chart: ChartProps;
    readonly table: TableProps;
    readonly path: PathProps;
    readonly icon: IconProps;
    readonly connector: ConnectorProps;
    readonly slideNumber: SlideNumberProps;
    readonly container: ContainerQueryProps;
}
/**
 * M11: o que o AUTOR escreve no JSX — CSS em camelCase (ver core/css-props.ts), mais as props da
 * engine (fit, minSize, src, data…). Toda prop aceita Responsive<X>.
 */
export interface AuthorPropsMap {
    readonly deck: DeckProps;
    readonly slide: CssSlideProps;
    readonly box: CssBoxPropsAuthor;
    readonly text: CssTextPropsAuthor;
    readonly image: CssImagePropsAuthor;
    readonly shape: CssShapePropsAuthor;
    readonly span: CssSpanProps;
    readonly chart: CssChartPropsAuthor;
    readonly table: CssTablePropsAuthor;
    readonly path: CssPathPropsAuthor;
    readonly icon: CssIconPropsAuthor;
    readonly connector: ConnectorPropsAuthor;
    readonly slideNumber: CssSlideNumberPropsAuthor;
    readonly container: CssContainerQueryPropsAuthor;
}
/** Marca que identifica uma primitiva (vs componente do usuário). */
export declare const PRIMITIVE_KIND: unique symbol;
/**
 * Primitiva: função "marcada" com seu kind. Nunca é chamada pela engine;
 * existe para dar tipagem ao JSX (<Box ...>) e ser reconhecida no resolve.
 */
export interface Primitive<K extends PrimitiveKind> {
    (props: AuthorPropsMap[K]): SlideElement;
    readonly [PRIMITIVE_KIND]: K;
}
/** M17: marca de identidade de um componente (`defineComponent`). */
export declare const COMPONENT_META: unique symbol;
/**
 * M17: identidade de um componente — o nome que vai no path (`displayName ?? name`), o DS de onde vem e
 * metadados livres que regras de lint e o catálogo leem (`catalog`, por exemplo). O núcleo não interpreta
 * nada além de `name`/`displayName`.
 */
/**
 * A camada de um componente no DS (o contrato do catálogo, docs/ds-catalog.md), da peça menor ao slide:
 * `widget` (atômico: um indicador, um selo), `component` (bloco de conteúdo: lista, citação),
 * `chart` (apresenta dados: gráfico, tabela, Gantt), `structure` (organiza outros numa região: colunas,
 * grade) e `template` (o slide inteiro: devolve o `<Slide>`; recomendado, para o agente começar cada slide).
 */
export type ComponentKind = 'widget' | 'component' | 'chart' | 'structure' | 'template';
/** Para quem o componente (ou uma variação dele) foi pensado. Outros valores valem; estes são os do catálogo. */
export type Audience = 'executive' | 'strategic' | 'tactical' | 'operational' | (string & {});
export interface ComponentMeta {
    readonly name: string;
    readonly displayName?: string;
    readonly ds?: string;
    readonly tags?: readonly string[];
    /** A camada no DS; padrão 'component'. 'template' sai como slide inteiro no catálogo. */
    readonly kind?: ComponentKind;
    /** Uma frase: o que o componente é (a 1ª linha dele no catálogo e no prompt). */
    readonly description?: string;
    /** Agrupa no catálogo dentro da camada ("Indicadores", "Roadmap"). */
    readonly category?: string;
    /** Públicos para os quais serve; cada caso do catálogo pode dizer o seu (variação tática × executiva). */
    readonly audience?: readonly Audience[];
    /** Situações em que o agente deve escolher este componente. */
    readonly whenToUse?: readonly string[];
    /** O que não fazer com ele (e o que usar no lugar). */
    readonly avoid?: readonly string[];
    /** Markdown com o guia do componente (vira uma seção do guia do DS). */
    readonly guide?: string;
    readonly [key: string]: unknown;
}
/** M18: um caso do catálogo de um componente (props de exemplo); `fill` = o componente preenche o corpo. */
export interface ComponentCatalogCase<P> {
    readonly name: string;
    readonly props: P;
    readonly fill?: boolean;
    /** Uma linha sobre o que o caso mostra. */
    readonly description?: string;
    /** O público desta variação (ex.: `['executive']` num caso resumido). */
    readonly audience?: readonly Audience[];
}
/** M18: o meta de `defineComponent` com `catalog` tipado pelas props do componente. */
export type ComponentMetaFor<P> = ComponentMeta & {
    readonly catalog?: readonly ComponentCatalogCase<P>[];
};
/** Componente do usuário: função pura (props) => elemento. Sem hooks, sem estado. */
export type Component<P = any> = ((props: P) => Child) & {
    readonly [COMPONENT_META]?: ComponentMeta;
};
/** M15: marca de um componente de layout (`defineLayout`). */
export declare const LAYOUT_DEFINITION: unique symbol;
export interface LayoutOptions {
    /** Placeholders que o layout oferece; os nós marcam com `placeholder="…"`. */
    readonly placeholders: readonly PlaceholderName[];
    /** Fundo do layout no master (cor ou imagem); os slides sem fundo próprio herdam. */
    readonly background?: Color | {
        readonly image: string;
    };
    /** Conteúdo fixo do layout (logotipo, faixa): sai no master, não em cada slide. Posicione com `position="absolute"`. */
    readonly fixed?: Children;
}
export interface LayoutDefinition extends LayoutOptions {
    readonly name: string;
}
/** Componente marcado por `defineLayout`: o <Slide> que ele devolve pertence ao layout. */
export type LayoutComponent<P = any> = Component<P> & {
    readonly [LAYOUT_DEFINITION]: LayoutDefinition;
};
export declare const FRAGMENT: unique symbol;
export type FragmentType = typeof FRAGMENT;
export type ElementType = Primitive<PrimitiveKind> | Component | FragmentType;
/** Saída de jsx(): imutável, sem nenhuma resolução feita. */
export interface SlideElement {
    readonly type: ElementType;
    readonly props: Readonly<Record<string, unknown>> & {
        readonly children?: Children;
    };
    readonly key?: string | number;
}
export type Child = SlideElement | string | number | boolean | null | undefined | readonly Child[];
export type Children = Child;
/** Elementos aqui têm que ser Span/B/I (o resolve valida; o TS não distingue elementos). */
export type TextChildren = string | number | null | undefined | boolean | SlideElement | readonly TextChildren[];
/** O arquivo do deck exporta default defineDeck(fn). */
export type DeckDefinition = (ctx: DeckContext) => SlideElement;
/**
 * Regras do path (ver CONTRACTS.md §resolve):
 *  - segmento = nome da primitiva capitalizada ('Box', 'Text'...) ou, quando
 *    um componente devolve a primitiva, o nome do componente ('CoverLayout').
 *  - sufixo [i] quando o pai tem mais de um filho com o mesmo segmento;
 *    i = índice entre esses irmãos homônimos, começando em 0.
 *  - o path começa no slide (o Deck não entra): 'CoverLayout > Box > Text[0]'.
 */
/**
 * M16: geometria de item e de contêiner (tamanho, posição, flex, padding, raio, fundo do slide) vive só
 * em `css`; a "visão da engine" não repete essas chaves.
 */
export type ItemGeometryKey = Exclude<keyof FlexItemProps, 'id'>;
export type ContainerGeometryKey = keyof ContainerProps;
/** Props de um nó resolvido sem `children` e sem a geometria (que está em `css`). */
export type NodeProps<P, Geometry extends PropertyKey = ItemGeometryKey> = Omit<P, 'children' | Geometry>;
interface ResolvedBase<K extends PrimitiveKind, P> {
    readonly kind: K;
    readonly path: string;
    /**
     * M11: estilo CSS normalizado do nó (sem shorthands, sem responsivo, já com herança). É a fonte de
     * toda a geometria (M16: layout, fit, emit e preview leem daqui).
     */
    readonly css: CssStyle;
    /**
     * M17: o componente mais externo que gerou este nó (o mesmo que dá o segmento do path). Função comum
     * tem só `{ name }`; `defineComponent` traz o `meta` inteiro. Ausente = a primitiva foi escrita direto.
     */
    readonly component?: ComponentMeta;
    /**
     * 1.0.5: `<Nome prop=valor … />` do componente do autor que gerou este nó (só na raiz da instância, junto
     * de `component`; os filhos não herdam). Os diagnósticos de dentro da instância levam isso em
     * `data.component`: é onde o autor mexe, não no path interno do DS. Ausente = primitiva escrita direto.
     */
    readonly source?: string;
    /**
     * Props que não são CSS geométrico: o estilo de texto resolvido (com herança; o fit troca o `size`),
     * política de ajuste (`fit`, `minSize`, `maxLines`), conteúdo (`src`, `data`, `columns`, `kind`…) e
     * marcações (`paginate`, `href`, `id`, `placeholder`). Padrões: core/defaults.ts.
     */
    readonly props: P;
}
export interface ResolvedBox extends ResolvedBase<'box', NodeProps<BoxProps, ItemGeometryKey | ContainerGeometryKey | 'radius'>> {
    readonly children: readonly ResolvedNode[];
    /**
     * M19: o Box veio de um `<Container>`. 'pending' = ainda sem medida (sem filhos; a 1ª passada mede o espaço);
     * 'measured' = a função já recebeu o tamanho e o Box tem largura/altura fixadas nele (contenção de tamanho).
     */
    readonly container?: 'pending' | 'measured';
}
/** M8: estilo efetivo de um trecho (herda do Text, sobrescrito pelos Span aninhados). */
export interface TextRun {
    readonly text: string;
    readonly weight: FontWeight;
    readonly italic: boolean;
    readonly color: Color;
    /** M11: decoração por run (Span textDecoration). */
    readonly underline?: boolean;
    readonly lineThrough?: boolean;
    /** M13: sobrescrito/subscrito; ausente = 'baseline'. */
    readonly verticalAlign?: VerticalAlign;
    /** M13: hyperlink do trecho, já traduzido para o deck de saída no fim do fit (`slide:N` de saída). */
    readonly href?: string;
    /** M13-7: corpo (pt) e família do trecho; ausentes = os do Text. */
    readonly size?: Pt;
    readonly family?: string;
}
export interface ResolvedText extends ResolvedBase<'text', NodeProps<TextProps>> {
    /** Conteúdo já concatenado; '\n' separa parágrafos. */
    readonly content: string;
    /** M8: runs em ordem; a concatenação de `text` é igual a `content`. Sem Span: um run só. */
    readonly runs: readonly TextRun[];
    /**
     * M13: este Text é um SlideNumber. `content` é o pior caso até o fim do fit; depois da paginação
     * vira o número real (`slideIndex`/`slideTotal`), e o emissor grava o número como campo `slidenum`.
     */
    readonly slideNumber?: {
        readonly format: SlideNumberFormat;
        readonly index?: number;
        readonly total?: number;
        /**
         * Larguras naturais (pt) do número e do rótulo inteiro, medidas no fit. O Keynote apaga todo
         * texto que divide o parágrafo com o campo `slidenum`, então "n / total" sai em duas caixas
         * (o campo e o " / total" logo depois dele) e o emissor precisa saber onde o número termina.
         */
        readonly numberWidth?: Pt;
        readonly labelWidth?: Pt;
    };
}
export type ResolvedImage = ResolvedBase<'image', NodeProps<ImageProps>>;
export type ResolvedShape = ResolvedBase<'shape', NodeProps<ShapeProps>>;
/** M10: dados já validados (séries do tamanho das categorias; pie com 1 série). */
export type ResolvedChart = ResolvedBase<'chart', NodeProps<ChartProps>>;
/** M10: `rows` já com todas as células como string e com `columns.length` células. */
/** M14: ícone de célula já lido (como o do Icon). */
export interface ResolvedCellIcon {
    readonly viewBox: Rect;
    readonly content: IconContent;
    readonly size: Pt;
    readonly color: Color;
}
/** M14: célula normalizada (cabeçalho incluído). Coberta por merge = não desenha nem mede. */
export interface ResolvedCell {
    readonly text: string;
    readonly runs: readonly TextRun[];
    readonly colSpan: number;
    readonly rowSpan: number;
    readonly covered: boolean;
    readonly header: boolean;
    readonly fill?: Color;
    readonly align: TextAlign;
    readonly valign: CellVerticalAlign;
    readonly icon?: ResolvedCellIcon;
}
export interface ResolvedTable extends ResolvedBase<'table', NodeProps<TableProps>> {
    /** Texto das linhas de dados (compatível com o M10). */
    readonly rows: readonly (readonly string[])[];
    /**
     * M14: grade completa na ordem de desenho: `headerRows`, a linha de `columns` (se há cabeçalho) e
     * os dados; `grid[i].length === columns.length`. `headerCount` = quantas linhas são cabeçalho.
     */
    readonly grid: readonly (readonly ResolvedCell[])[];
    readonly headerCount: number;
}
/**
 * M12: `commands` já absolutos e sem arcos; `viewBox` efetivo (o do autor com origem 0,0 ou a
 * caixa envolvente do `d`). `problem` = o `d` saiu do subconjunto: nada é desenhado e o fit
 * emite `path-invalid` (error).
 */
export interface ResolvedPath extends ResolvedBase<'path', NodeProps<PathProps>> {
    readonly commands: readonly PathCommand[];
    readonly viewBox: Rect;
    readonly problem?: string;
}
/** M12: o SVG já lido e classificado. Raster guarda o SVG recolorido e o motivo (vai no `icon-rasterized`). */
export type IconContent = {
    readonly paths: readonly VectorPath[];
} | {
    readonly raster: true;
    readonly svg: string;
    readonly reason: string;
};
/** M12: `props.color` já é a cor efetiva (a do autor ou a herdada). */
export interface ResolvedIcon extends ResolvedBase<'icon', NodeProps<IconProps>> {
    readonly viewBox: Rect;
    readonly content: IconContent;
}
export type ResolvedNode = ResolvedBox | ResolvedText | ResolvedImage | ResolvedShape | ResolvedChart | ResolvedTable | ResolvedPath | ResolvedIcon;
/**
 * M12: conector resolvido. Path como o de um nó ('Slide > Box > Connector[1]'). `label` é um Text
 * resolvido (path `<conector> > Label`) com o estilo de `labelStyle`; `missing` lista as pontas
 * cujo id não existe no slide do autor (o fit emite `connector-target-missing`).
 */
export interface ResolvedConnector {
    readonly kind: 'connector';
    readonly path: string;
    readonly props: Omit<ConnectorProps, 'label'>;
    readonly label?: ResolvedText;
    readonly missing?: readonly ('from' | 'to')[];
}
/** M15: notas resolvidas; '\n' em `content` separa parágrafos, como no Text. */
export interface ResolvedNotes {
    readonly content: string;
    readonly runs: readonly TextRun[];
}
/** M15: layout do slide; `fixedPath` = path do invólucro do conteúdo fixo (sai no master), quando há. */
export interface ResolvedLayout {
    readonly name: string;
    readonly placeholders: readonly PlaceholderName[];
    readonly background?: Color | {
        readonly image: string;
    };
    readonly fixedPath?: string;
}
export interface ResolvedSlide extends ResolvedBase<'slide', NodeProps<SlideProps, ContainerGeometryKey | 'background'>> {
    /** M9: índice do slide no <Deck> do autor, começando em 1 (clones de paginação repetem o do original). */
    readonly sourceSlide: number;
    /** M15: notas com estilo por trecho; ausente = sem notas. */
    readonly notes?: ResolvedNotes;
    /** M15: o layout (`defineLayout`) que gerou este slide; ausente = slide solto, sem master próprio. */
    readonly layout?: ResolvedLayout;
    readonly children: readonly ResolvedNode[];
    /** M12: conectores do slide inteiro (de qualquer Box), fora da árvore de layout. Ausente = nenhum. */
    readonly connectors?: readonly ResolvedConnector[];
}
export interface ResolvedDeck {
    readonly slides: readonly ResolvedSlide[];
    /** M15: seções validadas (números do autor). */
    readonly sections?: readonly DeckSection[];
}
export interface FontFace {
    readonly family: string;
    /** M13: 'regular', 'bold' ou o peso numérico da face (100–900; o do OS/2 do arquivo). */
    readonly weight: FontWeight;
    readonly italic?: boolean;
    /** Caminho para .ttf/.otf/.ttc. Ausente (M8) → procurar nas pastas de fontes do sistema. */
    readonly file?: string;
    /**
     * M8: nome PostScript da face dentro de uma coleção (.ttc); o resolver do sistema preenche e o
     * registro abre essa face (`fontkit.openSync(file, postscriptName)`). Ausente → 1ª face do arquivo.
     */
    readonly postscriptName?: string;
}
export interface TextStyle {
    readonly family: string;
    readonly size: Pt;
    readonly weight: FontWeight;
    readonly italic: boolean;
    /** M11: 'normal' = altura natural da face; número = múltiplo do size (CSS). */
    readonly lineHeight: number | 'normal';
}
export interface TextMeasureInput {
    readonly content: string;
    /** Estilo base: family, size, lineHeight; weight/italic são o padrão dos runs. */
    readonly style: TextStyle;
    /** M8: runs com estilo; ausente = um run com o estilo base. A concatenação tem que ser igual a `content`. */
    readonly runs?: readonly TextRun[];
    /** Largura disponível em pt; Infinity = sem limite. */
    readonly maxWidth: Pt;
    readonly maxLines?: number;
    /** M8: um parágrafo = uma linha; `width` pode passar de maxWidth. */
    readonly nowrap?: boolean;
    /** M11: pt somados a cada caractere (CSS letter-spacing; `spc` no PPTX). */
    readonly letterSpacing?: Pt;
}
/** M8: pedaço de uma linha já quebrada que pertence a um run. */
export interface LineRun {
    readonly text: string;
    /** Índice em TextMeasureInput.runs / ResolvedText.runs. */
    readonly run: number;
}
/**
 * M13: face que o medidor escolheu para um run (correspondência de peso do CSS Fonts §5.2). O PPTX só
 * tem a flag negrito: faces 400/700 saem com a família do autor e `b`; as outras, com o nome de
 * família da própria face (`Inter SemiBold`, name ID 1) e `b="0"`.
 */
export interface RunFace {
    /** Nome que vai em `<a:latin typeface>`. */
    readonly typeface: string;
    readonly bold: boolean;
    /** Peso da face usada (usWeightClass, ou o registrado). */
    readonly weight: number;
    /** Peso pedido, quando a face usada difere dele em mais de DEFAULTS.text.weightTolerance (font-weight-fallback). */
    readonly requested?: number;
    /** Família para o preview (resvg): a tipográfica da face (name ID 16, senão ID 1). */
    readonly previewFamily: string;
}
/** Resultado da medição: as linhas JÁ quebradas são usadas por todos os emissores. */
export interface TextLayout {
    /** Todas as linhas, achatadas (preview, diagnósticos). */
    readonly lines: readonly string[];
    /**
     * As mesmas linhas agrupadas por parágrafo do autor ('\n'). O emissor usa para
     * separar parágrafo (<a:p>) de quebra suave (<a:br/>). paragraphs.flat() === lines.
     */
    readonly paragraphs: readonly (readonly string[])[];
    /** M8: cada linha de `lines` dividida por run; lineRuns[i].map(r => r.text).join('') === lines[i]. */
    readonly lineRuns: readonly (readonly LineRun[])[];
    readonly fontSize: Pt;
    /** Altura de uma linha em pt. */
    readonly lineHeight: Pt;
    /** Distância do topo da linha até a baseline, em pt. */
    readonly ascent: Pt;
    /** Largura da linha mais longa. */
    readonly width: Pt;
    /** lines.length * lineHeight */
    readonly height: Pt;
    /**
     * Largura (pt) do maior trecho indivisível (segmento sem ponto de quebra) no size
     * atual, medido no texto inteiro. Permite ao fit ir direto ao size em que a palavra cabe.
     */
    readonly widestSegment: Pt;
    /** true se alguma palavra precisou ser quebrada por caractere (não coube na linha). */
    readonly wordBroken: boolean;
    /** true se maxLines cortou o texto (reticências aplicadas). */
    readonly truncated: boolean;
    /** true se a família/peso não estava registrada (medição foi estimada). */
    readonly fontMissing: boolean;
    /** M13: face de cada run (mesmo índice de `runs`); ausente na estimativa (sem fonte real). */
    readonly faces?: readonly RunFace[];
    /**
     * M13-7: altura e ascendente de cada linha quando algum run muda corpo ou família (a linha tem a
     * maior ascendente e a maior descendente dos runs dela). Ausentes = todas as linhas iguais a
     * `lineHeight`/`ascent`. Com elas, `lineHeight`/`ascent` são os da 1ª linha e `height` é a soma.
     */
    readonly lineHeights?: readonly Pt[];
    readonly ascents?: readonly Pt[];
}
/** Strategy: estimativa (M2) ou fontkit (M3). Função pura. */
export type TextMeasurer = (input: TextMeasureInput) => TextLayout;
/** Dependências injetadas no layout (mantém a etapa pura e testável). */
export interface LayoutEnv {
    readonly measureText: TextMeasurer;
    /** Dimensões intrínsecas da imagem em px. */
    readonly imageSize: (src: string) => Dimensions;
    /** M11: folga da quebra do medidor (o min-content do grid precisa dela); padrão DEFAULTS.textSafety. */
    readonly textSafety?: number;
}
interface Positioned {
    /** Border-box ABSOLUTO no slide, em pt, antes de girar (o layout ignora a rotação, como o browser). */
    readonly frame: Rect;
    /** M11: margem, borda e padding resolvidos pelo layout (pt; % e auto já calculados). */
    readonly box: BoxMetrics;
    /** M11: item de grid: a célula que o grid lhe deu (absoluta). É a área disponível do item no fit. */
    readonly cell?: Rect;
}
export interface PositionedBox extends Omit<ResolvedBox, 'children'>, Positioned {
    readonly children: readonly PositionedNode[];
}
export interface PositionedText extends ResolvedText, Positioned {
    readonly text: TextLayout;
}
export interface PositionedImage extends ResolvedImage, Positioned {
    readonly intrinsic: Dimensions;
}
export interface PositionedShape extends ResolvedShape, Positioned {
}
export interface PositionedChart extends ResolvedChart, Positioned {
}
/** M10: layout das células calculado pelo layout com a largura final do frame (ver CONTRACTS §M10 layout). */
export interface PositionedTable extends ResolvedTable, Positioned {
    /** Size efetivo das células (props.size após o fit, ou o padrão). */
    readonly fontSize: Pt;
    /** Uma largura por coluna (pt); soma = frame.width. */
    readonly columnWidths: readonly Pt[];
    /** Uma altura por linha (pt), cabeçalho incluído quando há; soma = altura medida da tabela. */
    readonly rowHeights: readonly Pt[];
    /** Linhas já quebradas por célula: cells[linha][coluna] (cabeçalho na linha 0 quando há). */
    readonly cells: readonly (readonly TextLayout[])[];
    /** Σ rowHeights: a altura que a tabela pediu (pode passar de frame.height = estouro). */
    readonly height: Pt;
    /**
     * M14: largura natural de um espaço na face da tabela, no size final. O Keynote ignora margem por
     * célula, então o espaço do ícone de uma célula sai como um espaço inicial com `charSpacing`.
     */
    readonly spaceWidth?: Pt;
}
export interface PositionedPath extends ResolvedPath, Positioned {
}
export interface PositionedIcon extends ResolvedIcon, Positioned {
}
export type PositionedNode = PositionedBox | PositionedText | PositionedImage | PositionedShape | PositionedChart | PositionedTable | PositionedPath | PositionedIcon;
/** M12: rótulo do conector já medido; `frame` inclui o respiro do fundo (o texto fica no frame do `text`). */
export interface ConnectorLabel {
    readonly frame: Rect;
    readonly text: PositionedText;
    readonly background?: Color;
}
/**
 * M12: conector roteado depois do ajuste. `points` = polilinha em coordenadas do slide (2 pontos
 * em `straight`; 3 ou 4 em `elbow`; pontos colineares já fundidos). Começa em `from`, termina em `to`.
 */
export interface PositionedConnector {
    readonly kind: 'connector';
    readonly path: string;
    readonly props: Omit<ConnectorProps, 'label'>;
    readonly points: readonly {
        readonly x: Pt;
        readonly y: Pt;
    }[];
    readonly label?: ConnectorLabel;
}
export interface PositionedSlide extends Omit<ResolvedSlide, 'children' | 'connectors'>, Positioned {
    readonly children: readonly PositionedNode[];
    /** M12: só os conectores com as duas pontas nesta parte do slide. Ausente = nenhum. */
    readonly connectors?: readonly PositionedConnector[];
}
export interface PositionedDeck {
    readonly size: SlideSize;
    readonly slides: readonly PositionedSlide[];
    /** M15: como em ResolvedDeck; o emissor traduz `from` pelo `sourceSlide` das partes. */
    readonly sections?: readonly DeckSection[];
}
export type Severity = 'error' | 'warning' | 'info';
export type DiagnosticCode = 'text-overflow' | 'text-shrunk' | 'text-truncated' | 'text-word-broken'
/** Fonte reduzida pelo ajuste abaixo de DEFAULTS.text.minLegibleSize (warning). */
 | 'text-illegible' | 'box-overflow' | 'out-of-slide'
/** M19: Container medido com largura ou altura 0 (o pai depende do conteúdo); a função recebe 0 (warning). */
 | 'container-unsized'
/** M19: Container aninhado além de DEFAULTS.containerQuery.maxDepth; o nível de dentro fica vazio (error). */
 | 'container-depth' | 'image-upscaled' | 'font-missing'
/** M8: fonte pedida em embedFonts com fsType restrito; não foi embutida (warning, slide 0, path ''). */
 | 'font-not-embeddable'
/** M9: slide do autor dividido em partes por um Box paginate (info, só na 1ª parte). */
 | 'slide-continued'
/** M10: tabela não coube nem no minSize (ou fit="error"); data: overflowPt/Pct, fromSize, toSize, shown/total linhas. */
 | 'table-overflow'
/** M11: propriedade CSS que chegou ao nó e não tem de/para para o PPTX (warning; não sai no arquivo nem no preview). */
 | 'css-not-emitted'
/** M13: peso pedido sem face registrada a até 100 dele; usada a face mais próxima pelo CSS (warning). */
 | 'font-weight-fallback'
/** M13: `href="slide:N"` com N fora do deck do autor (error; o link não sai). */
 | 'link-target-missing'
/** M14: tabela com ícone em célula: o ícone é uma forma sobre a tabela, não parte dela (info, 1 por tabela). */
 | 'table-cell-icon' | 'placeholder-unused'
/** M12: `from`/`to` de um Connector sem nó com esse id no slide (error; o conector não sai). */
 | 'connector-target-missing'
/** M12: as pontas do conector caíram em partes diferentes de um slide paginado (info; omitido nesta parte). */
 | 'connector-omitted'
/** M12: ícone SVG fora do subconjunto vetorial (ou render="image") virou imagem (info). */
 | 'icon-rasterized'
/** M12: `d` de um Path com comando fora do subconjunto (error; o Path não é desenhado). */
 | 'path-invalid'
/** 1.0.5: `console.warn`/`console.error` de um módulo do deck ou do DS durante o build (warning, slide 0, path ''). */
 | 'module-warning';
export interface Diagnostic {
    readonly severity: Severity;
    readonly code: DiagnosticCode;
    readonly size: string;
    /** Índice do slide, começando em 1. */
    readonly slide: number;
    /** Agrupado: índices entre colchetes, ex.: 'Slide > Box > Box[0..7] > Text' ou 'Box[0,2,5]'. */
    readonly path: string;
    /** Escrita para um LLM ler e agir. */
    readonly message: string;
    readonly data?: {
        readonly overflowPt?: number;
        readonly overflowPct?: number;
        readonly fromSize?: number;
        readonly toSize?: number;
        /** Path do nó cuja demanda de espaço provocou este estouro (ver CONTRACTS §fit "Causa"). */
        readonly causedBy?: string;
        /** Quantos irmãos homônimos este diagnóstico representa (≥ 2 quando agrupado). */
        readonly count?: number;
        /** Paths individuais quando agrupado, na ordem dos índices. */
        readonly paths?: readonly string[];
        /** M8: eixo do estouro de texto; ausente = 'height'. 'width' só com nowrap. */
        readonly axis?: 'width' | 'height';
        /**
         * M9: índice (base 1) do slide no <Deck> do autor. Presente quando difere de `slide`
         * (um slide anterior foi paginado) ou quando o slide é parte de uma paginação.
         */
        readonly sourceSlide?: number;
        /** M9: em 'slide-continued', total de partes em que o slide do autor foi dividido. */
        readonly parts?: number;
        /** M13: em 'slide-continued', o que paginou: um Box (itens) ou um Text (linhas). Ausente = 'box'. */
        readonly kind?: 'box' | 'text' | 'table';
        /** M13: em 'font-weight-fallback', peso pedido e peso da face usada. */
        readonly requestedWeight?: number;
        readonly usedWeight?: number;
        /** M10: em 'table-overflow', linhas de dados inteiramente visíveis e total. */
        readonly shownRows?: number;
        readonly totalRows?: number;
        /** M15: em 'placeholder-unused', o layout e os placeholders sem nó. */
        readonly layout?: string;
        readonly placeholders?: readonly string[];
        /**
         * 1.0.5: o componente do autor (com as props) dentro do qual está o nó, `<Risks items=[4 itens] status="warn" />`
         * (ResolvedBase.source do ancestral mais próximo). Ausente = o nó está fora de componentes.
         */
        readonly component?: string;
    };
}
/** M15: as 12 cores do tema do arquivo (as do seletor de cores do PowerPoint/Keynote). Ausentes = Office. */
export interface ThemeColors {
    readonly dark1?: Color;
    readonly light1?: Color;
    readonly dark2?: Color;
    readonly light2?: Color;
    readonly accent1?: Color;
    readonly accent2?: Color;
    readonly accent3?: Color;
    readonly accent4?: Color;
    readonly accent5?: Color;
    readonly accent6?: Color;
    readonly hyperlink?: Color;
    readonly followedHyperlink?: Color;
}
export type ThemeColorName = keyof ThemeColors;
/** M15: famílias do tema; um Text nessa família sai como fonte do tema (+mj-lt / +mn-lt). */
export interface ThemeFonts {
    readonly heading?: string;
    readonly body?: string;
}
/**
 * M15: tema do arquivo. Cor de nó **igual** (hex) a uma cor do tema sai como referência ao tema;
 * trocar a cor do tema no PowerPoint repinta essas formas. Não muda layout nem preview.
 */
export interface Theme {
    readonly name?: string;
    readonly colors?: ThemeColors;
    readonly fonts?: ThemeFonts;
}
/** M15: tema normalizado (cores '#RRGGBB'); `declared` = cores que o autor deu (só elas viram referência ao tema). */
export interface ResolvedTheme {
    readonly name: string;
    readonly colors: Readonly<Record<ThemeColorName, Color>>;
    readonly declared: readonly ThemeColorName[];
    readonly fonts: ThemeFonts;
}
/** M15: propriedades do documento (Arquivo → Propriedades). */
export interface DocumentProperties {
    readonly title?: string;
    readonly author?: string;
    readonly company?: string;
    readonly subject?: string;
    readonly keywords?: string | readonly string[];
}
export interface PreviewOptions {
    readonly mode: 'native';
    /** Largura em px de cada slide; padrão 1280. */
    readonly width?: number;
}
export interface RenderOptions {
    /** padrão ['16x9'] */
    readonly sizes?: readonly SizeSpec[];
    readonly fonts?: readonly FontFace[];
    /** padrão { mode: 'native', width: 1280 }; false desliga. */
    readonly preview?: PreviewOptions | false;
    readonly debug?: boolean;
    /** Base para resolver src relativos de imagens; padrão process.cwd(). */
    readonly baseDir?: string;
    /** Idioma do texto (corretor ortográfico do PowerPoint); padrão 'pt-BR'. */
    readonly lang?: string;
    /** Fator de segurança da quebra de linha; padrão 0.97. */
    readonly textSafety?: number;
    /**
     * 'forced' (padrão): cada linha calculada vira <a:br/>; o renderizador não requebra (calibração 34/34).
     * 'auto': só parágrafos; o renderizador quebra com a própria métrica (mais editável, pode divergir 1 linha).
     */
    readonly lineBreaks?: LineBreaks;
    /** Inclui a árvore posicionada (pós-fit) no resultado, para ferramentas e previews externos. Padrão false. */
    readonly includePositioned?: boolean;
    /**
     * M8: embute no .pptx (EOT em ppt/fonts/) as faces registradas que o deck usa, exceto a Arial
     * padrão (Liberation não pode sair sob o nome Arial) e fontes com fsType restrito. Padrão false.
     */
    readonly embedFonts?: boolean;
    /** M15: tema de cores e fontes do arquivo; ausente = tema padrão do pptxgenjs, cores em hex. */
    readonly theme?: Theme;
    /** M15: título, autor etc. A CLI usa o nome do arquivo do deck como título padrão. */
    readonly document?: DocumentProperties;
}
export type LineBreaks = 'forced' | 'auto';
export interface RenderResult {
    readonly size: string;
    readonly buffer: Buffer;
    readonly diagnostics: readonly Diagnostic[];
    readonly preview?: {
        readonly slides: readonly Buffer[];
        readonly overview: Buffer;
    };
    /** Só com RenderOptions.includePositioned. */
    readonly positioned?: PositionedDeck;
    /** M8: faces pedidas em `fonts` sem `file` que não foram achadas no sistema ("Inter bold"). */
    readonly fontsMissing?: readonly string[];
}
export {};
