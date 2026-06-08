use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use novel_agent::agents::ModelHandle;
use novel_agent::config::AppConfig;
use novel_agent::domain::{NovelId, TargetPlatform};
use novel_agent::model::{ModelProvider, RigModelClient};
use novel_agent::storage::SqliteStorage;
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
    },
    Outline {
        #[arg(long)]
        novel_id: String,
        #[arg(long, default_value_t = 30)]
        chapters: u32,
    },
    Write {
        #[arg(long)]
        novel_id: String,
        #[arg(long)]
        chapter: u32,
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
    },
    Export {
        #[arg(long)]
        novel_id: String,
        #[arg(long, value_enum, default_value_t = ExportFormat::Markdown)]
        format: ExportFormat,
        #[arg(long)]
        output: Option<PathBuf>,
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
    let model: ModelHandle = Arc::new(RigModelClient::new(provider, config.model.model.clone()));

    match cli.command {
        Commands::New { idea, platform } => {
            let platform = TargetPlatform::from_str(&platform)?;
            let workflow = NovelCreationWorkflow::new(&storage, model.clone());
            let result = workflow.create_from_idea(&idea, platform).await?;

            println!("小说项目 ID: {}", result.novel.id);
            println!("标题: {}", result.novel.title);
            println!("目标平台: {}", result.novel.target_platform);
            println!("核心卖点: {}", result.bible.premise);
            println!("前 {} 章大纲已生成。", result.outlines.len());
            if result.used_fallback {
                println!("提示: 部分 Agent 调用失败，已使用 smoke fallback 产物。");
            }
        }
        Commands::Outline { novel_id, chapters } => {
            let workflow = NovelCreationWorkflow::new(&storage, model.clone());
            let outlines = workflow
                .generate_outline(&NovelId::from(novel_id), chapters)
                .await?;

            println!("已为小说生成/更新 {} 章大纲。", outlines.len());
        }
        Commands::Write { novel_id, chapter } => {
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
        Commands::Rewrite { novel_id, chapter } => {
            let workflow = ChapterGenerationWorkflow::new(&storage, model.clone());
            let draft = workflow
                .rewrite_chapter(&NovelId::from(novel_id), chapter)
                .await?;

            println!(
                "已生成第 {} 章重写版本 v{}。",
                draft.chapter_index, draft.version
            );
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
    }

    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "novel_agent=info".into());

    tracing_subscriber::fmt().with_env_filter(filter).init();
    info!("novel-agent starting");
}
