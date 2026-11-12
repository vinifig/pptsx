import { Box, Text } from 'pptsx';
import { defineComponent } from '@pptsx/ds-kit';
import { tokens } from '../tokens.ts';

export type GanttStatus = 'done' | 'active' | 'planned' | 'risk';
export type GanttDetail = 'tactical' | 'executive';

export interface GanttTask {
  readonly label: string;
  /** Início, em períodos a partir de 0 (o 1º de `periods`); aceita fração (1.5 = meio do 2º período). */
  readonly start: number;
  /** Fim exclusivo, na mesma escala: `start: 0, end: 2` ocupa os dois primeiros períodos. */
  readonly end: number;
  /** Frente ou fase da tarefa. No modo executivo, cada grupo vira uma barra só, do início ao fim dele. */
  readonly group?: string;
  /** Define a cor da barra; padrão 'planned'. */
  readonly status?: GanttStatus;
  /** Dono ou time, escrito dentro da barra (só no modo tático). */
  readonly owner?: string;
}

export interface GanttMilestone {
  readonly label: string;
  /** Posição na escala dos períodos. */
  readonly at: number;
}

export interface GanttProps {
  /** Rótulos das colunas de tempo, na ordem (meses, sprints, trimestres). */
  readonly periods: readonly string[];
  readonly tasks: readonly GanttTask[];
  readonly milestones?: readonly GanttMilestone[];
  /**
   * 'tactical' (padrão): uma linha por tarefa, com dono, agrupada pelas frentes. 'executive': uma linha por
   * frente, do início ao fim dela, com o status mais crítico; para diretoria e visão de portfólio.
   */
  readonly detail?: GanttDetail;
  /** Marca "hoje" na escala dos períodos (ex.: 2.5 = meio do 3º período). */
  readonly today?: number;
}

const { color, font, space } = tokens;

/** Medidas por nível de detalhe (pt): o executivo tem menos linhas, então cada uma é mais alta e legível. */
const METRICS: Readonly<Record<GanttDetail, { row: number; gap: number; label: number; text: number; column: number }>> = {
  tactical: { row: 20, gap: 6, label: font.small - 2, text: font.caption - 1, column: 170 },
  executive: { row: 34, gap: 12, label: font.body - 4, text: font.small - 2, column: 170 },
};

const STATUS_FILL: Readonly<Record<GanttStatus, string>> = {
  done: color.muted,
  active: color.accent,
  planned: color.accentSoft,
  risk: color.negative,
};

const STATUS_INK: Readonly<Record<GanttStatus, string>> = { done: color.white, active: color.white, planned: color.ink, risk: color.white };

/** Do mais ao menos crítico: a barra de uma frente leva o pior status das tarefas dela. */
const SEVERITY: readonly GanttStatus[] = ['risk', 'active', 'planned', 'done'];

interface Row {
  readonly label: string;
  readonly start: number;
  readonly end: number;
  readonly status: GanttStatus;
  readonly owner?: string;
  /** Linha de título de frente (sem barra). */
  readonly heading?: boolean;
}

const groupsOf = (tasks: readonly GanttTask[]): readonly string[] => [...new Set(tasks.map((task) => task.group ?? ''))];

const worst = (tasks: readonly GanttTask[]): GanttStatus => SEVERITY.find((status) => tasks.some((task) => (task.status ?? 'planned') === status)) ?? 'planned';

const executiveRows = (tasks: readonly GanttTask[]): readonly Row[] =>
  groupsOf(tasks).map((group) => {
    const own = tasks.filter((task) => (task.group ?? '') === group);
    return { label: group || own[0]!.label, start: Math.min(...own.map((t) => t.start)), end: Math.max(...own.map((t) => t.end)), status: worst(own) };
  });

const tacticalRows = (tasks: readonly GanttTask[]): readonly Row[] => {
  const groups = groupsOf(tasks);
  const titled = groups.length > 1 || groups[0] !== '';
  return groups.flatMap((group) => {
    const own: readonly Row[] = tasks.filter((task) => (task.group ?? '') === group).map((task) => ({ label: task.label, start: task.start, end: task.end, status: task.status ?? 'planned', ...(task.owner ? { owner: task.owner } : {}) }));
    return titled && group !== '' ? [{ label: group, start: 0, end: 0, status: 'planned' as const, heading: true }, ...own] : own;
  });
};

/** Posição na escala como porcentagem da trilha (a trilha é a largura que sobra depois da coluna de rótulos). */
const percent = (value: number, total: number): string => `${(Math.max(0, Math.min(value, total)) / total) * 100}%`;

/**
 * Gantt de roadmap feito de Box (editável no PowerPoint, sem imagem): coluna de rótulos à esquerda e uma
 * trilha por linha, com a barra em `position="absolute"` e `left`/`width` em porcentagem da trilha. Assim o
 * mesmo código serve em 16:9 e 4:3, e a escala só depende do número de períodos. As linhas de grade e o
 * "hoje" ficam numa camada absoluta atrás das trilhas (zIndex -1 / 1), do topo ao fim da última linha.
 */
