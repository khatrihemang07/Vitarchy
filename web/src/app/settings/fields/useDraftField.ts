import { type KeyboardEvent, useState } from "react";

interface DraftFieldOptions<V> {
  /** The saved value. The draft follows it whenever it changes (a save landing, or a rollback). */
  value: V;
  format: (value: V) => string;
  /** The value the typed text means, or undefined if it means nothing (the draft then reverts). */
  parse: (text: string) => V | undefined;
  /** Called only when the committed value differs from `value`. */
  onCommit: (value: V) => void;
  /** Cleans text as it is typed (e.g. a length limit). */
  sanitize?: (text: string) => string;
}

/**
 * Draft-then-commit behaviour for typed fields: edits stay local, and commit
 * when the user leaves the field or presses Enter. Returns the props to
 * spread onto the input.
 */
export function useDraftField<V>({ value, format, parse, onCommit, sanitize }: DraftFieldOptions<V>) {
  const saved = format(value);
  const [draft, setDraft] = useState(saved);
  const [seen, setSeen] = useState(saved);
  if (saved !== seen) {
    setSeen(saved);
    setDraft(saved);
  }

  const commit = () => {
    const next = parse(draft);
    if (next === undefined) {
      setDraft(saved);
      return;
    }
    setDraft(format(next));
    if (next !== value) onCommit(next);
  };

  return {
    value: draft,
    onChange: (event: { target: { value: string } }) => {
      setDraft(sanitize ? sanitize(event.target.value) : event.target.value);
    },
    onBlur: commit,
    onKeyDown: (event: KeyboardEvent<HTMLInputElement>) => {
      if (event.key === "Enter") event.currentTarget.blur();
    },
  };
}
