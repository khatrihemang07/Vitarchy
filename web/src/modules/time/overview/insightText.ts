/** The sentence for each kind of Insight, written from its figures. Pure. */
import type { Insight, InsightKind, InsightSubjectKind } from "@/api/time";
import { formatDayLong } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import { type Clock, formatMinuteOfDay } from "../shared/timeOfDay";

const RANGE_SEPARATOR = " – ";
const WEEKDAYS = ["Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays", "Sundays"] as const;

/** What a sentence needs besides the Insight itself. */
interface Words {
  insight: Insight;
  clock: Clock;
}

const span = (minutes: number) => formatSpan(minutes * 60);
const clockTime = ({ clock }: Words, minute: number) => formatMinuteOfDay(minute, clock);
const range = (words: Words) => {
  const { p25, p75 } = words.insight.numbers;
  return p25 !== undefined && p25 !== null && p75 !== undefined && p75 !== null ? ` (${clockTime(words, p25)}${RANGE_SEPARATOR}${clockTime(words, p75)})` : "";
};
const lengthRange = ({ insight }: Words) => {
  const { p25, p75 } = insight.numbers;
  return p25 !== undefined && p25 !== null && p75 !== undefined && p75 !== null ? ` (${span(p25)}${RANGE_SEPARATOR}${span(p75)})` : "";
};
const delta = ({ insight }: Words) => insight.numbers.delta ?? 0;
const isClock = ({ insight }: Words) => insight.measure !== "length";
const verb = ({ insight }: Words) => (insight.measure === "start" ? "start" : "finish");

/** How each kind of subject is talked about when a length is the subject of the sentence. */
interface LengthPhrases {
  /** "Office usually takes 32 min". */
  usually: (name: string, amount: string) => string;
  /** "Office takes 12 min less", in the past or the present. */
  gap: (name: string, amount: string, more: boolean, past: boolean, startsSentence: boolean) => string;
  /** The label before an extreme Day's date. */
  extreme: (name: string, longest: boolean) => string;
}

const taking: LengthPhrases = {
  usually: (name, amount) => `${name} usually takes ${amount}`,
  gap: (name, amount, more, past) => `${name} ${past ? "took" : "takes"} ${amount} ${more ? "longer" : "less"}`,
  extreme: (name, longest) => `${longest ? "Longest" : "Shortest"} ${name} day`,
};

const LENGTH_PHRASES: Record<InsightSubjectKind, LengthPhrases> = {
  routine_step: taking,
  description: taking,
  routine_total: { ...taking, usually: (name, amount) => `${name} usually takes ${amount} door to door` },
  app: screenTime(),
  context: screenTime(),
};

function screenTime(): LengthPhrases {
  return {
    usually: (name, amount) => `You usually spend ${amount} a day in ${name}`,
    gap: (name, amount, more, past, startsSentence) => `${startsSentence ? "You" : "you"} ${past ? "spent" : "spend"} ${amount} ${more ? "more" : "less"} a day in ${name}`,
    extreme: (name, longest) => `${longest ? "Most" : "Least"} time in ${name}`,
  };
}

const phrases = ({ insight }: Words) => LENGTH_PHRASES[insight.subject.kind];
const name = ({ insight }: Words) => insight.subject.name;

function typicalTime(words: Words): string {
  return `You usually ${verb(words)} ${name(words)} at ${clockTime(words, words.insight.numbers.value)}${range(words)}`;
}

function typicalDuration(words: Words): string {
  return phrases(words).usually(name(words), span(words.insight.numbers.value)) + lengthRange(words);
}

function weekdayDifference(words: Words): string {
  const weekday = `On ${WEEKDAYS[words.insight.numbers.weekday ?? 0] ?? "that weekday"}`;
  const change = delta(words);
  const amount = span(Math.abs(change));
  if (isClock(words)) {
    return `${weekday} you ${verb(words)} ${name(words)} ${amount} ${change < 0 ? "earlier" : "later"} than usual`;
  }
  return `${weekday}, ${phrases(words).gap(name(words), amount, change > 0, false, false)} than usual`;
}

function change(words: Words): string {
  const amount = delta(words);
  return `${phrases(words).gap(name(words), span(Math.abs(amount)), amount > 0, true, true)} than in the previous 4 weeks`;
}

const CLOCK_EXTREMES = {
  start: ["Earliest start", "Latest start"],
  end: ["Earliest finish", "Latest finish"],
} as const;

function outlier(words: Words): string {
  const { insight } = words;
  const later = delta(words) > 0;
  const day = insight.days[0] !== undefined ? formatDayLong(insight.days[0]) : "";
  const { value, reference } = insight.numbers;
  if (insight.measure === "length") {
    return `${phrases(words).extreme(name(words), later)}: ${day}, ${span(value)}`;
  }
  const [earliest, latest] = CLOCK_EXTREMES[insight.measure];
  const usual = reference !== undefined && reference !== null ? ` (usually ${clockTime(words, reference)})` : "";
  return `${later ? latest : earliest} of ${name(words)}: ${day}, ${clockTime(words, value)}${usual}`;
}

const TEMPLATES: Record<InsightKind, (words: Words) => string> = {
  typical_time: typicalTime,
  typical_duration: typicalDuration,
  weekday_difference: weekdayDifference,
  change,
  outlier,
};

/** The sentence for `insight`, with times written on the person's `clock`. */
export function describeInsight(insight: Insight, clock: Clock): string {
  return TEMPLATES[insight.kind]({ insight, clock });
}
