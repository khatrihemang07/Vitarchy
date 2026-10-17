//! What an Insight is: a kind, a subject, a few numbers and a notability score.
//! The sentence is written by the web from a template per kind, so it follows the
//! person's clock and date settings.

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

/// The kind of statement an Insight makes.
#[derive(Debug, Clone, Copy, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum InsightKind {
    /// A Step or description usually starts or ends at about the same time.
    TypicalTime,
    /// Something usually takes about the same length of time.
    TypicalDuration,
    /// One weekday differs from the rest.
    WeekdayDifference,
    /// The last four weeks differ from the four before.
    Change,
    /// One Day is far from the usual.
    Outlier,
}

/// What an Insight is about.
#[derive(Debug, Clone, Copy, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    /// A Step of a saved Routine (named by its label).
    RoutineStep,
    /// A whole saved Routine, door to door (named by the Routine).
    RoutineTotal,
    /// A frequent Time Entry description.
    Description,
    /// An app's screen time.
    App,
    /// A Context's screen time.
    Context,
}

/// The saved Routine a Step or a total belongs to.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoutineRef {
    /// Its identifier, which tells apart two Routines with the same name or Step labels.
    pub id: i64,
    /// Its name.
    pub name: String,
}

/// The thing an Insight is about.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct Subject {
    /// What sort of thing it is.
    pub kind: SubjectKind,
    /// Its name as the person knows it.
    pub name: String,
    /// The Routine it belongs to (for a Routine Step or a Routine's total).
    pub routine: Option<RoutineRef>,
}

/// Which quantity of the subject the Insight measures.
#[derive(Debug, Clone, Copy, Serialize, ToSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    /// When it starts, as a time of day.
    Start,
    /// When it ends, as a time of day.
    End,
    /// How long it lasts (screen time: per Day).
    Length,
}

/// The figures a sentence is built from. Every figure is in whole minutes; a time of day is
/// minutes after local midnight.
#[derive(Debug, Clone, Default, Serialize, ToSchema, PartialEq, Eq)]
pub struct Numbers {
    /// The figure the sentence is about: the usual value, one weekday's usual value, the
    /// last four weeks' usual value, or the outlying Day's value.
    pub value: i64,
    /// First quartile (typical times only).
    pub p25: Option<i64>,
    /// Third quartile (typical times only).
    pub p75: Option<i64>,
    /// What `value` is compared with: all Days, the four weeks before, or the usual.
    pub reference: Option<i64>,
    /// `value` minus `reference`, going the short way round the clock for a time of day.
    pub delta: Option<i64>,
    /// Weekday, 0 for Monday to 6 for Sunday (weekday differences only).
    pub weekday: Option<u8>,
    /// Days measured.
    pub samples: i64,
}

/// One plain statement found in the person's own data.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq)]
pub struct Insight {
    /// The kind of statement.
    pub kind: InsightKind,
    /// What it is about.
    pub subject: Subject,
    /// Which quantity of the subject.
    pub measure: Measure,
    /// The figures.
    pub numbers: Numbers,
    /// How notable it is (see `score.rs`); higher is more notable, and the list is ranked by it.
    pub score: f64,
    /// Days it refers to (the outlying Day), earliest first.
    #[schema(value_type = Vec<String>, format = Date)]
    pub days: Vec<NaiveDate>,
}

impl Insight {
    /// An Insight about `series`, scored from the strength (`effect`, 0 to 1) of what was found
    /// over `numbers.samples` Days.
    #[must_use]
    pub fn found(
        kind: InsightKind,
        series: &super::inputs::Series,
        numbers: Numbers,
        effect: f64,
        days: Vec<NaiveDate>,
    ) -> Self {
        let samples = usize::try_from(numbers.samples).unwrap_or(0);
        Self {
            kind,
            subject: series.subject.clone(),
            measure: series.measure,
            score: super::score::score(kind, effect, samples),
            numbers,
            days,
        }
    }
}
