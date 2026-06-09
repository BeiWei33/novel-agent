import { FormEvent, useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Cpu, Save } from "lucide-react";
import type { ApiModelPricing, ApiModelProvider, ApiModelSettings } from "../types/api";
import { api, queryKeys } from "../lib/api";
import { Button } from "./ui/Button";
import { Badge } from "./ui/Badge";

const providerOptions: Array<{ value: ApiModelProvider; label: string }> = [
  { value: "smoke", label: "Smoke" },
  { value: "openai", label: "OpenAI" },
  { value: "deepseek", label: "DeepSeek" },
];

const reasoningOptions = ["", "low", "medium", "high"];

type ModelSettingsDraft = Omit<ApiModelSettings, "pricing"> & {
  pricing?: Partial<ApiModelPricing> | null;
};

const defaultSettings: ApiModelSettings = {
  provider: "smoke",
  model: "smoke",
  reasoning_effort: null,
  pricing: {
    prompt_cost_micro_usd_per_million_tokens: 1_000_000,
    completion_cost_micro_usd_per_million_tokens: 2_000_000,
  },
};

export function ModelSettingsPanel() {
  const queryClient = useQueryClient();
  const status = api.getClientStatus();
  const modelQuery = useQuery({
    queryKey: queryKeys.model,
    queryFn: () => api.getModelSettings(),
    refetchInterval: status.mode === "real" ? 30_000 : false,
    retry: status.mode === "real" ? 1 : false,
  });
  const [draft, setDraft] = useState<ModelSettingsDraft>(defaultSettings);
  const [hasLoadedModel, setHasLoadedModel] = useState(false);
  const saved = modelQuery.data ?? defaultSettings;
  const normalizedDraft = useMemo(() => normalizeDraft(draft), [draft]);
  const isDirty =
    normalizedDraft.provider !== saved.provider ||
    normalizedDraft.model !== saved.model ||
    (normalizedDraft.reasoning_effort ?? null) !== (saved.reasoning_effort ?? null) ||
    !samePricing(normalizedDraft.pricing, saved.pricing);
  const updateMutation = useMutation({
    mutationFn: (settings: ApiModelSettings) => api.updateModelSettings(settings),
    onSuccess: (settings) => {
      queryClient.setQueryData(queryKeys.model, settings);
      setDraft(settings);
      setHasLoadedModel(true);
    },
  });

  useEffect(() => {
    if (modelQuery.data && !updateMutation.isPending && !hasLoadedModel) {
      setDraft(modelQuery.data);
      setHasLoadedModel(true);
    }
  }, [hasLoadedModel, modelQuery.data, updateMutation.isPending]);

  function handleProviderChange(provider: ApiModelProvider) {
    setDraft((current) => ({
      provider,
      model: shouldReplaceDefaultModel(current) ? defaultModelForProvider(provider) : current.model,
      reasoning_effort: provider === "openai" ? (current.reasoning_effort ?? "") : null,
      pricing: current.pricing ?? null,
    }));
  }

  function handlePricingChange(field: keyof ApiModelPricing, value: string) {
    setDraft((current) => {
      const nextPricing = { ...(current.pricing ?? {}) };
      const parsed = value === "" ? undefined : Number(value);
      if (typeof parsed === "number" && Number.isFinite(parsed)) {
        nextPricing[field] = parsed;
      } else {
        delete nextPricing[field];
      }
      return {
        ...current,
        pricing: Object.keys(nextPricing).length > 0 ? nextPricing : null,
      };
    });
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    updateMutation.mutate(normalizedDraft);
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-2 border-t border-line pt-3">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 text-xs font-semibold text-slate-700">
          <Cpu className="h-3.5 w-3.5" />
          模型
        </div>
        <Badge tone={modelQuery.isError ? "rose" : updateMutation.isPending ? "amber" : "blue"}>{saved.provider}</Badge>
      </div>
      <label className="block">
        <span className="field-label">Provider</span>
        <select
          className="input mt-1 h-8 px-2 text-xs"
          value={providerValue(draft.provider)}
          onChange={(event) => handleProviderChange(event.target.value as ApiModelProvider)}
          disabled={updateMutation.isPending}
        >
          {providerOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </label>
      <label className="block">
        <span className="field-label">Model</span>
        <input
          className="input mt-1 h-8 px-2 text-xs"
          value={draft.model}
          onChange={(event) => setDraft((current) => ({ ...current, model: event.target.value }))}
          disabled={updateMutation.isPending}
        />
      </label>
      {providerValue(draft.provider) === "openai" ? (
        <label className="block">
          <span className="field-label">Reasoning</span>
          <select
            className="input mt-1 h-8 px-2 text-xs"
            value={draft.reasoning_effort ?? ""}
            onChange={(event) =>
              setDraft((current) => ({
                ...current,
                reasoning_effort: event.target.value || null,
              }))
            }
            disabled={updateMutation.isPending}
          >
            {reasoningOptions.map((option) => (
              <option key={option || "default"} value={option}>
                {option || "default"}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      <div className="grid grid-cols-2 gap-2">
        <label className="block">
          <span className="field-label">Prompt price</span>
          <input
            type="number"
            min={0}
            step={1}
            className="input mt-1 h-8 px-2 text-xs"
            value={draft.pricing?.prompt_cost_micro_usd_per_million_tokens ?? ""}
            onChange={(event) => handlePricingChange("prompt_cost_micro_usd_per_million_tokens", event.target.value)}
            disabled={updateMutation.isPending}
          />
        </label>
        <label className="block">
          <span className="field-label">Output price</span>
          <input
            type="number"
            min={0}
            step={1}
            className="input mt-1 h-8 px-2 text-xs"
            value={draft.pricing?.completion_cost_micro_usd_per_million_tokens ?? ""}
            onChange={(event) => handlePricingChange("completion_cost_micro_usd_per_million_tokens", event.target.value)}
            disabled={updateMutation.isPending}
          />
        </label>
      </div>
      <div className="flex items-center justify-between gap-2">
        <div className="min-w-0 text-xs text-slate-500">
          {modelQuery.isLoading
            ? "读取中"
            : updateMutation.isError
              ? errorMessage(updateMutation.error)
              : modelQuery.isError
                ? errorMessage(modelQuery.error)
                : updateMutation.isSuccess
                  ? "已保存"
                  : saved.model}
        </div>
        <Button type="submit" size="sm" variant="secondary" disabled={!isDirty || updateMutation.isPending || modelQuery.isLoading}>
          <Save className="h-3.5 w-3.5" />
          保存
        </Button>
      </div>
    </form>
  );
}

function providerValue(provider: string): ApiModelProvider {
  if (provider === "openai" || provider === "deepseek") {
    return provider;
  }
  return "smoke";
}

function defaultModelForProvider(provider: string): string {
  if (provider === "openai") {
    return "gpt-5";
  }
  if (provider === "deepseek") {
    return "deepseek-chat";
  }
  return "smoke";
}

function shouldReplaceDefaultModel(settings: ModelSettingsDraft): boolean {
  return settings.model.trim() === "" || settings.model === defaultModelForProvider(providerValue(settings.provider));
}

function normalizeDraft(settings: ModelSettingsDraft): ApiModelSettings {
  const provider = providerValue(settings.provider);
  return {
    provider,
    model: settings.model.trim() || defaultModelForProvider(provider),
    reasoning_effort: provider === "openai" ? (settings.reasoning_effort?.trim() || null) : null,
    pricing: normalizePricing(settings.pricing),
  };
}

function normalizePricing(pricing: ModelSettingsDraft["pricing"]): ApiModelSettings["pricing"] {
  const promptCost = pricing?.prompt_cost_micro_usd_per_million_tokens;
  const completionCost = pricing?.completion_cost_micro_usd_per_million_tokens;
  if (typeof promptCost !== "number" || typeof completionCost !== "number") {
    return null;
  }
  return {
    prompt_cost_micro_usd_per_million_tokens: Math.max(0, Math.round(promptCost)),
    completion_cost_micro_usd_per_million_tokens: Math.max(0, Math.round(completionCost)),
  };
}

function samePricing(left: ApiModelSettings["pricing"], right: ApiModelSettings["pricing"]): boolean {
  return (
    (left?.prompt_cost_micro_usd_per_million_tokens ?? null) === (right?.prompt_cost_micro_usd_per_million_tokens ?? null) &&
    (left?.completion_cost_micro_usd_per_million_tokens ?? null) === (right?.completion_cost_micro_usd_per_million_tokens ?? null)
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "请求失败";
}
