import { useState, useEffect, useCallback } from "react";
import { pauseHotkey } from "../lib/commands";

interface HotkeyInputProps {
  value: string;
  onChange: (hotkey: string) => void;
}

// Map JS KeyboardEvent.code to rdev key names
const CODE_TO_RDEV: Record<string, string> = {
  ControlLeft: "ControlLeft",
  ControlRight: "ControlRight",
  ShiftLeft: "ShiftLeft",
  ShiftRight: "ShiftRight",
  AltLeft: "AltLeft",
  AltRight: "AltRight",
  MetaLeft: "MetaLeft",
  MetaRight: "MetaRight",
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  Escape: "Escape",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  CapsLock: "CapsLock",
  ArrowUp: "ArrowUp",
  ArrowDown: "ArrowDown",
  ArrowLeft: "ArrowLeft",
  ArrowRight: "ArrowRight",
  Minus: "Minus",
  Equal: "Equal",
  BracketLeft: "BracketLeft",
  BracketRight: "BracketRight",
  Backslash: "Backslash",
  Semicolon: "Semicolon",
  Quote: "Quote",
  Backquote: "Backquote",
  Comma: "Comma",
  Period: "Period",
  Slash: "Slash",
  NumpadDecimal: "NumpadDecimal",
  NumpadAdd: "NumpadAdd",
  NumpadSubtract: "NumpadSubtract",
  NumpadMultiply: "NumpadMultiply",
  NumpadDivide: "NumpadDivide",
  NumpadEnter: "NumpadEnter",
};

// Add letter keys (KeyA-KeyZ)
for (let i = 65; i <= 90; i++) {
  const letter = String.fromCharCode(i);
  CODE_TO_RDEV[`Key${letter}`] = `Key${letter}`;
}

// Add digit keys (Digit0-Digit9)
for (let i = 0; i <= 9; i++) {
  CODE_TO_RDEV[`Digit${i}`] = `Digit${i}`;
}

// Add function keys (F1-F12)
for (let i = 1; i <= 12; i++) {
  CODE_TO_RDEV[`F${i}`] = `F${i}`;
}

// Add numpad keys (Numpad0-Numpad9)
for (let i = 0; i <= 9; i++) {
  CODE_TO_RDEV[`Numpad${i}`] = `Numpad${i}`;
}

// Human-readable display names for rdev key names
const DISPLAY_NAMES: Record<string, string> = {
  ControlLeft: "Left Ctrl",
  ControlRight: "Right Ctrl",
  ShiftLeft: "Left Shift",
  ShiftRight: "Right Shift",
  AltLeft: "Left Alt",
  AltRight: "Right Alt",
  MetaLeft: "Left Super",
  MetaRight: "Right Super",
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  Escape: "Esc",
  Backspace: "Backspace",
  Delete: "Delete",
  Insert: "Insert",
  CapsLock: "Caps Lock",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Backquote: "`",
  Comma: ",",
  Period: ".",
  Slash: "/",
};

function displayName(rdevKey: string): string {
  if (DISPLAY_NAMES[rdevKey]) return DISPLAY_NAMES[rdevKey];
  if (rdevKey.startsWith("Key")) return rdevKey.slice(3);
  if (rdevKey.startsWith("Digit")) return rdevKey.slice(5);
  if (rdevKey.startsWith("Numpad")) return "Num " + rdevKey.slice(6);
  return rdevKey;
}

function formatHotkeyDisplay(hotkey: string): string {
  return hotkey
    .split("+")
    .map((k) => displayName(k))
    .join(" + ");
}

export default function HotkeyInput({ value, onChange }: HotkeyInputProps) {
  const [editing, setEditing] = useState(false);
  const [heldKeys, setHeldKeys] = useState<Set<string>>(new Set());

  const startEditing = useCallback(() => {
    setEditing(true);
    setHeldKeys(new Set());
    pauseHotkey(true).catch(console.error);
  }, []);

  const stopEditing = useCallback(() => {
    setEditing(false);
    setHeldKeys(new Set());
    pauseHotkey(false).catch(console.error);
  }, []);

  useEffect(() => {
    if (!editing) return;

    const held = new Set<string>();

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const rdevKey = CODE_TO_RDEV[e.code];
      if (!rdevKey) return;
      held.add(rdevKey);
      setHeldKeys(new Set(held));
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (held.size > 0) {
        // Accept the combo on first key release
        const combo = Array.from(held).join("+");
        onChange(combo);
        held.clear();
        stopEditing();
      }
    };

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("keyup", handleKeyUp, true);

    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("keyup", handleKeyUp, true);
    };
  }, [editing, onChange, stopEditing]);

  return (
    <div className="space-y-1">
      <label className="block text-sm font-medium text-text-muted">
        Global Hotkey
      </label>
      {editing ? (
        <div className="relative">
          <div className="w-full bg-bg border border-accent rounded px-3 py-2 text-text text-sm focus-within:outline-none">
            {heldKeys.size > 0 ? (
              <span>
                Holding:{" "}
                <kbd className="px-2 py-0.5 bg-primary rounded text-xs">
                  {Array.from(heldKeys).map(displayName).join(" + ")}
                </kbd>
              </span>
            ) : (
              <span className="text-text-muted">
                Press key combination... (release to save)
              </span>
            )}
          </div>
          <button
            onClick={stopEditing}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-text-muted hover:text-text text-xs"
          >
            Cancel
          </button>
        </div>
      ) : (
        <button
          onClick={startEditing}
          className="w-full text-left bg-bg border border-primary rounded px-3 py-2 text-text text-sm hover:border-accent transition-colors"
        >
          <kbd className="px-2 py-0.5 bg-primary rounded text-xs">
            {formatHotkeyDisplay(value)}
          </kbd>
          <span className="text-text-muted text-xs ml-2">Click to change</span>
        </button>
      )}
    </div>
  );
}
