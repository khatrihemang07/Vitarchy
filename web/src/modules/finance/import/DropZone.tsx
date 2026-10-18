import { Upload } from "lucide-react";
import { useRef, useState } from "react";
import { cn, MotionButton } from "@/design";

/** The file types a Statement can be. */
export const ACCEPTED_FILES = ".pdf,.csv,.xls,.xlsx";

interface DropZoneProps {
  onFiles: (files: File[]) => void;
  disabled: boolean;
}

/** A drop target with a file picker for several Statement files at once. */
export function DropZone({ onFiles, disabled }: DropZoneProps) {
  const input = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);
  const send = (list: FileList | null) => {
    const files = Array.from(list ?? []);
    if (files.length > 0 && !disabled) onFiles(files);
  };
  return (
    <div
      data-over={over}
      onDragOver={(event) => { event.preventDefault(); setOver(true); }}
      onDragLeave={() => { setOver(false); }}
      onDrop={(event) => {
        event.preventDefault();
        setOver(false);
        send(event.dataTransfer.files);
      }}
      className={cn(
        "flex flex-col items-center gap-3 rounded-xl border-2 border-dashed p-8 text-center transition-colors",
        over ? "border-module bg-module/10" : "border-border",
      )}
    >
      <span className="grid size-12 place-items-center rounded-full bg-module/15 text-module">
        <Upload aria-hidden="true" className="size-6" />
      </span>
      <div>
        <p className="font-display text-lg font-semibold">Drop Statements here</p>
        <p className="text-sm text-muted-foreground">PDF, CSV, XLS or XLSX. You can choose several files at once.</p>
      </div>
      <input
        ref={input}
        type="file"
        multiple
        hidden
        accept={ACCEPTED_FILES}
        aria-label="Statement files"
        onChange={(event) => {
          send(event.target.files);
          event.target.value = "";
        }}
      />
      <MotionButton disabled={disabled} onClick={() => { input.current?.click(); }}>
        Choose files
      </MotionButton>
    </div>
  );
}
