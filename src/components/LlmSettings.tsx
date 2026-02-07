import { useState } from "react";
import { testLlmConnection } from "../lib/commands";
import type { AppSettings, LlmConfig, FewShotExample } from "../lib/types";

interface LlmSettingsProps {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}

export default function LlmSettings({ settings, onChange }: LlmSettingsProps) {
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editInput, setEditInput] = useState("");
  const [editOutput, setEditOutput] = useState("");

  const examples = settings.llm.few_shot_examples ?? [];

  const updateLlm = (partial: Partial<LlmConfig>) => {
    onChange({ ...settings, llm: { ...settings.llm, ...partial } });
  };

  const updateExamples = (newExamples: FewShotExample[]) => {
    updateLlm({ few_shot_examples: newExamples });
  };

  const startEdit = (idx: number) => {
    setEditingIdx(idx);
    setEditInput(examples[idx].input);
    setEditOutput(examples[idx].output);
  };

  const saveEdit = () => {
    if (editingIdx === null) return;
    const updated = [...examples];
    updated[editingIdx] = { input: editInput, output: editOutput };
    updateExamples(updated);
    setEditingIdx(null);
  };

  const cancelEdit = () => {
    setEditingIdx(null);
  };

  const deleteExample = (idx: number) => {
    updateExamples(examples.filter((_, i) => i !== idx));
    if (editingIdx === idx) setEditingIdx(null);
  };

  const addExample = () => {
    updateExamples([...examples, { input: "", output: "" }]);
    setEditingIdx(examples.length);
    setEditInput("");
    setEditOutput("");
  };

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testLlmConnection();
      setTestResult(`Success: ${result.slice(0, 100)}`);
    } catch (e) {
      setTestResult(`Error: ${String(e)}`);
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-medium text-text">LLM Configuration</h3>

      <div className="flex gap-4">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="api_type"
            checked={settings.llm.api_type === "ollama"}
            onChange={() => updateLlm({ api_type: "ollama" })}
            className="accent-accent"
          />
          <span className="text-sm text-text">Ollama</span>
        </label>
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="api_type"
            checked={settings.llm.api_type === "openai"}
            onChange={() => updateLlm({ api_type: "openai" })}
            className="accent-accent"
          />
          <span className="text-sm text-text">OpenAI-compatible</span>
        </label>
      </div>

      <div className="space-y-3">
        <div>
          <label className="block text-xs text-text-muted mb-1">Endpoint</label>
          <input
            type="text"
            value={settings.llm.endpoint}
            onChange={(e) => updateLlm({ endpoint: e.target.value })}
            placeholder="http://localhost:11434"
            className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
          />
        </div>

        <div>
          <label className="block text-xs text-text-muted mb-1">Model</label>
          <input
            type="text"
            value={settings.llm.model}
            onChange={(e) => updateLlm({ model: e.target.value })}
            placeholder="mistral"
            className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent"
          />
        </div>

        <div>
          <label className="block text-xs text-text-muted mb-1">System Prompt</label>
          <textarea
            value={settings.llm.system_prompt}
            onChange={(e) => updateLlm({ system_prompt: e.target.value })}
            rows={4}
            className="w-full bg-bg border border-primary rounded px-3 py-2 text-text text-sm focus:outline-none focus:ring-1 focus:ring-accent resize-y"
          />
        </div>

        {/* Few-shot examples */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <label className="block text-xs text-text-muted">
              Few-Shot Examples ({examples.length})
            </label>
            <button
              onClick={addExample}
              className="px-2 py-0.5 text-xs bg-primary rounded hover:bg-blue-700 transition-colors"
            >
              + Add
            </button>
          </div>
          <div className="space-y-2 max-h-80 overflow-y-auto">
            {examples.map((ex, idx) => (
              <div key={idx} className="bg-bg rounded p-3 border border-primary/20">
                {editingIdx === idx ? (
                  <div className="space-y-2">
                    <div>
                      <label className="block text-xs text-text-muted mb-1">
                        Input (raw speech)
                      </label>
                      <textarea
                        value={editInput}
                        onChange={(e) => setEditInput(e.target.value)}
                        rows={2}
                        className="w-full bg-surface border border-primary rounded px-2 py-1 text-text text-xs focus:outline-none focus:ring-1 focus:ring-accent resize-y"
                      />
                    </div>
                    <div>
                      <label className="block text-xs text-text-muted mb-1">
                        Output (cleaned)
                      </label>
                      <textarea
                        value={editOutput}
                        onChange={(e) => setEditOutput(e.target.value)}
                        rows={2}
                        className="w-full bg-surface border border-primary rounded px-2 py-1 text-text text-xs focus:outline-none focus:ring-1 focus:ring-accent resize-y"
                      />
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={saveEdit}
                        className="px-2 py-0.5 text-xs bg-primary rounded hover:bg-blue-700 transition-colors"
                      >
                        Save
                      </button>
                      <button
                        onClick={cancelEdit}
                        className="px-2 py-0.5 text-xs text-text-muted hover:text-text transition-colors"
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <div
                    className="cursor-pointer"
                    onClick={() => startEdit(idx)}
                  >
                    <p className="text-xs text-text-muted mb-1">
                      <span className="text-text-muted/60">In:</span>{" "}
                      {ex.input.slice(0, 80)}
                      {ex.input.length > 80 && "..."}
                    </p>
                    <p className="text-xs text-text">
                      <span className="text-text-muted/60">Out:</span>{" "}
                      {ex.output.slice(0, 80)}
                      {ex.output.length > 80 && "..."}
                    </p>
                    <div className="flex justify-end mt-1">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteExample(idx);
                        }}
                        className="px-2 py-0.5 text-xs text-error hover:bg-error/10 rounded transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))}
            {examples.length === 0 && (
              <p className="text-xs text-text-muted italic">
                No examples. The LLM may not clean text correctly without examples.
              </p>
            )}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={handleTest}
            disabled={testing}
            className="px-4 py-2 text-sm bg-primary rounded hover:bg-blue-700 disabled:opacity-50 transition-colors"
          >
            {testing ? "Testing..." : "Test Connection"}
          </button>
          {testResult && (
            <span
              className={`text-xs ${testResult.startsWith("Success") ? "text-success" : "text-error"}`}
            >
              {testResult}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
