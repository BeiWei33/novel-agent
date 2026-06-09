import type { FormEvent } from "react";
import { useState } from "react";
import { Clock3, Loader2, PlusCircle } from "lucide-react";
import type { CreateNovelInput, TargetPlatform } from "../../types/domain";
import { Button } from "../../components/ui/Button";

const platformOptions: Array<{ value: TargetPlatform; label: string }> = [
  { value: "qidian", label: "起点" },
  { value: "fanqie", label: "番茄" },
  { value: "general", label: "通用" },
];

export function NewNovelForm({
  onSubmit,
  onQueueSubmit,
  isPending,
  isQueueing,
}: {
  onSubmit: (input: CreateNovelInput) => void;
  onQueueSubmit: (input: CreateNovelInput) => void;
  isPending: boolean;
  isQueueing: boolean;
}) {
  const [idea, setIdea] = useState("都市重生商业文，主角回到十年前暴雨外卖站，用调度经验避开事故并拿下第一份真实订单。");
  const [genre, setGenre] = useState("都市重生商业文");
  const [targetPlatform, setTargetPlatform] = useState<TargetPlatform>("fanqie");
  const [targetWords, setTargetWords] = useState(1_200_000);
  const [chapterWords, setChapterWords] = useState(2600);
  const [targetChapters, setTargetChapters] = useState(30);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onSubmit(currentInput());
  }

  function currentInput(): CreateNovelInput {
    return {
      idea,
      genre,
      target_platform: targetPlatform,
      target_words: targetWords,
      chapter_words: chapterWords,
      target_chapters: targetChapters,
    };
  }

  return (
    <form onSubmit={handleSubmit} className="max-w-4xl space-y-5 p-4">
      <label className="block space-y-2">
        <span className="field-label">创意描述</span>
        <textarea value={idea} onChange={(event) => setIdea(event.target.value)} className="textarea min-h-36" required />
      </label>

      <div className="grid gap-4 md:grid-cols-2">
        <label className="block space-y-2">
          <span className="field-label">题材</span>
          <input value={genre} onChange={(event) => setGenre(event.target.value)} className="input" required />
        </label>
        <label className="block space-y-2">
          <span className="field-label">目标平台</span>
          <select value={targetPlatform} onChange={(event) => setTargetPlatform(event.target.value as TargetPlatform)} className="input">
            {platformOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label className="block space-y-2">
          <span className="field-label">规划章节数</span>
          <input
            type="number"
            min={1}
            max={300}
            step={1}
            value={targetChapters}
            onChange={(event) => setTargetChapters(Number(event.target.value))}
            className="input"
          />
        </label>
        <label className="block space-y-2">
          <span className="field-label">目标字数</span>
          <input
            type="number"
            min={100000}
            step={50000}
            value={targetWords}
            onChange={(event) => setTargetWords(Number(event.target.value))}
            className="input"
          />
        </label>
        <label className="block space-y-2">
          <span className="field-label">章节字数</span>
          <input
            type="number"
            min={1000}
            step={100}
            value={chapterWords}
            onChange={(event) => setChapterWords(Number(event.target.value))}
            className="input"
          />
        </label>
      </div>

      <div className="flex flex-wrap gap-2">
      <Button type="submit" variant="primary" disabled={isPending || isQueueing}>
        {isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <PlusCircle className="h-4 w-4" />}
        创建小说
      </Button>
        <Button type="button" variant="secondary" disabled={isPending || isQueueing} onClick={() => onQueueSubmit(currentInput())}>
          {isQueueing ? <Loader2 className="h-4 w-4 animate-spin" /> : <Clock3 className="h-4 w-4" />}
          后台创建
        </Button>
      </div>
    </form>
  );
}
