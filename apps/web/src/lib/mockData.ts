import type {
  AgentRun,
  ApiJob,
  Chapter,
  ChapterOutline,
  ChapterVersion,
  CharacterCard,
  Fact,
  Novel,
  NovelBible,
  ReviewReport,
  TargetPlatform,
  WorldSetting,
} from "../types/domain";
import { countWords } from "./format";

const baseTime = new Date("2026-06-09T08:00:00+08:00").getTime();

function minutesAgo(minutes: number): string {
  return new Date(baseTime - minutes * 60_000).toISOString();
}

function makeId(prefix: string, value: string | number): string {
  return `${prefix}-${value}`;
}

const chapterOneContent = `# 第1章 旧账重开

雨下得像一场迟到十年的清算。

林砚站在十八岁那年的公交站牌下，指尖还残留着上一世病房里消毒水的冷味。站牌背后贴着网吧开业的传单，手机屏幕显示的时间却清清楚楚：2008 年 6 月 9 日。

他没有立刻狂喜。

上一世，他从这里开始错过父亲的最后一通电话，错过母亲被债主围堵前的求救，也错过了那个会在十年后掀翻整个行业的支付入口。

这一次，公交车还没来，父亲的号码先亮了起来。

“砚子，别回家，先去你三叔那儿。”电话那头的声音压得很低，像被人捂住了半截，“厂里账本出事了。”

林砚抬头，看见街角那家小卖部的玻璃门里，穿灰夹克的男人正借着买烟的动作看他。上一世他记得这个人，债务公司的打手，后来成了压垮林家的第一块石头。

“爸，你听我说。”林砚把书包带往肩上一勒，“账本不要给三叔，三叔已经签了转让协议。”

电话里安静了三秒。

“你怎么知道？”

林砚没有解释。他穿过雨幕，径直走向小卖部。灰夹克明显一愣，手里的烟盒差点掉到地上。

“哥，借个火。”林砚笑了一下。

灰夹克下意识摸口袋。

就在这一瞬间，林砚抓住柜台上的固定电话，拨给了市电视台民生热线。上一世他用了七年才学会一件事：成年人最怕的不是拳头，是突然亮起来的镜头。

“您好，我实名举报城南机械厂账务造假，相关人员正在胁迫未成年人家属转移证据。”

灰夹克的脸色变了。

雨声更大，公交车缓缓停下。林砚看着车窗里十八岁的自己，终于确定命运给他的不是补偿，而是一张还没有被涂黑的草稿纸。

他要先把家救回来。

然后，把上一世踩着林家尸骨上位的人，一个一个请回棋盘。`;

const rewriteContent = `${chapterOneContent}

公交车门合上的瞬间，林砚没有上车。他把手机卡拔出来，塞进站牌底座的缝隙里，像藏下一枚刚刚点燃的火种。

灰夹克追出小卖部时，民生热线的记者已经回拨。林砚接起电话，只说了一句：“来城南机械厂后门，账本会自己走出来。”

这一世，他不只要报警。

他要让所有人都以为，林家手里还有第二本账。`;

const initialNovels: Novel[] = [
  {
    id: "novel-urban-rebirth",
    title: "重生日记：我在十八岁重开商业帝国",
    genre: "都市重生",
    target_platform: "qidian",
    status: "active",
    created_at: minutesAgo(9000),
    updated_at: minutesAgo(18),
  },
  {
    id: "novel-fantasy-upgrade",
    title: "万相骨",
    genre: "玄幻升级",
    target_platform: "fanqie",
    status: "active",
    created_at: minutesAgo(7000),
    updated_at: minutesAgo(96),
  },
  {
    id: "novel-romance-comeback",
    title: "春潮不渡",
    genre: "现代情感",
    target_platform: "general",
    status: "draft",
    created_at: minutesAgo(5000),
    updated_at: minutesAgo(420),
  },
];

