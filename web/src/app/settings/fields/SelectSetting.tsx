import {
  SegmentedControl,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/design";
import type { FieldProps } from "./types";
import { usesSegmentedControl } from "./usesSegmentedControl";

export function SelectSetting({ field, value, onChange, id }: FieldProps<"select">) {
  if (usesSegmentedControl(field.options)) {
    return (
      <SegmentedControl
        id={id}
        label={field.label}
        value={value}
        onValueChange={onChange}
        options={field.options}
      />
    );
  }
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id} aria-label={field.label} className="w-full md:w-80">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {field.options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
