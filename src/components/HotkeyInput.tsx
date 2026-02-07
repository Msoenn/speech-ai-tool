import { useState } from "react";

interface HotkeyInputProps {
  value: string;
  onChange: (hotkey: string) => void;
}

export default function HotkeyInput({ value, onChange }: HotkeyInputProps) {
  const [editing, setEditing] = useState(false);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push("CmdOrCtrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");

    const key = e.key;
    if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
      // Map special keys
      const keyMap: Record<string, string> = {
        " ": "Space",
        ArrowUp: "Up",
        ArrowDown: "Down",
        ArrowLeft: "Left",
        ArrowRight: "Right",
      };
      parts.push(keyMap[key] || key.length === 1 ? key.toUpperCase() : key);

      const hotkey = parts.join("+");
      onChange(hotkey);
      setEditing(false);
    }
  };

  return (
    <div className="space-y-1">
      <label className="block text-sm font-medium text-text-muted">Global Hotkey</label>
      {editing ? (
        <input
          autoFocus
          onKeyDown={handleKeyDown}
          onBlur={() => setEditing(false)}
          placeholder="Press key combination..."
          className="w-full bg-bg border border-accent rounded px-3 py-2 text-text text-sm focus:outline-none"
          readOnly
        />
      ) : (
        <button
          onClick={() => setEditing(true)}
          className="w-full text-left bg-bg border border-primary rounded px-3 py-2 text-text text-sm hover:border-accent transition-colors"
        >
          <kbd className="px-2 py-0.5 bg-primary rounded text-xs">{value}</kbd>
          <span className="text-text-muted text-xs ml-2">Click to change</span>
        </button>
      )}
    </div>
  );
}