function makeBible(novel: Novel): NovelBible {
  const isUrban = novel.id.includes("urban");
  const isFantasy = novel.id.includes("fantasy");
  return {
    novel_id: novel.id,
    title_candidates: [
      {
        title: novel.title,
        reason: isUrban
          ? "重生、商业逆转和家庭救赎都能在书名中直接露出。"
          : isFantasy
            ? "突出升级体系的陌生感和主角稀缺天赋。"
            : "情绪感强，适合现代情感线的迟来和解。"
      },
      {
        title: isUrban ? "重回十八岁，我让旧账见光" : isFantasy ? "骨相天书" : "不渡春潮",
        reason: "备用名更短，便于封面和榜单展示。",
      },
    ],
    premise: isUrban
      ? "破产中年人重回十八岁，用记忆差和商业嗅觉救回家业。"
      : isFantasy
        ? "废骨少年发现万相骨可借万物之形，一步步翻开宗门旧案。"
        : "离婚律师与旧爱重逢，在一桩遗产案里重新审视亲密关系。",
    genre: novel.genre,
    target_platform: novel.target_platform,
    target_readers: isUrban
      ? "偏好爽点明确、事业线强、亲情修复的都市读者。"
      : isFantasy
        ? "偏好升级、秘境、宗门权谋和热血反打的读者。"
        : "偏好成熟情感、职业质感和拉扯关系的读者。",
    core_selling_points: isUrban
      ? ["重生信息差", "家庭救赎", "商业反杀", "时代红利"]
      : isFantasy
        ? ["稀缺天赋", "宗门悬案", "等级突破", "秘境奇观"]
        : ["旧爱重逢", "职业对抗", "情绪疗愈", "家族秘密"],
    reader_expectations: ["开局要快", "每章有明确推进", "人物选择要有代价"],
    main_conflict: isUrban
      ? "林家账本牵出地方商会与亲族背叛，主角必须在旧时代规则中建立新秩序。"
      : isFantasy
        ? "主角被宗门定义为废骨，却逐渐发现万相骨是封印旧神的钥匙。"
        : "男女主在职业立场和未解误会之间不断靠近又互相推开。",
    protagonist_goal: isUrban ? "救回家业并建立自己的商业版图。" : "查清身世并完成骨相觉醒。",
    emotional_value: isUrban ? "失而复得、提前布局、把遗憾改写成胜利。" : "被轻视者一步步证明自身价值。",
    tone: "节奏紧、冲突显性、句子利落，关键节点保留情绪回响。",
    platform_tags: [novel.genre, "强情节", "长线伏笔"],
    world_rules: ["事实变更必须写入事实表", "伏笔回收前不能自相矛盾", "每章结尾保留下一步期待"],
    constraints: ["避免大段背景说明", "避免 Agent 输出裸 JSON 进入正文"],
    opening_strategy: {
      first_scene: isUrban ? "雨夜公交站，主角接到父亲电话。" : "宗门测骨台，主角被判废骨。",
      first_conflict: isUrban ? "账本被夺和家族背叛同时发生。" : "外门长老当众剥夺主角名额。",
      first_three_chapters_goal: "完成主角处境翻转，建立长期对手和第一条主线线索。",
    },
    platform_profile: {
      target_platform: novel.target_platform,
      opening_speed: novel.target_platform === "fanqie" ? "very_fast" : "fast",
      setup_ratio: 0.28,
      dialogue_ratio: 0.38,
      payoff_frequency: "每章至少一个显性回报",
      cliffhanger_strength: "strong",
      review_bias: {
        pacing: "优先检查开头三页是否有行动推进",
        cliffhanger: "章尾必须给出下一章明确问题",
      },
    },
  };
}

