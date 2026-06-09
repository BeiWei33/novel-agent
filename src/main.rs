use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand, ValueEnum};
use novel_agent::agents::ModelHandle;
use novel_agent::config::AppConfig;
use novel_agent::domain::{NovelId, TargetPlatform};
use novel_agent::model::{ModelProvider, RigModelClient, SmokeModelClient};
use novel_agent::storage::{AgentRunStatusSummary, SqliteStorage};
use novel_agent::workflow::{ChapterGenerationWorkflow, NovelCreationWorkflow};
use tracing::info;

#[derive(Debug, Parser)]
#[command(name = "novel-agent")]
#[command(about = "多 Agent 编排的中文长篇网文自动创作系统")]
struct Cli {
    #[arg(long, default_value = "novel-agent.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    New {
        idea: String,
        #[arg(long, default_value = "general")]
        platform: String,
        #[arg(long, default_value_t = 30)]
        chapters: u32,
        #[arg(long, default_value_t = 5)]
        outline_batch_size: u32,
        #[arg(long)]
        resume_novel_id: Option<String>,
    },
    Outline {
        #[arg(long)]
        novel_id: String,
        #[arg(long, default_value_t = 30)]
        chapters: u32,
        #[arg(long, default_value_t = 5)]
        batch_size: u32,
    },
    Write {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
        #[arg(long)]
        stream: bool,
    },
    Review {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
    },
    Rewrite {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
        #[arg(long)]
        stream: bool,
    },
    Edit {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        summary: Option<String>,
    },
    Export {
        #[arg(long)]
        novel_id: String,
        #[arg(long, value_enum, default_value_t = ExportFormat::Markdown)]
        format: ExportFormat,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Runs {
        #[arg(long)]
        novel_id: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        summary: bool,
        #[arg(long)]
        fail_on_bad_status: bool,
    },
    Serve {
        #[arg(long, default_value = "127.0.0.1:3001")]
        bind: String,
    },
    Versions {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
        #[arg(long)]
        show: Option<u32>,
        #[arg(long)]
        from: Option<u32>,
        #[arg(long)]
        to: Option<u32>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ExportFormat {
    Markdown,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = AppConfig::load(&cli.config).await?;
    let storage = SqliteStorage::connect(&config.storage.database_url)
        .await
        .context("failed to connect storage")?;
    storage.migrate().await?;
    let provider = ModelProvider::parse(&config.model.provider)?;
    let model: ModelHandle = match provider {
        ModelProvider::OpenAi | ModelProvider::DeepSeek => Arc::new(
            RigModelClient::new(provider, config.model.model.clone())
                .with_reasoning_effort(config.model.reasoning_effort.clone()),
        ),
        ModelProvider::Smoke => Arc::new(SmokeModelClient::new(config.model.model.clone())),
    };

    match cli.command {
        Commands::New {
            idea,
            platform,
            chapters,
            outline_batch_size,
            resume_novel_id,
        } => {
            let platform = TargetPlatform::from_str(&platform)?;
            let workflow = NovelCreationWorkflow::new(&storage, model.clone());
            let result = if let Some(novel_id) = resume_novel_id {
                workflow
                    .resume_from_idea_with_outline_batch_size(
                        &NovelId::from(novel_id),
                        &idea,
                        chapters,
                        outline_batch_size,
                    )
                    .await?
            } else {
                workflow
                    .create_from_idea_with_outline_batch_size(
                        &idea,
                        platform,
                        chapters,
                        outline_batch_size,
                    )
                    .await?
            };

            println!("小说项目 ID: {}", result.novel.id);
            println!("标题: {}", result.novel.title);
            println!("目标平台: {}", result.novel.target_platform);
            println!("核心卖点: {}", result.bible.premise);
            println!("前 {} 章大纲已生成。", result.outlines.len());
            if result.used_fallback {
                println!("提示: 部分 Agent 调用失败，已使用 smoke fallback 产物。");
            }
        }
        Commands::Outline {
            novel_id,
            chapters,
            batch_size,
        } => {
            let workflow = NovelCreationWorkflow::new(&storage, model.clone());
            let outlines = workflow
                .generate_outline_with_batch_size(&NovelId::from(novel_id), chapters, batch_size)
                .await?;

            println!("已为小说生成/更新 {} 章大纲。", outlines.len());
        }
        Commands::Write {
            novel_id,
            chapter,
            stream,
        } => {
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let draft = workflow
                .write_chapter(&NovelId::from(novel_id), chapter)
                .await?;

            println!("第 {} 章: {}", draft.chapter_index, draft.title);
            println!("摘要: {}", draft.summary);
            println!("字数: {}", draft.word_count);
            if draft
                .continuity_notes
                .iter()
                .any(|note| note.to_ascii_lowercase().contains("fallback"))
            {
                println!("提示: 本章使用 smoke fallback 生成。");
            }
            if stream {
                print_streamed_text(&draft.content)?;
            }
        }
        Commands::Review { novel_id, chapter } => {
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let report = workflow
                .review_chapter(&NovelId::from(novel_id), chapter)
                .await?;

            println!("审稿总分: {}", report.total_score);
            println!("是否通过: {}", if report.passed { "是" } else { "否" });
            println!("修改建议: {}", report.suggestions.join("；"));
            if report
                .suggestions
                .iter()
                .any(|suggestion| suggestion.to_ascii_lowercase().contains("fallback"))
            {
                println!("提示: 本次审稿使用 smoke fallback。");
            }
        }
        Commands::Rewrite {
            novel_id,
            chapter,
            stream,
        } => {
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let draft = workflow
                .rewrite_chapter(&NovelId::from(novel_id), chapter)
                .await?;

            println!(
                "已生成第 {} 章重写版本 v{}。",
                draft.chapter_index, draft.version
            );
            if stream {
                print_streamed_text(&draft.content)?;
            }
        }
        Commands::Edit {
            novel_id,
            chapter,
            input,
            title,
            summary,
        } => {
            let content = tokio::fs::read_to_string(&input)
                .await
                .with_context(|| format!("failed to read edit input {}", input.display()))?;
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let draft = workflow
                .save_manual_edit(&NovelId::from(novel_id), chapter, title, content, summary)
                .await?;

            println!(
                "已保存第 {} 章人工编辑版本 v{}。",
                draft.chapter_index, draft.version
            );
            println!("标题: {}", draft.title);
            println!("摘要: {}", draft.summary);
            println!("字数: {}", draft.word_count);
        }
        Commands::Export {
            novel_id,
            format: ExportFormat::Markdown,
            output,
        } => {
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let path = workflow
                .export_markdown(&NovelId::from(novel_id), output)
                .await?;

            println!("已导出 Markdown: {}", path.display());
        }
        Commands::Runs {
            novel_id,
            limit,
            summary,
            fail_on_bad_status,
        } => {
            let novel_id = novel_id.map(NovelId::from);
            let runs = storage
                .agent_runs()
                .list_recent(novel_id.as_ref(), limit)
                .await?;
            let status_summary = AgentRunStatusSummary::from_runs(&runs);

            if runs.is_empty() {
                println!("暂无 Agent 运行记录。");
            } else {
                for run in &runs {
                    print_agent_run(run);
                }
            }

            if summary || fail_on_bad_status {
                print_agent_run_summary(&status_summary);
            }

            if fail_on_bad_status && status_summary.has_bad_status() {
                bail!(
                    "AgentRun status check failed: fallback={}, parse_error={} in listed {} runs",
                    status_summary.fallback,
                    status_summary.parse_error,
                    status_summary.total
                );
            }
        }
        Commands::Serve { bind } => {
            let recovered_jobs = storage
                .jobs()
                .fail_incomplete(
                    "API server restarted before the job completed; create a new job to retry.",
                )
                .await?;
            if recovered_jobs > 0 {
                info!(
                    count = recovered_jobs,
                    "marked incomplete API jobs as failed"
                );
            }
            let listener = tokio::net::TcpListener::bind(&bind)
                .await
                .with_context(|| format!("failed to bind API server at {bind}"))?;
            let local_addr = listener
                .local_addr()
                .context("failed to read API server local address")?;
            println!("API 服务已启动: http://{local_addr}");
            axum::serve(listener, novel_agent::api::router(storage, model.clone())).await?;
        }
        Commands::Versions {
            novel_id,
            chapter,
            show,
            from,
            to,
        } => {
            let novel_id = NovelId::from(novel_id);
            let chapter = storage
                .chapters()
                .find_by_index(&novel_id, chapter)
                .await?
                .ok_or_else(|| anyhow::anyhow!("chapter not found"))?;
            let versions = storage
                .chapter_versions()
                .list_version_numbers(&chapter.id)
                .await?;
            if versions.is_empty() {
                println!("第 {} 章暂无版本快照。", chapter.chapter_index);
                return Ok(());
            }

            let labels = versions
                .iter()
                .map(|version| format!("v{version}"))
                .collect::<Vec<_>>()
                .join(", ");
            println!("第 {} 章版本: {}", chapter.chapter_index, labels);

            if let Some(version) = show {
                let content = storage
                    .chapter_versions()
                    .content_for_version(&chapter.id, version)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("version v{version} not found"))?;
                println!("=== v{} ===", version);
                println!("{content}");
            }

            match (from, to) {
                (Some(from), Some(to)) => {
                    let from_content = storage
                        .chapter_versions()
                        .content_for_version(&chapter.id, from)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("version v{from} not found"))?;
                    let to_content = storage
                        .chapter_versions()
                        .content_for_version(&chapter.id, to)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("version v{to} not found"))?;
                    print_version_compare(from, &from_content, to, &to_content);
                }
                (None, None) => {}
                _ => bail!("--from and --to must be provided together"),
            }
        }
    }

    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "novel_agent=info".into());

    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("novel-agent starting");
}

