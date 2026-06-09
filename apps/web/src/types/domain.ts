export type TargetPlatform = "general" | "qidian" | "fanqie";
export type NovelStatus = "draft" | "active" | "completed" | "archived";
export type ChapterStatus =
  | "outlined"
  | "drafted"
  | "reviewed"
  | "rewrite_needed"
  | "final";
export type AgentRole =
  | "orchestrator"
  | "market"
  | "plot"
  | "character"
  | "worldbuilding"
  | "writer"
  | "continuity"
  | "style"
  | "reviewer";
export type AgentTask =
  | "create_novel"
  | "generate_outline"
  | "generate_chapter"
  | "review_chapter"
  | "rewrite_chapter"
  | "extract_facts"
  | "polish_style"
  | "check_continuity";
export type AgentRunStatus = "ok" | "fallback" | "parse_error" | "running";
export type ApiJobKind = "create_novel" | "write_chapter" | "write_chapters" | "review_chapter" | "rewrite_chapter";
export type ApiJobStatus = "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface Novel {
  id: string;
  title: string;
  genre: string;
  target_platform: TargetPlatform;
  status: NovelStatus;
  created_at: string;
  updated_at: string;
}

export interface NovelListItem extends Novel {
  chapter_count: number;
  recent_score: number | null;
}

export interface TitleCandidate {
  title: string;
  reason: string;
}

export interface OpeningStrategy {
  first_scene: string;
  first_conflict: string;
  first_three_chapters_goal: string;
}

export interface PlatformProfile {
  target_platform: TargetPlatform;
  opening_speed: string;
  setup_ratio: number;
  dialogue_ratio: number;
  payoff_frequency: string;
  cliffhanger_strength: string;
  review_bias: Record<string, unknown>;
}

export interface NovelBible {
  novel_id: string;
  title_candidates: TitleCandidate[];
  premise: string;
  genre: string;
  target_platform: TargetPlatform;
  target_readers: string;
  core_selling_points: string[];
  reader_expectations: string[];
  main_conflict: string;
  protagonist_goal: string;
  emotional_value: string;
  tone: string;
  platform_tags: string[];
  world_rules: string[];
  constraints: string[];
  opening_strategy: OpeningStrategy;
  platform_profile?: PlatformProfile;
}

export interface FactTriple {
  subject: string;
  predicate: string;
  object: string;
  importance: number;
}

export interface Fact extends FactTriple {
  id: string;
  novel_id: string;
  chapter_id?: string | null;
  created_at: string;
}

export interface CharacterRelationship {
  target: string;
  relationship: string;
  tension: string;
}

export interface CharacterArc {
  start: string;
  turning_points: string[];
  expected_end: string;
}

export interface CharacterCard {
  id: string;
  novel_id: string;
  id_hint: string;
  name: string;
  role: string;
  identity: string;
  personality: string[];
  desire: string;
  motivation: string;
  secret: string;
  abilities: string[];
  limitations: string[];
  current_state: string;
  relationship_map: CharacterRelationship[];
  arc: CharacterArc;
  first_appearance_chapter: number;
  chapter_1_to_30_plan: string[];
}

export interface WorldSetting {
  genre_type: string;
  overview: string;
  power_system: {
    name: string;
    levels: string[];
    rules: string[];
    costs: string[];
    limits: string[];
  };
  organizations: Array<{
    name: string;
    role: string;
    resources: string[];
    conflicts: string[];
  }>;
  locations: Array<{
    name: string;
    description: string;
    story_use: string;
  }>;
  taboos: string[];
  hard_rules: string[];
}

export interface ChapterOutline {
  novel_id: string;
  volume_index: number;
  chapter_index: number;
  title: string;
  pov: string;
  goal: string;
  conflict: string;
  key_events: string[];
  character_changes: string[];
  new_facts: FactTriple[];
  payoff: string;
  foreshadowing: string[];
  cliffhanger: string;
  estimated_word_count: number;
}

export interface Chapter {
  id: string;
  novel_id: string;
  volume_index: number;
  chapter_index: number;
  title: string;
  outline: string;
  content?: string | null;
  summary?: string | null;
  status: ChapterStatus;
  score?: number | null;
  word_count: number;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface Foreshadowing {
  seed: string;
  status: "planted" | "advanced" | "paid_off" | "contradicted";
  expected_payoff: string;
}

export interface ChapterDraft {
  volume_index: number;
  chapter_id: string;
  novel_id: string;
  chapter_index: number;
  title: string;
  content: string;
  summary: string;
  word_count: number;
  pov: string;
  key_events: string[];
  new_facts: FactTriple[];
  foreshadowing: Foreshadowing[];
  continuity_notes: string[];
  version: number;
}

export interface ReviewScores {
  opening_hook_score: number;
  pacing_score: number;
  payoff_score: number;
  character_score: number;
  dialogue_score: number;
  continuity_score: number;
  cliffhanger_score: number;
  platform_fit_score: number;
}

export interface ReviewIssue {
  severity: "low" | "medium" | "high";
  dimension:
    | "opening_hook"
    | "pacing"
    | "payoff"
    | "character"
    | "dialogue"
    | "continuity"
    | "cliffhanger"
    | "platform_fit";
  location: string;
  description: string;
}

export interface RewriteDecision {
  needed: boolean;
  rewrite_type: "none" | "partial" | "full";
  priority: "low" | "medium" | "high";
  goals: string[];
  preserve: string[];
  change: string[];
  avoid: string[];
}

export interface ReviewReport {
  id: string;
  chapter_id: string;
  total_score: number;
  passed: boolean;
  scores: ReviewScores;
  strengths: string[];
  issues: ReviewIssue[];
  suggestions: string[];
  rewrite_instruction: RewriteDecision;
  created_at: string;
}

export interface ChapterVersion {
  id: string;
  chapter_id: string;
  version: number;
  title: string;
  content: string;
  summary: string;
  word_count: number;
  data: {
    source: "writer" | "rewrite" | "manual_edit";
    score?: number | null;
    notes?: string;
  };
  created_at: string;
}

export interface AgentRun {
  id: string;
  novel_id?: string | null;
  role: AgentRole;
  task: AgentTask;
  provider: "smoke" | "openai" | "deepseek";
  status: AgentRunStatus;
  attempt?: number | null;
  duration_ms: number;
  total_tokens?: number | null;
  output_summary: string;
  structured: Record<string, unknown>;
  raw_text: string;
  raw_notes: string;
  parse_error?: string | null;
  created_at: string;
}

export interface AgentRunStatusSummary {
  total: number;
  ok: number;
  fallback: number;
  parse_error: number;
  duration_ms_total: number;
  tokenized_runs: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

export interface AgentRunReport {
  runs: AgentRun[];
  summary: AgentRunStatusSummary;
}

export interface ApiJob {
  id: string;
  kind: ApiJobKind;
  status: ApiJobStatus;
  novel_id?: string | null;
  chapter_index?: number | null;
  source_job_id?: string | null;
  progress_current?: number;
  progress_total?: number;
  payload: Record<string, unknown>;
  result?: Record<string, unknown> | null;
  error?: string | null;
  created_at: string;
  updated_at: string;
}

export interface NovelDetail {
  novel: Novel;
  bible: NovelBible;
  characters: CharacterCard[];
  world_setting: WorldSetting;
  chapter_outlines: ChapterOutline[];
  facts: Fact[];
}

export interface CreateNovelInput {
  idea: string;
  genre: string;
  target_platform: TargetPlatform;
  target_words: number;
  chapter_words: number;
}