function makeWorldSetting(novel: Novel): WorldSetting {
  if (novel.id.includes("fantasy")) {
    return {
      genre_type: "玄幻",
      overview: "万相大陆以骨相定命，宗门、城邦和秘境共同维持等级秩序。",
      power_system: {
        name: "骨相九阶",
        levels: ["醒骨", "炼骨", "换骨", "铭相", "万相"],
        rules: ["骨相决定可借之形", "越阶借形会损伤神识"],
        costs: ["消耗寿元或记忆碎片"],
        limits: ["同一日不能连续借用相克之形"],
      },
      organizations: [
        {
          name: "青岚宗",
          role: "主角起点与压迫来源",
          resources: ["测骨台", "藏经阁", "外门试炼"],
          conflicts: ["长老派系争夺万相骨线索"],
        },
      ],
      locations: [
        {
          name: "坠星谷",
          description: "陨铁与妖骨交错的禁地。",
          story_use: "第一卷秘境和身世线索所在地。",
        },
      ],
      taboos: ["禁止私刻他人骨纹"],
      hard_rules: ["骨相伤势无法用普通丹药恢复"],
    };
  }

  return {
    genre_type: novel.id.includes("urban") ? "都市" : "其他",
    overview: "现实城市与行业变化共同构成主角选择的压力场。",
    power_system: {
      name: "信息差与现金流",
      levels: ["线索", "证据", "资源", "渠道", "资本"],
      rules: ["所有商业行动必须有信息来源和执行成本"],
      costs: ["信任透支", "现金压力", "法律风险"],
      limits: ["主角不能凭空获得未经历过的细节"],
    },
    organizations: [
      {
        name: "城南机械厂",
        role: "家族危机和第一阶段主战场",
        resources: ["账本", "厂房", "老员工"],
        conflicts: ["亲族转让、外部债务、行业下行"],
      },
    ],
    locations: [
      {
        name: "城南旧街",
        description: "十年前仍未改造的老城区。",
        story_use: "重生开场、家庭线和时代红利的交汇点。",
      },
    ],
    taboos: ["不能用超自然解释商业成功"],
    hard_rules: ["所有时代信息必须符合 2008 年背景"],
  };
}

function makeCharacters(novel: Novel): CharacterCard[] {
  const protagonist = novel.id.includes("urban") ? "林砚" : novel.id.includes("fantasy") ? "沈照" : "许知潮";
  return [
    {
      id: makeId("char", `${novel.id}-protagonist`),
      novel_id: novel.id,
      id_hint: "protagonist",
      name: protagonist,
      role: "protagonist",
      identity: novel.id.includes("urban") ? "重生回十八岁的创业者" : "被低估的核心主角",
      personality: ["克制", "行动快", "记仇但有底线"],
      desire: "扭转当前困境，拿回主动权。",
      motivation: "不再让上一轮遗憾重演。",
      secret: "掌握一段未来或身世相关的关键信息。",
      abilities: ["复盘能力强", "能在压力下快速决策"],
      limitations: ["资源少", "不能暴露全部底牌"],
      current_state: "刚进入第一卷主冲突。",
      relationship_map: [
        { target: "顾南枝", relationship: "盟友", tension: "信任尚未建立" },
        { target: "周启明", relationship: "对手", tension: "旧账与现实利益冲突" },
      ],
      arc: {
        start: "带着不甘进入局面。",
        turning_points: ["第 3 章拿到第一份证据", "第 12 章第一次公开反击", "第 24 章付出关键代价"],
        expected_end: "完成第一阶段逆转，但引出更大的对手。",
      },
      first_appearance_chapter: 1,
      chapter_1_to_30_plan: ["建立目标", "获得盟友", "第一次反击", "暴露更深敌人"],
    },
    {
      id: makeId("char", `${novel.id}-ally`),
      novel_id: novel.id,
      id_hint: "ally",
      name: "顾南枝",
      role: "ally",
      identity: "拥有关键资源的潜在同盟",
      personality: ["敏锐", "现实", "不轻易站队"],
      desire: "确认主角是否值得下注。",
      motivation: "借主角打破自己受限的处境。",
      secret: "她掌握另一份未公开证据。",
      abilities: ["人脉", "谈判", "信息整理"],
      limitations: ["家族压力", "不能公开出面"],
      current_state: "旁观主角第一步行动。",
      relationship_map: [{ target: protagonist, relationship: "观察者", tension: "互相试探" }],
      arc: {
        start: "不信任主角。",
        turning_points: ["第 6 章提供线索", "第 18 章共同承担风险"],
        expected_end: "成为第一卷可靠盟友。",
      },
      first_appearance_chapter: 2,
      chapter_1_to_30_plan: ["制造误会", "交换线索", "共同设局"],
    },
    {
      id: makeId("char", `${novel.id}-antagonist`),
      novel_id: novel.id,
      id_hint: "antagonist",
      name: "周启明",
      role: "antagonist",
      identity: "第一卷明面阻力",
      personality: ["精明", "强势", "善于借规则压人"],
      desire: "在主角反应过来前完成资源吞并。",
      motivation: "害怕旧账暴露并失去靠山。",
      secret: "他不是幕后主使，只是执行人。",
      abilities: ["资金", "灰色关系", "舆论操控"],
      limitations: ["怕公开调查", "过度自信"],
      current_state: "正在推动第一场危机。",
      relationship_map: [{ target: protagonist, relationship: "敌对", tension: "主角掌握他的破绽" }],
      arc: {
        start: "占据优势。",
        turning_points: ["第 8 章被迫退让", "第 20 章反扑"],
        expected_end: "败退后供出更高层线索。",
      },
      first_appearance_chapter: 1,
      chapter_1_to_30_plan: ["施压", "试探", "反扑", "暴露幕后"],
    },
  ];
}

