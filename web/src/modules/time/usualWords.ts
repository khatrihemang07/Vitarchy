/** How the Usual is named in the UI, written once: the Time pages, the toggle, tooltips and notes all derive from it. */
export const USUAL_SPAN = "last 4 weeks";

/** The span as a label ("Last 4 weeks"). */
export const USUAL_LABEL = USUAL_SPAN.charAt(0).toUpperCase() + USUAL_SPAN.slice(1);

/** The span inside a sentence ("the last 4 weeks"). */
export const USUAL_PHRASE = `the ${USUAL_SPAN}`;
