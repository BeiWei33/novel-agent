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
const DEMO_NOVEL_ID = "urban_rebirth_fanqie_demo";

function minutesAgo(minutes: number): string {
  return new Date(baseTime - minutes * 60_000).toISOString();
}

function makeId(prefix: string, value: string | number): string {
  return `${prefix}-${value}`;
}

const chapterOneContent = `# 第1章 暴雨回站

林舟睁开眼时，雨声正砸在外卖站的铁皮棚上。

棚顶漏水，一滴一滴落进装着头盔的塑料筐。墙上那块老旧电子钟闪了两下，时间停在晚上九点四十七分，日期是 2016 年 7 月 18 日。

他盯着那串数字，手指慢慢攥紧。

十年前。

也是周启明把暴雨事故甩到他头上的那个夜晚。

“林舟，你还愣着干什么？”站长周启明一把推开办公室门，手里攥着一叠签收单，“城南那条线没人跑，你带两个人过去。出了问题，单子算你头上。”

屋里十几个骑手都看了过来。有人低头擦雨衣，有人假装没听见。暴雨把城区分成几块孤岛，城南高架底下积水最深，上一世就是那条线，周启明硬派单，后来一个新人骑手撞上逆行货车，赔偿、罚款、扣薪全压到林舟身上。

他那时只知道争，争到最后还是输。

这一世不一样。

林舟拿过桌上的调度板，指尖在三条线路上划过：“城南不走。医院单改走新桥，学校单合并给老刘，写字楼那四单等十分钟。”

周启明脸色一沉：“你说不走就不走？平台罚款谁出？”

“我出。”林舟抬头，“但如果城南出事故，赔偿你出，还是我出？”

办公室安静了一瞬。

周启明当然不会接这个话。他只想找人签字，把风险推出去。

林舟把调度板翻过来，在背面写下三行字：积水点、改派路线、预计超时金额。未来十年的外卖系统怎么变，他记得清楚，但现在不是讲趋势的时候。现在要让这间屋里的人看见，哪条路会死人，哪条路只会罚钱。

“老刘，你跑医院三单，多绕七分钟，但不会过高架。”林舟把钥匙推给旁边的老骑手，“小赵，你别去城南，去学校门口等两单并单。许店长那边我打电话，她能帮我们把便利店单压十五分钟。”

“你认识许蔓？”有人问。

林舟没有解释。

他拨通便利店座机，雨声从话筒那头挤进来。许蔓的声音很冷：“你们站点又想拖单？”

“不是拖单，是保单。”林舟说，“你店里现在有八单去医院家属楼，如果按原路线，全超时。你给我十五分钟，我保证先送退烧药和粥，其他赔付我跟站里谈。”

许蔓沉默两秒：“你是谁？”

“林舟。一个不想再背锅的人。”

电话挂断后，周启明冷笑：“你拿什么保证？”

“拿监控、签收记录和这张调度板。”林舟把手机录音打开，放到桌上，“今晚每一步都留记录。谁派的单，谁改的线，谁让新人去城南，都说清楚。”

周启明的笑僵住了。

第一波骑手冲进雨里时，林舟的手机震了一下。许蔓发来一条短信：十五分钟，超过我照样投诉。

同一时间，站点门口停下一辆黑色轿车。车窗降下半截，陈岳坐在里面，看着调度板上的路线，眼神像是盯住了一块刚露头的矿。

林舟知道这个眼神。

前世，就是陈岳把本地生活试点包装成资本故事，踩着无数小站点起家。

而现在，对方比上一世提前看见了他。

短信又响了一声。许蔓追加了一句：如果你真能把医院单救回来，明天早上七点，带合同来店里谈。

林舟看着窗外暴雨。

第一单还没送完，下一步压力已经来了。他必须在天亮前拿到第一份真实订单，否则今晚所有改线、罚款和得罪周启明的代价，都会重新压回他身上。`;

const rewriteContent = `${chapterOneContent}

十分钟后，医院家属楼门口的取餐架终于亮起第一条签收记录。

林舟没有让老刘立刻回站。他让对方把药袋和粥放在监控正下方，又让小赵补拍学校门口积水线。所有证据连成一串，周启明想甩锅，就必须先解释为什么原路线会把新人送进城南高架。

许蔓的第二条短信很短：明早七点，别迟到。

林舟把手机扣在调度板上，抬头看向门口那辆还没离开的黑色轿车。

真正的第一单，不是送到医院的那袋药。

是他必须在天亮前，把许蔓的便利店变成第一个愿意签字的商家。`;