fn print_streamed_text(content: &str) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "正文流式输出:")?;
    for chunk in chunk_text(content, 48) {
        write!(stdout, "{chunk}")?;
        stdout.flush()?;
    }
    writeln!(stdout)?;
    Ok(())
}

fn chunk_text(text: &str, chunk_chars: usize) -> Vec<String> {
    let chunk_chars = chunk_chars.max(1);
    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if current.chars().count() >= chunk_chars {
            chunks.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn print_agent_run(run: &novel_agent::storage::AgentRunRecord) {
    let attempt = display_optional_u64(run.attempt());
    let duration_ms = display_optional_u64(run.duration_ms());
    let total_tokens = display_optional_u64(run.total_tokens());
    let novel_id = run
        .novel_id
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "-".to_string());

    println!(
        "{} role={} task={} novel={} attempt={} status={} duration_ms={} total_tokens={}",
        run.created_at.to_rfc3339(),
        run.role,
        run.task,
        novel_id,
        attempt,
        run.status().as_str(),
        duration_ms,
        total_tokens
    );

    if let Some(parse_error) = &run.parse_error {
        println!("  parse_error: {}", parse_error);
    }
}

fn print_agent_run_summary(summary: &AgentRunStatusSummary) {
    println!(
        "agent_run_summary total={} ok={} fallback={} parse_error={} duration_ms_total={} tokenized_runs={} prompt_tokens={} completion_tokens={} total_tokens={}",
        summary.total,
        summary.ok,
        summary.fallback,
        summary.parse_error,
        summary.duration_ms_total,
        summary.tokenized_runs,
        summary.prompt_tokens,
        summary.completion_tokens,
        summary.total_tokens
    );
}

fn display_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn print_version_compare(from: u32, from_content: &str, to: u32, to_content: &str) {
    let from_count = count_non_whitespace_chars(from_content);
    let to_count = count_non_whitespace_chars(to_content);
    let delta = i64::from(to_count) - i64::from(from_count);
    let shared_prefix = shared_prefix_chars(from_content, to_content);

    println!("对比 v{} -> v{}", from, to);
    println!("v{} 字数: {}", from, from_count);
    println!("v{} 字数: {}", to, to_count);
    println!("字数变化: {:+}", delta);
    println!("共同前缀字符: {}", shared_prefix);
    println!("v{} 预览: {}", from, preview_text(from_content, 80));
    println!("v{} 预览: {}", to, preview_text(to_content, 80));
}

fn count_non_whitespace_chars(content: &str) -> u32 {
    content.chars().filter(|ch| !ch.is_whitespace()).count() as u32
}

fn shared_prefix_chars(left: &str, right: &str) -> u32 {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count() as u32
}

fn preview_text(content: &str, max_chars: usize) -> String {
    let mut preview = content
        .chars()
        .filter(|ch| *ch != '\r' && *ch != '\n')
        .take(max_chars)
        .collect::<String>();
    if content.chars().count() > max_chars {
        preview.push_str("...");
    }
    preview
}
