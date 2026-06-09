import type { AgentRole, AgentTask, ApiJobKind, ApiJobStatus, ChapterStatus, NovelStatus, TargetPlatform } from "../types/domain";

export const platformLabels: Record<TargetPlatform, string> = {
  qidian: "起点",
  fanqie: "番茄",
  general: "通用",
};

export const novelStatusLabels: Record<NovelStatus, string> = {
  draft: "草稿",
  active: "连载",
  completed: "完结",
  archived: "归档",
};

export const chapterStatusLabels: Record<ChapterStatus, string> = {
  outlined: "有大纲",
  drafted: "已起草",
  reviewed: "已审稿",
  rewrite_needed: "需返工",
  final: "定稿",
};

export const agentRoleLabels: Record<AgentRole, string> = {
  orchestrator: "编排",
  market: "市场",
  plot: "大纲",
  character: "人物",
  worldbuilding: "世界观",
  writer: "写作",
  continuity: "连续性",
  style: "风格",
  reviewer: "审稿",
};

export const agentTaskLabels: Record<AgentTask, string> = {
  create_novel: "新建小说",
  generate_outline: "生成大纲",
  generate_chapter: "生成章节",
  review_chapter: "审稿",
  rewrite_chapter: "重写",
  extract_facts: "抽取事实",
  polish_style: "风格润色",
  check_continuity: "连续性检查",
};

export const jobKindLabels: Record<ApiJobKind, string> = {
  create_novel: "新建小说",
  write_chapter: "写章节",
  write_chapters: "批量写章节",
  review_chapter: "审稿",
  rewrite_chapter: "返工",
};

export const jobStatusLabels: Record<ApiJobStatus, string> = {
  queued: "排队中",
  running: "运行中",
  succeeded: "已完成",
  failed: "失败",
  cancelled: "已取消",
};

export function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  return `${(ms / 1000).toFixed(1)}s`;
}

export function countWords(content: string): number {
  return content.replace(/\s/g, "").length;
}

export function clampScore(score?: number | null): string {
  return typeof score === "number" ? String(score) : "-";
}
