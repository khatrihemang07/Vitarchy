import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { SettingField, SettingValue } from "../schema";
import { FieldControl } from "./FieldControl";

function setup(field: SettingField, value: SettingValue) {
  const onChange = vi.fn();
  render(<FieldControl field={field} value={value} onChange={onChange} id="f" />);
  return { onChange, user: userEvent.setup() };
}

const segmented: SettingField = {
  kind: "select",
  key: "theme",
  label: "Theme",
  default: "system",
  options: [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ],
};

describe("select", () => {
  it("shows few options as a radio group and saves the chosen value", async () => {
    const { onChange, user } = setup(segmented, "system");
    expect(screen.getByRole("radio", { name: "System" })).toBeChecked();
    await user.click(screen.getByRole("radio", { name: "Dark" }));
    expect(onChange).toHaveBeenCalledWith("dark");
  });

  it("shows many options as a dropdown and saves the chosen value", async () => {
    const many: SettingField = {
      kind: "select",
      key: "n",
      label: "Pick",
      default: "a",
      options: ["a", "b", "c", "d", "e"].map((v) => ({ value: v, label: v.toUpperCase() })),
    };
    const { onChange, user } = setup(many, "a");
    await user.click(screen.getByRole("combobox", { name: "Pick" }));
    await user.click(await screen.findByRole("option", { name: "D" }));
    expect(onChange).toHaveBeenCalledWith("d");
  });
});

describe("toggle", () => {
  it("saves the flipped value", async () => {
    const { onChange, user } = setup({ kind: "toggle", key: "t", label: "T", default: false }, false);
    await user.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});

describe("text", () => {
  const field: SettingField = { kind: "text", key: "n", label: "Name", default: "", maxLength: 5 };

  it("saves on blur, not while typing", async () => {
    const { onChange, user } = setup(field, "");
    const input = screen.getByRole("textbox");
    await user.type(input, "Ada");
    expect(onChange).not.toHaveBeenCalled();
    await user.tab();
    expect(onChange).toHaveBeenCalledExactlyOnceWith("Ada");
  });

  it("saves on Enter", async () => {
    const { onChange, user } = setup(field, "");
    await user.type(screen.getByRole("textbox"), "Bo{Enter}");
    expect(onChange).toHaveBeenCalledExactlyOnceWith("Bo");
  });

  it("does not save an unchanged value", async () => {
    const { onChange, user } = setup(field, "Ada");
    await user.click(screen.getByRole("textbox"));
    await user.tab();
    expect(onChange).not.toHaveBeenCalled();
  });

  it("trims before saving", async () => {
    const { onChange, user } = setup(field, "");
    await user.type(screen.getByRole("textbox"), "  Ada  {Enter}");
    expect(onChange).toHaveBeenCalledExactlyOnceWith("Ada");
    expect(screen.getByRole("textbox")).toHaveValue("Ada");
  });

  it("counts emoji as one character each, like the server", async () => {
    const { onChange, user } = setup(field, "");
    await user.type(screen.getByRole("textbox"), "😀😀😀😀😀😀");
    expect(screen.getByRole("textbox")).toHaveValue("😀😀😀😀😀");
    await user.tab();
    expect(onChange).toHaveBeenCalledExactlyOnceWith("😀😀😀😀😀");
  });

  it("respects maxLength", async () => {
    const { user } = setup(field, "");
    await user.type(screen.getByRole("textbox"), "abcdefgh");
    expect(screen.getByRole("textbox")).toHaveValue("abcde");
  });
});

describe("number", () => {
  const field: SettingField = { kind: "number", key: "n", label: "N", default: 3, min: 1, max: 10 };

  it("saves a valid number on blur", async () => {
    const { onChange, user } = setup(field, 3);
    const input = screen.getByRole("spinbutton");
    await user.clear(input);
    await user.type(input, "7");
    await user.tab();
    expect(onChange).toHaveBeenCalledExactlyOnceWith(7);
  });

  it("clamps to the bounds", async () => {
    const { onChange, user } = setup(field, 3);
    const input = screen.getByRole("spinbutton");
    await user.clear(input);
    await user.type(input, "99");
    await user.tab();
    expect(onChange).toHaveBeenCalledExactlyOnceWith(10);
  });

  it("reverts when the text is not a number", async () => {
    const { onChange, user } = setup(field, 3);
    const input = screen.getByRole("spinbutton");
    await user.clear(input);
    await user.tab();
    expect(onChange).not.toHaveBeenCalled();
    expect(input).toHaveValue(3);
  });
});