function makeOutlines(novel: Novel): ChapterOutline[] {
  return Array.from({ length: 30 }, (_, index) => {
    const chapterIndex = index + 1;
    return {
      novel_id: novel.id,
      volume_index: 1,
      chapter_index: chapterIndex,
      title: chapterIndex === 1 ? "旧账重开" : `第${chapterIndex}章 局面推进`,
      pov: "第三人称限知",
      goal: chapterIndex === 1 ? "让主角确认重生并启动救家行动。" : "推进主线证据与人物关系。",
      conflict: chapterIndex % 3 === 0 ? "盟友立场动摇，主角必须付出交换条件。" : "外部压力逼近，主角需要用有限资源反制。",
      key_events: [
        `第 ${chapterIndex} 章出现新的行动目标。`,
        "主角通过对话或行动验证一条事实。",
        "章尾抛出下一步风险。",
      ],
      character_changes: ["主角获得更明确的局面判断", "对手开始调整策略"],
      new_facts: [
        {
          subject: "主线证据",
          predicate: "推进到",
          object: `第 ${chapterIndex} 章节点`,
          importance: chapterIndex <= 3 ? 5 : 3,
        },
      ],
      payoff: chapterIndex % 5 === 0 ? "阶段性反击成功。" : "获得一条可执行线索。",
      foreshadowing: [`第 ${chapterIndex} 章留下的未解释细节`],
      cliffhanger: chapterIndex === 1 ? "记者回拨电话，账本即将现身。" : "下一场冲突的关键人物主动出现。",
      estimated_word_count: 2600,
    };
  });
}

function makeChapters(novel: Novel): Chapter[] {
  return makeOutlines(novel).map((outline) => {
    const isFirstUrban = novel.id === "novel-urban-rebirth" && outline.chapter_index === 1;
    const hasDraft = isFirstUrban || outline.chapter_index <= 2;
    const content = isFirstUrban
      ? chapterOneContent
      : hasDraft
        ? `# ${outline.title}\n\n${outline.goal}\n\n${outline.conflict}\n\n本章 mock 正文用于 Web 工作台联调。`
        : null;
    return {
      id: makeId("chapter", `${novel.id}-${outline.chapter_index}`),
      novel_id: novel.id,
      volume_index: outline.volume_index,
      chapter_index: outline.chapter_index,
      title: outline.title,
      outline: outline.goal,
      content,
      summary: hasDraft ? `${outline.title}完成了${outline.goal}` : null,
      status: isFirstUrban ? "reviewed" : hasDraft ? "drafted" : "outlined",
      score: isFirstUrban ? 78 : hasDraft ? 72 : null,
      word_count: content ? countWords(content) : 0,
      version: isFirstUrban ? 2 : hasDraft ? 1 : 0,
      created_at: minutesAgo(8500 - outline.chapter_index * 18),
      updated_at: minutesAgo(60 - Math.min(outline.chapter_index, 20)),
    };
  });
}

function makeFacts(novel: Novel, chapters: Chapter[]): Fact[] {
  return [
    {
      id: makeId("fact", `${novel.id}-1`),
      novel_id: novel.id,
      chapter_id: chapters[0]?.id,
      subject: "主角",
      predicate: "长期目标",
      object: novel.id.includes("urban") ? "救回家业" : "查清身世",
      importance: 5,
      created_at: minutesAgo(75),
    },
    {
      id: makeId("fact", `${novel.id}-2`),
      novel_id: novel.id,
      chapter_id: null,
      subject: "第一卷对手",
      predicate: "弱点",
      object: "害怕公开调查",
      importance: 4,
      created_at: minutesAgo(72),
    },
    {
      id: makeId("fact", `${novel.id}-3`),
      novel_id: novel.id,
      chapter_id: chapters[0]?.id,
      subject: "章尾伏笔",
      predicate: "状态",
      object: "已埋下，待第 3 章推进",
      importance: 3,
      created_at: minutesAgo(70),
    },
  ];
}

