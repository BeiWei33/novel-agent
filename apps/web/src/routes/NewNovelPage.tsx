import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import type { CreateNovelInput } from "../types/domain";
import { api, queryKeys } from "../lib/api";
import { NewNovelForm } from "../features/novels/NewNovelForm";
import { PageHeader } from "../components/PageHeader";
import { StatusBanner } from "../components/StatusBanner";

export function NewNovelPage() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const createMutation = useMutation({
    mutationFn: (input: CreateNovelInput) => api.createNovel(input),
    onSuccess: async (novel) => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.novels }),
        queryClient.invalidateQueries({ queryKey: queryKeys.agentRuns() }),
      ]);
      navigate(`/novels/${novel.id}`);
    },
  });
  const createJobMutation = useMutation({
    mutationFn: (input: CreateNovelInput) => api.createNovelJob(input),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.jobsRoot });
      navigate("/jobs");
    },
  });

  return (
    <div>
      <PageHeader title="新建作品" />
      {createMutation.isPending ? (
        <StatusBanner title="正在创建小说">正在生成小说圣经、人物卡和 30 章 mock 大纲。</StatusBanner>
      ) : null}
      {createJobMutation.isPending ? (
        <StatusBanner title="正在提交后台创建任务">任务创建成功后会进入任务队列。</StatusBanner>
      ) : null}
      {createMutation.isError ? (
        <StatusBanner tone="danger" title="创建失败">
          {createMutation.error instanceof Error ? createMutation.error.message : "请检查输入后重试。"}
        </StatusBanner>
      ) : null}
      {createJobMutation.isError ? (
        <StatusBanner tone="danger" title="后台创建提交失败">
          {createJobMutation.error instanceof Error ? createJobMutation.error.message : "请检查输入后重试。"}
        </StatusBanner>
      ) : null}
      <NewNovelForm
        onSubmit={(input) => createMutation.mutate(input)}
        onQueueSubmit={(input) => createJobMutation.mutate(input)}
        isPending={createMutation.isPending}
        isQueueing={createJobMutation.isPending}
      />
    </div>
  );
}
