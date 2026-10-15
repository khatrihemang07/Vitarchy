import { NumberSetting } from "./NumberSetting";
import { SelectSetting } from "./SelectSetting";
import { TextSetting } from "./TextSetting";
import { ToggleSetting } from "./ToggleSetting";
import type { FieldRenderers } from "./types";

/** One renderer per field kind. A new kind will not compile until it is added here. */
export const fieldRenderers: FieldRenderers = {
  select: SelectSetting,
  toggle: ToggleSetting,
  text: TextSetting,
  number: NumberSetting,
};