function makeVersions(chapter: Chapter): ChapterVersion[] {
  if (!chapter.content) {
    return [];
  }
  return [
    {
      id: makeId("version", `${chapter.id}-1`),
      chapter_id: chapter.id,
      version: 1,
      title: chapter.title,
      content: chapter.content,
      summary: chapter.summary ?? "",
      word_count: chapter.word_count,
      data: {
        source: "writer",
        score: 72,
        notes: "Writer -> Continuity -> Style 初稿。",
      },
      created_at: minutesAgo(58),
    },
    {
      id: makeId("version", `${chapter.id}-2`),
      chapter_id: chapter.id,
      version: 2,
      title: chapter.title,
      content: rewriteContent,
      summary: "强化了章尾钩子，让主角行动更主动。",
      word_count: countWords(rewriteContent),
      data: {
        source: "rewrite",
        score: 78,
        notes: "根据 Reviewer 建议补强行动和悬念。",
      },
      created_at: minutesAgo(32),
    },
  ];
}

function makeReview(chapter: Chapter): ReviewReport {
  return {
    id: makeId("review", chapter.id),
    chapter_id: chapter.id,
    total_score: 78,
    passed: true,
    scores: {
      opening_hook_score: 8,
      pacing_score: 8,
      payoff_score: 7,
      character_score: 8,
      dialogue_score: 7,
      continuity_score: 9,
      cliffhanger_score: 8,
      platform_fit_score: 8,
    },
    strengths: ["开场行动明确", "主角目标清晰", "章尾继续推进主线"],
    issues: [
      {
        severity: "medium",
        dimension: "payoff",
        location: "中段",
        description: "电视台热线的压力效果可以更具体，增强读者即时回报。",
      },
      {
        severity: "low",
        dimension: "dialogue",
        location: "父亲电话",
        description: "父亲的反应还可以增加一处情绪犹疑。",
      },
    ],
    suggestions: ["补一处外部反馈，让反击结果更可见。", "保留雨夜和电话意象，作为第一卷情绪锚点。"],
    rewrite_instruction: {
      needed: true,
      rewrite_type: "partial",
      priority: "medium",
      goals: ["强化主角主动布局", "让章尾钩子更直接"],
      preserve: ["公交站雨夜开场", "父亲电话", "电视台热线"],
      change: ["增加记者回拨和账本即将现身的信息"],
      avoid: ["不要用大段解释替代行动"],
    },
    created_at: minutesAgo(30),
  };
}

