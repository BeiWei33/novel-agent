import { useState } from "react";
import type { NovelDetail } from "../../types/domain";
import { Tabs } from "../../components/ui/Tabs";
import { Badge } from "../../components/ui/Badge";
import { Section } from "../../components/ui/Section";
import { platformLabels } from "../../lib/format";

type BibleTab = "bible" | "characters" | "world" | "outline" | "facts" | "foreshadowing";

const tabs: Array<{ value: BibleTab; label: string }> = [
  { value: "bible", label: "小说圣经" },
  { value: "characters", label: "人物卡" },
  { value: "world", label: "世界观" },
  { value: "outline", label: "大纲" },
  { value: "facts", label: "事实表" },
  { value: "foreshadowing", label: "伏笔表" },
];

export function BibleTabs({ detail }: { detail: NovelDetail }) {
  const [tab, setTab] = useState<BibleTab>("bible");

  return (
    <div className="bg-white">
      <div className="border-b border-line px-4 py-3">
        <Tabs value={tab} items={tabs} onChange={setTab} />
      </div>
      {tab === "bible" ? <BibleView detail={detail} /> : null}
      {tab === "characters" ? <CharacterView detail={detail} /> : null}
      {tab === "world" ? <WorldView detail={detail} /> : null}
      {tab === "outline" ? <OutlineView detail={detail} /> : null}
      {tab === "facts" ? <FactsView detail={detail} /> : null}
      {tab === "foreshadowing" ? <ForeshadowingView detail={detail} /> : null}
    </div>
  );
}

function BibleView({ detail }: { detail: NovelDetail }) {
  const bible = detail.bible;
  return (
    <div className="grid gap-0 lg:grid-cols-[minmax(0,1.35fr)_minmax(300px,0.65fr)]">
      <Section title="核心资料">
        <dl className="grid gap-4 p-4 md:grid-cols-2">
          <Field label="一句话卖点" value={bible.premise} wide />
          <Field label="主冲突" value={bible.main_conflict} wide />
          <Field label="主角目标" value={bible.protagonist_goal} />
          <Field label="情绪价值" value={bible.emotional_value} />
          <Field label="目标读者" value={bible.target_readers} />
          <Field label="文风" value={bible.tone} />
        </dl>
      </Section>
      <Section title="平台策略">
        <div className="space-y-4 p-4 text-sm">
          <div className="flex flex-wrap gap-2">
            <Badge tone="blue">{platformLabels[bible.target_platform]}</Badge>
            {bible.platform_tags.map((tag) => (
              <Badge key={tag}>{tag}</Badge>
            ))}
          </div>
          <List title="核心卖点" values={bible.core_selling_points} />
          <List title="读者期待" values={bible.reader_expectations} />
          <List title="硬限制" values={bible.constraints} />
        </div>
      </Section>
    </div>
  );
}

function CharacterView({ detail }: { detail: NovelDetail }) {
  return (
    <div className="grid gap-3 p-4 xl:grid-cols-3">
      {detail.characters.map((character) => (
        <article key={character.id} className="rounded-md border border-border bg-white p-4 shadow-soft">
          <div className="mb-3 flex items-start justify-between gap-2">
            <div>
              <h3 className="text-sm font-semibold text-ink">{character.name}</h3>
              <p className="text-xs text-slate-500">{character.identity}</p>
            </div>
            <Badge tone={character.role === "protagonist" ? "teal" : character.role === "antagonist" ? "rose" : "blue"}>
              {character.role}
            </Badge>
          </div>
          <List title="性格" values={character.personality} />
          <Field label="欲望" value={character.desire} />
          <Field label="秘密" value={character.secret} />
          <Field label="当前状态" value={character.current_state} />
        </article>
      ))}
    </div>
  );
}