function GanttView({ periods, tasks, milestones = [], detail = 'tactical', today }: GanttProps) {
  const m = METRICS[detail];
  const total = Math.max(periods.length, 1);
  const rows = detail === 'executive' ? executiveRows(tasks) : tacticalRows(tasks);
  const marks = milestones.filter((ms) => ms.at >= 0 && ms.at <= total);
  return (
    <Box gap={m.gap} fontSize={m.label} color={color.ink}>
      <Box display="flex" flexDirection="row">
        <Box width={m.column} flexShrink={0} />
        <Box flex={1} display="flex" flexDirection="row">
          {periods.map((period) => (
            <Text flex={1} fontSize={m.text + 1} fontWeight="bold" color={color.muted} nowrap>{period}</Text>
          ))}
        </Box>
      </Box>
      <Box display="flex" flexDirection="row">
        <Box width={m.column} flexShrink={0} gap={m.gap} paddingRight={space.tight}>
          {rows.map((row) => (
            <Box height={m.row} justifyContent="center">
              <Text fontWeight={row.heading ? 'bold' : 'normal'} color={row.heading ? color.muted : color.ink} maxLines={1} fit="truncate">{row.label}</Text>
            </Box>
          ))}
        </Box>
        <Box flex={1} gap={m.gap}>
          {periods.slice(1).map((_, index) => (
            <Box position="absolute" left={percent(index + 1, total)} top={0} bottom={0} width={0.75} backgroundColor={color.surface} zIndex={-1} />
          ))}
          {rows.map((row) => (
            <Box height={m.row}>
              {row.heading ? null : (
                <Box position="absolute" left={percent(row.start, total)} width={percent(row.end - row.start, total)} top={0} bottom={0}
                  backgroundColor={STATUS_FILL[row.status]} borderRadius={m.row / 4} justifyContent="center" paddingLeft={4}>
                  {detail === 'tactical' && row.owner ? <Text fontSize={m.text} color={STATUS_INK[row.status]} maxLines={1} fit="truncate">{row.owner}</Text> : null}
                </Box>
              )}
            </Box>
          ))}
          {today === undefined ? null : (
            <Box position="absolute" left={percent(today, total)} top={0} bottom={0} width={1.5} backgroundColor={color.negative} zIndex={1} />
          )}
        </Box>
      </Box>
      {marks.length === 0 ? null : (
        <Box display="flex" flexDirection="row">
          <Box width={m.column} flexShrink={0}>
            <Text fontSize={m.text + 1} fontWeight="bold" color={color.muted}>Marcos</Text>
          </Box>
          <Box flex={1} height={m.row + m.text}>
            {marks.map((ms) => (
              <Text position="absolute" left={percent(ms.at, total)} top={0} fontSize={m.text + 1} color={color.ink} nowrap>{`▲ ${ms.label}`}</Text>
            ))}
          </Box>
        </Box>
      )}
    </Box>
  );
}

const PERIODS = ['Jan', 'Fev', 'Mar', 'Abr', 'Mai', 'Jun'];
const TASKS: readonly GanttTask[] = [
  { group: 'Plataforma', label: 'Migração do cluster', start: 0, end: 2, status: 'done', owner: 'Infra' },
  { group: 'Plataforma', label: 'Observabilidade', start: 1.5, end: 3.5, status: 'active', owner: 'SRE' },
  { group: 'Produto', label: 'Checkout novo', start: 1, end: 4, status: 'risk', owner: 'Squad Pagamentos' },
  { group: 'Produto', label: 'App v3', start: 3, end: 6, status: 'planned', owner: 'Squad Mobile' },
  { group: 'Dados', label: 'Catálogo de dados', start: 2, end: 5, status: 'active', owner: 'Data Eng' },
];
const MILESTONES: readonly GanttMilestone[] = [{ label: 'Beta', at: 3 }, { label: 'GA', at: 5.5 }];

export const Gantt = defineComponent<GanttProps>(
  {
    name: 'Gantt',
    ds: '@pptsx/ds-base',
    kind: 'chart',
    category: 'Roadmap',
    description: 'Roadmap em Gantt nativo (barras editáveis): frentes, tarefas, marcos e a linha de hoje, numa escala de períodos.',
    audience: ['tactical', 'executive'],
    whenToUse: [
      'Roadmap, cronograma ou plano de entregas com datas relativas (meses, sprints, trimestres).',
      "Para diretoria, detail=\"executive\": uma barra por frente, com o status mais crítico. Para o time, 'tactical' com dono por tarefa.",
    ],
    avoid: ['Mais de 12 linhas no modo tático: divida por frente em slides seguidos ou use o executivo.', 'Datas absolutas no meio da barra: a escala é por período, escreva as datas nos rótulos de `periods`.'],
    tags: ['roadmap', 'cronograma', 'dados'],
    catalog: [
      { name: 'tático', description: 'Uma linha por tarefa, agrupada por frente, com o dono na barra.', audience: ['tactical'], props: { periods: PERIODS, tasks: TASKS, milestones: MILESTONES, today: 2.5 } },
      { name: 'executivo', description: 'Uma barra por frente com o pior status dela; para diretoria.', audience: ['executive'], props: { periods: PERIODS, tasks: TASKS, milestones: MILESTONES, today: 2.5, detail: 'executive' } },
    ],
  },
  GanttView,
);