function makeAgentRuns(): AgentRun[] {
  return [
    {
      id: "run-001",
      novel_id: "novel-urban-rebirth",
      role: "reviewer",
      task: "review_chapter",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 940,
      prompt_tokens: 920,
      completion_tokens: 420,
      total_tokens: 1340,
      prompt_cost_micro_usd: 920,
      completion_cost_micro_usd: 840,
      total_cost_micro_usd: 1760,
      output_summary: "第 1 章评分 78，通过，建议局部重写章尾。",
      structured: { review_report: { total_score: 78 }, _engineering: { duration_ms: 940 } },
      raw_text: "",
      raw_notes: "mock reviewer report",
      parse_error: null,
      created_at: minutesAgo(30),
    },
    {
      id: "run-002",
      novel_id: "novel-urban-rebirth",
      role: "style",
      task: "polish_style",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 520,
      prompt_tokens: 760,
      completion_tokens: 380,
      total_tokens: 1140,
      prompt_cost_micro_usd: 760,
      completion_cost_micro_usd: 760,
      total_cost_micro_usd: 1520,
      output_summary: "完成第 1 章风格润色，保留紧凑叙事。",
      structured: { styled_chapter: { title: "旧账重开" }, _engineering: { duration_ms: 520 } },
      raw_text: "",
      raw_notes: "mock style output",
      parse_error: null,
      created_at: minutesAgo(34),
    },
    {
      id: "run-003",
      novel_id: "novel-urban-rebirth",
      role: "continuity",
      task: "check_continuity",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 480,
      prompt_tokens: 680,
      completion_tokens: 260,
      total_tokens: 940,
      prompt_cost_micro_usd: 680,
      completion_cost_micro_usd: 520,
      total_cost_micro_usd: 1200,
      output_summary: "连续性通过，新增 3 条事实。",
      structured: { continuity_report: { passed: true }, _engineering: { duration_ms: 480 } },
      raw_text: "",
      raw_notes: "mock continuity output",
      parse_error: null,
      created_at: minutesAgo(36),
    },
    {
      id: "run-004",
      novel_id: "novel-fantasy-upgrade",
      role: "writer",
      task: "generate_chapter",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 1100,
      prompt_tokens: 1050,
      completion_tokens: 1450,
      total_tokens: 2500,
      prompt_cost_micro_usd: 1050,
      completion_cost_micro_usd: 2900,
      total_cost_micro_usd: 3950,
      output_summary: "生成第 2 章初稿，约 112 字 mock 正文。",
      structured: { chapter_draft: { chapter_index: 2 }, _engineering: { duration_ms: 1100 } },
      raw_text: "",
      raw_notes: "mock writer output",
      parse_error: null,
      created_at: minutesAgo(96),
    },
    {
      id: "run-005",
      novel_id: "novel-romance-comeback",
      role: "plot",
      task: "generate_outline",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 1360,
      prompt_tokens: 1320,
      completion_tokens: 2120,
      total_tokens: 3440,
      prompt_cost_micro_usd: 1320,
      completion_cost_micro_usd: 4240,
      total_cost_micro_usd: 5560,
      output_summary: "生成 30 章大纲，等待章节写作。",
      structured: { plot_plan: { chapters: 30 }, _engineering: { duration_ms: 1360 } },
      raw_text: "",
      raw_notes: "mock plot output",
      parse_error: null,
      created_at: minutesAgo(420),
    },
  ];
}

function makeApiJobs(): ApiJob[] {
  return [
    {
      id: "job-003",
      kind: "rewrite_chapter",
      status: "running",
      novel_id: "novel-urban-rebirth",
      chapter_index: 1,
      source_job_id: null,
      progress_current: 0,
      progress_total: 1,
      payload: {
        novel_id: "novel-urban-rebirth",
        chapter_index: 1,
      },
      result: null,
      error: null,
      created_at: minutesAgo(5),
      updated_at: minutesAgo(2),
    },
    {
      id: "job-002",
      kind: "write_chapter",
      status: "failed",
      novel_id: "novel-urban-rebirth",
      chapter_index: 2,
      source_job_id: null,
      progress_current: 0,
      progress_total: 1,
      payload: {
        novel_id: "novel-urban-rebirth",
        chapter_index: 2,
      },
      result: null,
      error: "模型输出缺少 chapter_draft envelope，已保留 payload 可重试。",
      created_at: minutesAgo(44),
      updated_at: minutesAgo(41),
    },
    {
      id: "job-001",
      kind: "review_chapter",
      status: "succeeded",
      novel_id: "novel-urban-rebirth",
      chapter_index: 1,
      source_job_id: null,
      progress_current: 1,
      progress_total: 1,
      payload: {
        novel_id: "novel-urban-rebirth",
        chapter_index: 1,
      },
      result: {
        report: {
          total_score: 78,
          passed: true,
        },
      },
      error: null,
      created_at: minutesAgo(63),
      updated_at: minutesAgo(62),
    },
  ];
}

export interface MockDatabase {
  novels: Novel[];
  bibles: Record<string, NovelBible>;
  characters: Record<string, CharacterCard[]>;
  worldSettings: Record<string, WorldSetting>;
  outlines: Record<string, ChapterOutline[]>;
  chapters: Record<string, Chapter[]>;
  facts: Record<string, Fact[]>;
  versions: Record<string, ChapterVersion[]>;
  reviews: Record<string, ReviewReport>;
  agentRuns: AgentRun[];
  jobs: ApiJob[];
}

