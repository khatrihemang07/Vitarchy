import { PASSWORDS_CARD_ID, PASSWORD_LABEL_FIELD_ID } from "./passwordsAnchor";

/** A link to the Statement passwords card that scrolls to it and focuses its first field. */
export function PasswordsLink() {
  return (
    <>
      {" "}
      <a
        href={`#${PASSWORDS_CARD_ID}`}
        className="font-medium text-foreground underline underline-offset-4 focus-ring"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById(PASSWORDS_CARD_ID)?.scrollIntoView({ block: "center" });
          document.getElementById(PASSWORD_LABEL_FIELD_ID)?.focus();
        }}
      >
        Add a Statement password
      </a>
    </>
  );
}
