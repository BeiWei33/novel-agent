import { Link } from "react-router-dom";
import { useMutation, useQuery } from "@tanstack/react-query";
import { FilePlus2 } from "lucide-react";
import type { NovelListItem } from "../types/domain";
import { api, queryKeys } from "../lib/api";
import { NovelTable } from "../features/novels/NovelTable";
import { PageHeader } from "../components/PageHeader";
import { LoadingState } from "../components/LoadingState";
import { EmptyState } from "../components/EmptyState";
import { Button } from "../components/ui/Button";
import { StatusBanner } from "../components/StatusBanner";

export function NovelListPage() {
  const novelsQuery = useQuery({
    queryKey: queryKeys.novels,
    queryFn: api.getNovels,
  });

  const exportMutation = useMutation({
    mutationFn: (novel: NovelListItem) => api.exportMarkdown(novel.id).then((content) => ({ novel, content })),
    onSuccess: ({ novel, content }) => {
      const blob = new Blob([content], { type: "text/markdown;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `${novel.title}.md`;
      anchor.click();
      URL.revokeObjectURL(url);
    },
  });

  return (
    <div>
      <PageHeader
        title="作品"
        meta={<span>{novelsQuery.data?.length ?? 0} 本</span>}
        actions={
          <Link to="/novels/new">
            <Button variant="primary">
              <FilePlus2 className="h-4 w-4" />
              新建作品
            </Button>
          </Link>
        }
      />
      {exportMutation.isPending ? <StatusBanner title="正在导出 Markdown">正在整理当前作品章节正文。</StatusBanner> : null}
      {exportMutation.isError ? (
        <StatusBanner tone="danger" title="导出失败">
          {exportMutation.error instanceof Error ? exportMutation.error.message : "请稍后重试。"}
        </StatusBanner>
      ) : null}
      {novelsQuery.isLoading ? <LoadingState label="读取作品列表" /> : null}
      {novelsQuery.data && novelsQuery.data.length > 0 ? (
        <NovelTable novels={novelsQuery.data} onExport={(novel) => exportMutation.mutate(novel)} />
      ) : null}
      {novelsQuery.data?.length === 0 ? (
        <div className="p-4">
          <EmptyState
            title="暂无作品"
            action={
              <Link to="/novels/new">
                <Button variant="primary">新建作品</Button>
              </Link>
            }
          />
        </div>
      ) : null}
      {novelsQuery.isError ? (
        <StatusBanner tone="danger" title="作品列表读取失败">
          {novelsQuery.error instanceof Error ? novelsQuery.error.message : "请检查 API 或 mock 数据。"}
        </StatusBanner>
      ) : null}
    </div>
  );
}