export function createMockDatabase(): MockDatabase {
  const bibles: MockDatabase["bibles"] = {};
  const characters: MockDatabase["characters"] = {};
  const worldSettings: MockDatabase["worldSettings"] = {};
  const outlines: MockDatabase["outlines"] = {};
  const chapters: MockDatabase["chapters"] = {};
  const facts: MockDatabase["facts"] = {};
  const versions: MockDatabase["versions"] = {};
  const reviews: MockDatabase["reviews"] = {};

  initialNovels.forEach((novel) => {
    const chapterList = makeChapters(novel);
    bibles[novel.id] = makeBible(novel);
    characters[novel.id] = makeCharacters(novel);
    worldSettings[novel.id] = makeWorldSetting(novel);
    outlines[novel.id] = makeOutlines(novel);
    chapters[novel.id] = chapterList;
    facts[novel.id] = makeFacts(novel, chapterList);

    chapterList.forEach((chapter) => {
      versions[chapter.id] = novel.id === "novel-urban-rebirth" && chapter.chapter_index === 1 ? makeVersions(chapter) : [];
    });

    const firstChapter = chapterList[0];
    if (novel.id === "novel-urban-rebirth" && firstChapter) {
      reviews[firstChapter.id] = makeReview(firstChapter);
      firstChapter.content = rewriteContent;
      firstChapter.word_count = countWords(rewriteContent);
      firstChapter.summary = "林砚在雨夜重回十八岁，抢先打乱账本转移计划。";
    }
  });

  return {
    novels: [...initialNovels],
    bibles,
    characters,
    worldSettings,
    outlines,
    chapters,
    facts,
    versions,
    reviews,
    agentRuns: makeAgentRuns(),
    jobs: makeApiJobs(),
  };
}

export function makeNewNovel(input: {
  idea: string;
  genre: string;
  target_platform: TargetPlatform;
}): {
  novel: Novel;
  bible: NovelBible;
  characters: CharacterCard[];
  worldSetting: WorldSetting;
  outlines: ChapterOutline[];
  chapters: Chapter[];
  facts: Fact[];
} {
  const id = `novel-${crypto.randomUUID()}`;
  const title = input.idea.trim().slice(0, 18) || "未命名新书";
  const novel: Novel = {
    id,
    title,
    genre: input.genre,
    target_platform: input.target_platform,
    status: "draft",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  const bible = makeBible(novel);
  bible.premise = input.idea.trim() || bible.premise;
  return {
    novel,
    bible,
    characters: makeCharacters(novel),
    worldSetting: makeWorldSetting(novel),
    outlines: makeOutlines(novel),
    chapters: makeChapters(novel).map((chapter) => ({ ...chapter, content: null, summary: null, status: "outlined", score: null, word_count: 0, version: 0 })),
    facts: makeFacts(novel, []),
  };
}

export function makeGeneratedChapter(chapter: Chapter): string {
  return `# ${chapter.title}

${chapter.outline}

主角在压力下先确认目标，再用一个可执行的小动作撬动局面。对手没有立刻失败，但被迫暴露新的破绽。

一通电话在章尾打进来，带来下一场冲突的时间和地点。

“现在轮到我们去见他了。”`;
}

export function makeRuntimeAgentRun(input: Pick<AgentRun, "novel_id" | "role" | "task" | "output_summary">): AgentRun {
  const promptTokens = 580 + Math.floor(Math.random() * 640);
  const completionTokens = 260 + Math.floor(Math.random() * 960);
  return {
    id: `run-${crypto.randomUUID()}`,
    novel_id: input.novel_id,
    role: input.role,
    task: input.task,
    provider: "smoke",
    model: "smoke",
    status: "ok",
    duration_ms: 600 + Math.floor(Math.random() * 700),
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
    total_tokens: promptTokens + completionTokens,
    prompt_cost_micro_usd: promptTokens,
    completion_cost_micro_usd: completionTokens * 2,
    total_cost_micro_usd: promptTokens + completionTokens * 2,
    output_summary: input.output_summary,
    structured: { _engineering: { will_fallback: false } },
    raw_text: "",
    raw_notes: "mock runtime operation",
    parse_error: null,
    created_at: new Date().toISOString(),
  };
}

export function makeRetriedJob(source: ApiJob): ApiJob {
  const now = new Date().toISOString();
  return {
    ...source,
    id: `job-${crypto.randomUUID()}`,
    status: "queued",
    source_job_id: source.id,
    progress_current: 0,
    progress_total: source.progress_total ?? 1,
    result: null,
    error: null,
    created_at: now,
    updated_at: now,
  };
}