const initialNovels: Novel[] = [
  {
    id: DEMO_NOVEL_ID,
    title: "重回外卖站",
    genre: "都市重生商业文",
    target_platform: "fanqie",
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
          ? "书名直接落在外卖站场景，重生和商业逆袭的识别成本低。"
          : isFantasy
            ? "突出升级体系的陌生感和主角稀缺天赋。"
            : "情绪感强，适合现代情感线的迟来和解。"
      },
      {
        title: isUrban ? "暴雨外卖站" : isFantasy ? "骨相天书" : "不渡春潮",
        reason: "备用名更短，便于封面和榜单展示。",
      },
    ],
    premise: isUrban
      ? "林舟回到十年前的暴雨夜，必须在外卖站甩锅、事故倒计时和对手截胡前，用未来经验拿回第一单主动权。"
      : isFantasy
        ? "废骨少年发现万相骨可借万物之形，一步步翻开宗门旧案。"
        : "离婚律师与旧爱重逢，在一桩遗产案里重新审视亲密关系。",
    genre: novel.genre,
    target_platform: novel.target_platform,
    target_readers: isUrban
      ? "偏好都市重生、底层逆袭、事业线明确和每章强推进的番茄读者。"
      : isFantasy
        ? "偏好升级、秘境、宗门权谋和热血反打的读者。"
        : "偏好成熟情感、职业质感和拉扯关系的读者。",
    core_selling_points: isUrban
      ? ["底层外卖站切入，现实压力强", "未来行业节点转化为当场决策", "商业机会先表现为阻力和代价", "章尾给出下一章具体订单压力"]
      : isFantasy
        ? ["稀缺天赋", "宗门悬案", "等级突破", "秘境奇观"]
        : ["旧爱重逢", "职业对抗", "情绪疗愈", "家族秘密"],
    reader_expectations: isUrban
      ? ["前 800 字进入危机", "主角每章都用行动换主动权", "商业机会必须有阻力、成本和下一步压力"]
      : ["开局要快", "每章有明确推进", "人物选择要有代价"],
    main_conflict: isUrban
      ? "林舟既要避开周启明的事故甩锅，又要赶在陈岳截胡前把许蔓便利店变成第一份真实订单。"
      : isFantasy
        ? "主角被宗门定义为废骨，却逐渐发现万相骨是封印旧神的钥匙。"
        : "男女主在职业立场和未解误会之间不断靠近又互相推开。",
    protagonist_goal: isUrban ? "避开暴雨事故，留下责任证据，拿到许蔓便利店的第一份订单。" : "查清身世并完成骨相觉醒。",
    emotional_value: isUrban ? "被迫背责的人重新站上调度位，用一次次可验证的小胜利改写命运。" : "被轻视者一步步证明自身价值。",
    tone: "节奏紧、冲突显性、句子利落，关键节点保留情绪回响。",
    platform_tags: isUrban ? [novel.genre, "都市重生", "外卖站", "商业逆袭"] : [novel.genre, "强情节", "长线伏笔"],
    world_rules: ["事实变更必须写入事实表", "伏笔回收前不能自相矛盾", "每章结尾保留下一步期待"],
    constraints: ["避免大段背景说明", "避免 Agent 输出裸 JSON 进入正文"],
    opening_strategy: {
      first_scene: isUrban ? "暴雨外卖站，林舟回到事故甩锅夜。" : "宗门测骨台，主角被判废骨。",
      first_conflict: isUrban ? "周启明要求林舟接城南高风险路线并背责。" : "外门长老当众剥夺主角名额。",
      first_three_chapters_goal: isUrban
        ? "避开事故，拿到许蔓便利店第一份真实订单，引出陈岳截胡。"
        : "完成主角处境翻转，建立长期对手和第一条主线线索。",
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

  if (novel.id.includes("urban")) {
    return {
      genre_type: "都市",
      overview: "2016 年本地生活和即时配送刚进入片区抢位期，暴雨、订单、站点责任和商家信任共同构成主角压力场。",
      power_system: {
        name: "调度经验与本地生活订单",
        levels: ["路线", "证据", "保单", "商家", "片区", "平台入口"],
        rules: ["所有商业判断必须落到当场调度、合同或现金流", "未来经验只能转化为可验证的现实动作"],
        costs: ["罚款", "骑手信任", "商家投诉", "被竞争对手提前注意"],
        limits: ["林舟不能凭空拿到未接触商家的信任", "每次改线都必须承担超时或赔付压力"],
      },
      organizations: [
        {
          name: "暴雨外卖站",
          role: "开场压力源和第一阶段调度主战场",
          resources: ["骑手", "调度板", "签收记录", "站点监控"],
          conflicts: ["周启明试图把城南事故责任转嫁给骑手"],
        },
        {
          name: "许蔓便利店",
          role: "第一份真实订单的潜在来源",
          resources: ["医院家属楼订单", "商家投诉权", "片区熟客"],
          conflicts: ["许蔓只给十五分钟验证窗口，不接受空口承诺"],
        },
        {
          name: "陈岳资本团队",
          role: "提前发现本地生活试点价值的外部竞争者",
          resources: ["资金", "包装能力", "商家拜访团队"],
          conflicts: ["复制林舟方案并截胡签约商家"],
        },
      ],
      locations: [
        {
          name: "暴雨外卖站",
          description: "铁皮棚漏水、调度板混乱，所有甩锅和反制都从这里开始。",
          story_use: "重生确认、调度反击、责任证据留存。",
        },
        {
          name: "医院家属楼",
          description: "药和粥必须先送到的高需求片区。",
          story_use: "验证保单方案，形成第一份即时回报。",
        },
        {
          name: "城南高架",
          description: "暴雨积水最深、上一世事故发生的高风险路线。",
          story_use: "持续提醒读者事故倒计时和周启明甩锅风险。",
        },
      ],
      taboos: ["不能用超自然解释商业成功", "不能让合作无阻力谈成"],
      hard_rules: ["时代信息必须符合 2016 年本地生活行业背景", "事实变更必须写入事实表"],
    };
  }

  return {
    genre_type: "其他",
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
        name: "律所合伙人会议",
        role: "现代情感线中的职业压力来源",
        resources: ["客户", "案件资料", "谈判时间"],
        conflicts: ["职业立场与旧日关系互相牵制"],
      },
    ],
    locations: [
      {
        name: "旧城区法援中心",
        description: "现实案件与私人关系交错的公共空间。",
        story_use: "现代情感线的案件推进和人物重逢地点。",
      },
    ],
    taboos: ["不能用超自然解释商业成功"],
    hard_rules: ["人物选择必须受到职业伦理和现实成本约束"],
  };
}