function WorldView({ detail }: { detail: NovelDetail }) {
  const world = detail.world_setting;
  return (
    <div className="grid gap-0 lg:grid-cols-2">
      <Section title="世界观">
        <div className="space-y-4 p-4">
          <Field label="类型" value={world.genre_type} />
          <Field label="总述" value={world.overview} />
          <Field label="体系" value={world.power_system.name} />
          <List title="规则" values={world.power_system.rules} />
          <List title="代价" values={world.power_system.costs} />
        </div>
      </Section>
      <Section title="组织与地点">
        <div className="space-y-4 p-4">
          {world.organizations.map((org) => (
            <div key={org.name} className="border-b border-line pb-3 last:border-0 last:pb-0">
              <h3 className="text-sm font-semibold">{org.name}</h3>
              <p className="mt-1 text-sm text-slate-600">{org.role}</p>
            </div>
          ))}
          {world.locations.map((location) => (
            <div key={location.name} className="border-b border-line pb-3 last:border-0 last:pb-0">
              <h3 className="text-sm font-semibold">{location.name}</h3>
              <p className="mt-1 text-sm text-slate-600">{location.story_use}</p>
            </div>
          ))}
        </div>
      </Section>
    </div>
  );
}

function OutlineView({ detail }: { detail: NovelDetail }) {
  return (
    <div className="overflow-x-auto">
      <table className="min-w-[900px] w-full border-collapse text-sm">
        <thead className="table-head">
          <tr>
            <th className="px-4 py-3">章节</th>
            <th className="px-3 py-3">目标</th>
            <th className="px-3 py-3">冲突</th>
            <th className="px-3 py-3">章尾</th>
            <th className="px-3 py-3 text-right">预计字数</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {detail.chapter_outlines.map((outline) => (
            <tr key={outline.chapter_index} className="hover:bg-slate-50">
              <td className="px-4 py-3 font-medium">{outline.title}</td>
              <td className="px-3 py-3 text-slate-600">{outline.goal}</td>
              <td className="px-3 py-3 text-slate-600">{outline.conflict}</td>
              <td className="px-3 py-3 text-slate-600">{outline.cliffhanger}</td>
              <td className="px-3 py-3 text-right tabular-nums">{outline.estimated_word_count}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function FactsView({ detail }: { detail: NovelDetail }) {
  return (
    <div className="overflow-x-auto">
      <table className="min-w-[760px] w-full border-collapse text-sm">
        <thead className="table-head">
          <tr>
            <th className="px-4 py-3">主体</th>
            <th className="px-3 py-3">关系</th>
            <th className="px-3 py-3">客体</th>
            <th className="px-3 py-3 text-right">重要性</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-line">
          {detail.facts.map((fact) => (
            <tr key={fact.id}>
              <td className="px-4 py-3 font-medium">{fact.subject}</td>
              <td className="px-3 py-3 text-slate-600">{fact.predicate}</td>
              <td className="px-3 py-3 text-slate-600">{fact.object}</td>
              <td className="px-3 py-3 text-right tabular-nums">{fact.importance}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ForeshadowingView({ detail }: { detail: NovelDetail }) {
  const rows = detail.chapter_outlines.flatMap((outline) =>
    outline.foreshadowing.map((seed) => ({
      chapter: outline.chapter_index,
      seed,
      payoff: outline.payoff,
      cliffhanger: outline.cliffhanger,
    })),
  );

  return (
    <div className="grid gap-3 p-4 lg:grid-cols-2">
      {rows.slice(0, 12).map((row) => (
        <article key={`${row.chapter}-${row.seed}`} className="rounded-md border border-border bg-white p-4 shadow-soft">
          <div className="mb-2 flex items-center justify-between">
            <Badge tone="blue">第 {row.chapter} 章</Badge>
            <span className="text-xs text-slate-500">planted</span>
          </div>
          <h3 className="text-sm font-semibold text-ink">{row.seed}</h3>
          <p className="mt-2 text-sm leading-6 text-slate-600">{row.payoff}</p>
        </article>
      ))}
    </div>
  );
}

function Field({ label, value, wide = false }: { label: string; value: string; wide?: boolean }) {
  return (
    <div className={wide ? "md:col-span-2" : undefined}>
      <dt className="field-label">{label}</dt>
      <dd className="mt-1 text-sm leading-6 text-slate-700">{value}</dd>
    </div>
  );
}

function List({ title, values }: { title: string; values: string[] }) {
  return (
    <div>
      <div className="field-label mb-2">{title}</div>
      <div className="flex flex-wrap gap-2">
        {values.map((value) => (
          <Badge key={value} tone="slate">
            {value}
          </Badge>
        ))}
      </div>
    </div>
  );
}