function makeCharacters(novel: Novel): CharacterCard[] {
  if (novel.id.includes("urban")) {
    return [
      {
        id: makeId("char", `${novel.id}-protagonist`),
        novel_id: novel.id,
        id_hint: "protagonist",
        name: "林舟",
        role: "protagonist",
        identity: "重回十年前暴雨夜的外卖站骑手",
        personality: ["克制", "行动快", "敢担责", "对风险敏感"],
        desire: "避开城南事故，拿到第一份真实订单。",
        motivation: "不再替周启明背锅，也不让上一世的事故重演。",
        secret: "记得十年后的本地生活行业节点和暴雨事故细节。",
        abilities: ["调度复盘", "成本判断", "证据留存", "压力下决策"],
        limitations: ["现金少", "站内话语权弱", "不能暴露重生底牌"],
        current_state: "刚回到 2016 年 7 月 18 日暴雨夜，正在重排路线。",
        relationship_map: [
          { target: "许蔓", relationship: "潜在盟友", tension: "必须用十五分钟保单结果换信任" },
          { target: "周启明", relationship: "直接对手", tension: "周启明急于把城南事故责任推出去" },
          { target: "陈岳", relationship: "未来竞争者", tension: "陈岳提前注意到林舟的调度方案" },
        ],
        arc: {
          start: "被站点甩锅的骑手。",
          turning_points: ["第 1 章留下调度证据", "第 3 章争取许蔓合作", "第 12 章建立片区配送闭环"],
          expected_end: "从被动背责者变成能组织片区订单的人。",
        },
        first_appearance_chapter: 1,
        chapter_1_to_30_plan: ["避开事故", "拿下便利店订单", "建立片区试点", "面对陈岳截胡"],
      },
      {
        id: makeId("char", `${novel.id}-ally`),
        novel_id: novel.id,
        id_hint: "ally",
        name: "许蔓",
        role: "ally",
        identity: "便利店店长 / 潜在运营盟友",
        personality: ["现实", "敏锐", "边界清楚", "重视兑现"],
        desire: "确认林舟是否能稳定保住门店订单。",
        motivation: "不再被外卖站拖单和投诉成本牵着走。",
        secret: "她手里有医院家属楼一批稳定高频订单。",
        abilities: ["商家资源", "投诉议价", "门店运营", "判断人是否靠谱"],
        limitations: ["不信任站点承诺", "不能承担持续延误损失"],
        current_state: "给林舟十五分钟保单验证窗口。",
        relationship_map: [{ target: "林舟", relationship: "观察中的合作方", tension: "只看结果，不听空话" }],
        arc: {
          start: "对外卖站不信任。",
          turning_points: ["第 1 章给出十五分钟窗口", "第 3 章夜谈合作边界", "第 10 章参与片区试点"],
          expected_end: "成为第一卷核心商家盟友。",
        },
        first_appearance_chapter: 1,
        chapter_1_to_30_plan: ["提出条件", "验证保单", "签下第一单", "共同抗住截胡"],
      },
      {
        id: makeId("char", `${novel.id}-antagonist`),
        novel_id: novel.id,
        id_hint: "antagonist",
        name: "周启明",
        role: "antagonist",
        identity: "外卖站承包商",
        personality: ["短视", "强势", "善于甩锅", "怕承担公开责任"],
        desire: "让林舟接下城南高风险路线并签字背责。",
        motivation: "保住站点罚款和事故赔偿之外的自身利益。",
        secret: "他知道城南路线风险更高，却仍想把责任推出去。",
        abilities: ["站点排班权", "罚款规则", "骑手考勤", "现场施压"],
        limitations: ["怕录音和监控", "不懂本地生活长期价值"],
        current_state: "被林舟要求公开说明派单责任。",
        relationship_map: [{ target: "林舟", relationship: "压迫者", tension: "林舟开始把每一步留证据" }],
        arc: {
          start: "站内占据绝对话语权。",
          turning_points: ["第 1 章甩锅失败", "第 8 章试图扣薪反扑", "第 18 章被商家和骑手共同施压"],
          expected_end: "明面败退，并暴露更大的外部竞争压力。",
        },
        first_appearance_chapter: 1,
        chapter_1_to_30_plan: ["逼单", "扣罚", "反扑", "被证据链压制"],
      },
      {
        id: makeId("char", `${novel.id}-competitor`),
        novel_id: novel.id,
        id_hint: "competitor",
        name: "陈岳",
        role: "antagonist",
        identity: "未来本地生活试点竞争者",
        personality: ["冷静", "嗅觉敏锐", "善于包装", "行动隐蔽"],
        desire: "抢在林舟前把片区配送方案包装成资本故事。",
        motivation: "提前占住本地生活入口的商业价值。",
        secret: "他已经在接触一批愿意尝试新配送方案的商家。",
        abilities: ["资金", "资源整合", "商业包装", "快速复制"],
        limitations: ["不懂底层站点真实成本", "需要可展示的样板订单"],
        current_state: "在站点门口看见林舟的调度板。",
        relationship_map: [
          { target: "林舟", relationship: "竞争者", tension: "林舟掌握现场经验，陈岳掌握外部资源" },
          { target: "许蔓", relationship: "潜在截胡目标", tension: "他会试图绕过林舟直接谈商家" },
        ],
        arc: {
          start: "旁观者和潜在投资人姿态。",
          turning_points: ["第 3 章开始截胡", "第 15 章复制片区试点", "第 28 章逼林舟升级合同和资金"],
          expected_end: "成为推动主角升级本地生活入口的长期竞争者。",
        },
        first_appearance_chapter: 1,
        chapter_1_to_30_plan: ["提前注意", "截胡商家", "复制方案", "引出平台巨头线"],
      },
    ];
  }

  const protagonist = novel.id.includes("fantasy") ? "沈照" : "许知潮";
  const allyName = novel.id.includes("fantasy") ? "闻霜" : "陆时晏";
  const antagonistName = novel.id.includes("fantasy") ? "薛长老" : "沈临川";
  return [
    {
      id: makeId("char", `${novel.id}-protagonist`),
      novel_id: novel.id,
      id_hint: "protagonist",
      name: protagonist,
      role: "protagonist",
      identity: "被低估的核心主角",
      personality: ["克制", "行动快", "记仇但有底线"],
      desire: "扭转当前困境，拿回主动权。",
      motivation: "不再让上一轮遗憾重演。",
      secret: "掌握一段未来或身世相关的关键信息。",
      abilities: ["复盘能力强", "能在压力下快速决策"],
      limitations: ["资源少", "不能暴露全部底牌"],
      current_state: "刚进入第一卷主冲突。",
      relationship_map: [
        { target: allyName, relationship: "盟友", tension: "信任尚未建立" },
        { target: antagonistName, relationship: "对手", tension: "现实利益和关键秘密冲突" },
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
      name: allyName,
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
      name: antagonistName,
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
  if (novel.id.includes("urban")) {
    type UrbanOutlineSeed = {
      title: string;
      goal: string;
      conflict: string;
      payoff: string;
      cliffhanger: string;
    };
    const earlyChapters: Record<number, UrbanOutlineSeed> = {
      1: {
        title: "暴雨回站",
        goal: "避开事故，拿回责任主动权。",
        conflict: "周启明逼林舟接城南高风险路线并签字背责。",
        payoff: "林舟重排路线并留下录音、监控和调度板证据。",
        cliffhanger: "许蔓要求明早七点带合同来谈，陈岳已经提前注意到调度方案。",
      },
      2: {
        title: "第一份真实订单",
        goal: "用调度方案证明能力，争取许蔓便利店的第一份真实订单。",
        conflict: "罚款、油钱和骑手质疑同时压来，林舟必须先拿出可见结果。",
        payoff: "医院家属楼订单准时送达，许蔓开始愿意谈合作。",
        cliffhanger: "陈岳的人也出现在便利店门口，提出更高补贴。",
      },
      3: {
        title: "便利店夜谈",
        goal: "争取许蔓合作，对手开始截胡。",
        conflict: "许蔓要求林舟说明赔付边界和履约能力，陈岳试图绕开站点直接签店。",
        payoff: "林舟拿到带条件的试点机会。",
        cliffhanger: "周启明扣住骑手排班，逼林舟第二天无人可用。",
      },
    };
    return Array.from({ length: 30 }, (_, index) => {
      const chapterIndex = index + 1;
      const early = earlyChapters[chapterIndex];
      const phase: UrbanOutlineSeed =
        chapterIndex <= 10
          ? {
              title: `第${chapterIndex}章 片区试点`,
              goal: "建立小范围配送闭环，拿到第一批商家。",
              conflict: "站点罚款、骑手排班和商家观望互相牵制。",
              payoff: "片区订单形成稳定样板。",
              cliffhanger: "新的商家要求更高履约保证。",
            }
          : chapterIndex <= 20
            ? {
                title: `第${chapterIndex}章 对手加码`,
                goal: "补齐合同和资金，挡住陈岳复制方案。",
                conflict: "陈岳用补贴和资本包装抢商家，周启明继续卡站点资源。",
                payoff: "林舟用真实履约数据稳住关键商家。",
                cliffhanger: "陈岳拿出更大的片区合作邀约。",
              }
            : {
                title: `第${chapterIndex}章 本地生活入口`,
                goal: "从外卖站升级到社区团购试点，埋下平台巨头线。",
                conflict: "订单规模扩大后，资金、合同和系统能力都逼近上限。",
                payoff: "林舟建立第一套可复制的片区模型。",
                cliffhanger: "平台方开始关注这套异常增长的片区数据。",
              };
      const item = early ?? phase;
      return {
        novel_id: novel.id,
        volume_index: 1,
        chapter_index: chapterIndex,
        title: item.title,
        pov: "第三人称限知",
        goal: item.goal,
        conflict: item.conflict,
        key_events:
          chapterIndex === 1
            ? [
                "林舟确认自己回到 2016 年 7 月 18 日暴雨夜。",
                "他拒绝城南高风险路线，改派医院和学校订单。",
                "许蔓给出十五分钟保单窗口，陈岳提前看见调度方案。",
              ]
            : [
                `第 ${chapterIndex} 章推进本地生活订单目标。`,
                "林舟用可执行动作换取商家或骑手信任。",
                "章尾抛出下一步订单、资金或截胡压力。",
              ],
        character_changes:
          chapterIndex === 1
            ? ["林舟从被动背责转为主动留证", "许蔓从不信任站点转为给出验证窗口"]
            : ["林舟获得更明确的商业判断", "对手开始调整截胡策略"],
        new_facts: [
          {
            subject: chapterIndex === 1 ? "林舟" : "片区试点",
            predicate: chapterIndex === 1 ? "回到" : "推进到",
            object: chapterIndex === 1 ? "2016 年 7 月 18 日暴雨夜" : `第 ${chapterIndex} 章节点`,
            importance: chapterIndex <= 3 ? 5 : 3,
          },
        ],
        payoff: item.payoff,
        foreshadowing: chapterIndex === 1 ? ["陈岳提前注意调度方案"] : [`第 ${chapterIndex} 章留下的商家或资金压力`],
        cliffhanger: item.cliffhanger,
        estimated_word_count: 2600,
      };
    });
  }

  const isFantasy = novel.id.includes("fantasy");
  return Array.from({ length: 30 }, (_, index) => {
    const chapterIndex = index + 1;
    return {
      novel_id: novel.id,
      volume_index: 1,
      chapter_index: chapterIndex,
      title: chapterIndex === 1 ? (isFantasy ? "测骨生变" : "重逢开局") : `第${chapterIndex}章 局面推进`,
      pov: "第三人称限知",
      goal: chapterIndex === 1 ? "让主角进入第一卷核心困境并启动反击。" : "推进主线线索与人物关系。",
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
      cliffhanger: chapterIndex === 1 ? "下一场冲突的关键人物主动出现。" : "新的压力把主角推向下一步选择。",
      estimated_word_count: 2600,
    };
  });
}

function makeChapters(novel: Novel): Chapter[] {
  return makeOutlines(novel).map((outline) => {
    const isFirstUrban = novel.id === DEMO_NOVEL_ID && outline.chapter_index === 1;
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
      score: isFirstUrban ? 84 : hasDraft ? 72 : null,
      word_count: content ? countWords(content) : 0,
      version: isFirstUrban ? 2 : hasDraft ? 1 : 0,
      created_at: minutesAgo(8500 - outline.chapter_index * 18),
      updated_at: minutesAgo(60 - Math.min(outline.chapter_index, 20)),
    };
  });
}

function makeFacts(novel: Novel, chapters: Chapter[]): Fact[] {
  if (novel.id === DEMO_NOVEL_ID) {
    return [
      {
        id: makeId("fact", `${novel.id}-1`),
        novel_id: novel.id,
        chapter_id: chapters[0]?.id,
        subject: "林舟",
        predicate: "回到",
        object: "2016 年 7 月 18 日暴雨夜",
        importance: 5,
        created_at: minutesAgo(75),
      },
      {
        id: makeId("fact", `${novel.id}-2`),
        novel_id: novel.id,
        chapter_id: chapters[0]?.id,
        subject: "城南路线",
        predicate: "存在",
        object: "高风险事故隐患",
        importance: 5,
        created_at: minutesAgo(72),
      },
      {
        id: makeId("fact", `${novel.id}-3`),
        novel_id: novel.id,
        chapter_id: chapters[0]?.id,
        subject: "周启明",
        predicate: "试图",
        object: "把城南事故责任转嫁给林舟",
        importance: 4,
        created_at: minutesAgo(70),
      },
      {
        id: makeId("fact", `${novel.id}-4`),
        novel_id: novel.id,
        chapter_id: chapters[0]?.id,
        subject: "许蔓",
        predicate: "给出",
        object: "十五分钟保单验证窗口",
        importance: 4,
        created_at: minutesAgo(68),
      },
      {
        id: makeId("fact", `${novel.id}-5`),
        novel_id: novel.id,
        chapter_id: chapters[0]?.id,
        subject: "陈岳",
        predicate: "提前注意",
        object: "林舟的调度方案",
        importance: 4,
        created_at: minutesAgo(66),
      },
    ];
  }

  return [
    {
      id: makeId("fact", `${novel.id}-1`),
      novel_id: novel.id,
      chapter_id: chapters[0]?.id,
      subject: "主角",
      predicate: "长期目标",
      object: novel.id.includes("fantasy") ? "查清身世" : "处理核心案件并修复关键关系",
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
        score: 80,
        notes: "Writer -> Continuity -> Style 展示初稿。",
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
        score: 84,
        notes: "根据 Reviewer 建议压缩调度解释并补强第一单回报。",
      },
      created_at: minutesAgo(32),
    },
  ];
}

function makeReview(chapter: Chapter): ReviewReport {
  return {
    id: makeId("review", chapter.id),
    chapter_id: chapter.id,
    total_score: 84,
    passed: true,
    scores: {
      opening_hook_score: 9,
      pacing_score: 8,
      payoff_score: 8,
      character_score: 8,
      dialogue_score: 8,
      continuity_score: 9,
      cliffhanger_score: 8,
      platform_fit_score: 8,
    },
    strengths: [
      "前 800 字直接进入暴雨、甩锅和事故压力",
      "主角用未来信息做当场调度，而不是长篇解释行业趋势",
      "章尾落到许蔓合同和陈岳截胡，下一章行动方向清楚",
    ],
    issues: [
      {
        severity: "medium",
        dimension: "payoff",
        location: "中段调度",
        description: "医院单、学校单和便利店单都出现了，但即时回报还可以更集中。",
      },
      {
        severity: "low",
        dimension: "character",
        location: "周启明出场",
        description: "周启明的短视和甩锅成立，但他的利益算盘还可以再具体一句。",
      },
    ],
    suggestions: ["把三条路线压成一条主线，集中展示第一单送达回报。", "补出周启明为什么急着甩锅，强化他的现实利益算盘。"],
    rewrite_instruction: {
      needed: true,
      rewrite_type: "partial",
      priority: "medium",
      goals: ["压缩调度解释", "强化第一单送达回报"],
      preserve: ["暴雨事故压力", "许蔓十五分钟条件", "陈岳提前注意"],
      change: ["把三条路线压成一条主线", "补出周启明为什么急着甩锅"],
      avoid: ["不要增加长篇行业分析", "不要让合作无阻力谈成"],
    },
    created_at: minutesAgo(30),
  };
}

function makeAgentRuns(): AgentRun[] {
  return [
    {
      id: "run-001",
      novel_id: DEMO_NOVEL_ID,
      role: "market",
      task: "create_novel",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 720,
      prompt_tokens: 860,
      completion_tokens: 360,
      total_tokens: 1220,
      prompt_cost_micro_usd: 860,
      completion_cost_micro_usd: 720,
      total_cost_micro_usd: 1580,
      output_summary: "都市重生商业文，番茄向，第一冲突为暴雨外卖站甩锅。",
      structured: {
        market_brief: {
          title: "重回外卖站",
          platform: "fanqie",
          first_conflict: "暴雨外卖站甩锅",
        },
        _engineering: { duration_ms: 720 },
      },
      raw_text: "",
      raw_notes: "mock market brief",
      parse_error: null,
      created_at: minutesAgo(44),
    },
    {
      id: "run-002",
      novel_id: DEMO_NOVEL_ID,
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
      output_summary: "30 章大纲已生成，首章目标为避开事故并拿到第一份订单。",
      structured: {
        plot_plan: {
          chapters: 30,
          sections: ["暴雨回站", "片区试点", "对手加码", "本地生活入口"],
        },
        _engineering: { duration_ms: 1360 },
      },
      raw_text: "",
      raw_notes: "mock plot output",
      parse_error: null,
      created_at: minutesAgo(40),
    },
    {
      id: "run-003",
      novel_id: DEMO_NOVEL_ID,
      role: "character",
      task: "create_novel",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 860,
      prompt_tokens: 940,
      completion_tokens: 780,
      total_tokens: 1720,
      prompt_cost_micro_usd: 940,
      completion_cost_micro_usd: 1560,
      total_cost_micro_usd: 2500,
      output_summary: "生成林舟、许蔓、周启明、陈岳 4 个核心人物。",
      structured: {
        characters: ["林舟", "许蔓", "周启明", "陈岳"],
        _engineering: { duration_ms: 860 },
      },
      raw_text: "",
      raw_notes: "mock character output mapped to create_novel task",
      parse_error: null,
      created_at: minutesAgo(36),
    },
    {
      id: "run-004",
      novel_id: DEMO_NOVEL_ID,
      role: "writer",
      task: "generate_chapter",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 1280,
      prompt_tokens: 1180,
      completion_tokens: 1680,
      total_tokens: 2860,
      prompt_cost_micro_usd: 1180,
      completion_cost_micro_usd: 3360,
      total_cost_micro_usd: 4540,
      output_summary: "生成第 1 章《暴雨回站》，约 2200 字。",
      structured: {
        chapter_draft: { chapter_index: 1, title: "暴雨回站" },
        _engineering: { duration_ms: 1280 },
      },
      raw_text: "",
      raw_notes: "mock writer output",
      parse_error: null,
      created_at: minutesAgo(33),
    },
    {
      id: "run-005",
      novel_id: DEMO_NOVEL_ID,
      role: "reviewer",
      task: "review_chapter",
      provider: "smoke",
      model: "smoke",
      status: "ok",
      duration_ms: 940,
      prompt_tokens: 980,
      completion_tokens: 460,
      total_tokens: 1440,
      prompt_cost_micro_usd: 980,
      completion_cost_micro_usd: 920,
      total_cost_micro_usd: 1900,
      output_summary: "审稿总分 84，通过，建议局部压缩调度解释。",
      structured: { review_report: { total_score: 84, passed: true }, _engineering: { duration_ms: 940 } },
      raw_text: "",
      raw_notes: "mock reviewer report",
      parse_error: null,
      created_at: minutesAgo(30),
    },
    {
      id: "run-006",
      novel_id: "urban_rebirth_deepseek_baseline",
      role: "reviewer",
      task: "review_chapter",
      provider: "deepseek",
      model: "deepseek-chat",
      status: "fallback",
      duration_ms: 1180,
      prompt_tokens: 1040,
      completion_tokens: 520,
      total_tokens: 1560,
      prompt_cost_micro_usd: 1040,
      completion_cost_micro_usd: 1040,
      total_cost_micro_usd: 2080,
      output_summary: "历史负向样本人工 36/50：即时压力不足、谈判偏顺、章尾方向宣言，建议返工。",
      structured: {
        baseline_review: {
          manual_score: "36/50",
          rewrite_type: "partial",
          major_issues: ["即时压力不足", "谈判偏顺", "章尾方向宣言"],
        },
        _engineering: { duration_ms: 1180, will_fallback: true },
      },
      raw_text: "",
      raw_notes: "negative baseline sample before b-quality-2026-06-09-r3",
      parse_error: null,
      created_at: minutesAgo(120),
    },
  ];
}

function makeApiJobs(): ApiJob[] {
  return [
    {
      id: "job-003",
      kind: "rewrite_chapter",
      status: "running",
      novel_id: DEMO_NOVEL_ID,
      chapter_index: 1,
      source_job_id: null,
      progress_current: 0,
      progress_total: 1,
      payload: {
        novel_id: DEMO_NOVEL_ID,
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
      novel_id: DEMO_NOVEL_ID,
      chapter_index: 2,
      source_job_id: null,
      progress_current: 0,
      progress_total: 1,
      payload: {
        novel_id: DEMO_NOVEL_ID,
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
      novel_id: DEMO_NOVEL_ID,
      chapter_index: 1,
      source_job_id: null,
      progress_current: 1,
      progress_total: 1,
      payload: {
        novel_id: DEMO_NOVEL_ID,
        chapter_index: 1,
      },
      result: {
        report: {
          total_score: 84,
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
      versions[chapter.id] = novel.id === DEMO_NOVEL_ID && chapter.chapter_index === 1 ? makeVersions(chapter) : [];
    });

    const firstChapter = chapterList[0];
    if (novel.id === DEMO_NOVEL_ID && firstChapter) {
      reviews[firstChapter.id] = makeReview(firstChapter);
      firstChapter.content = rewriteContent;
      firstChapter.word_count = countWords(rewriteContent);
      firstChapter.summary = "林舟在暴雨外卖站回到十年前，拒绝城南高风险路线并争取许蔓第一份真实订单。";
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
