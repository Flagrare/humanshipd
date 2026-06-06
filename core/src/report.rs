//! Banded provenance report (signals spec §5).
//!
//! Renders a record's ordered spans into word-count *proportions* of how text
//! entered the document — **not** a confidence score that the text "is AI." Bands
//! describe verified provenance (typed / pasted / AI-tool / unknown); the
//! denominator is all observed authoring activity plus any final-document text the
//! tool never saw enter (the `Unknown` band). This mirrors Grammarly's
//! word-count-proportion model while staying inside the honest provenance envelope.

use crate::record::{Provenance, WritingSessionRecord};
use serde::{Deserialize, Serialize};

/// Minimum document length before process-shape is even assessed (signals spec §6,
/// Tier 2). Below this there is too little material to say anything.
const PROCESS_SHAPE_MIN_CHARS: u64 = 80;
/// Planning pauses needed to count the pause-rhythm signal as present.
const PROCESS_SHAPE_MIN_PAUSES: u64 = 2;
/// Distinct writing bursts needed to count the segmentation signal as present.
const PROCESS_SHAPE_MIN_BURSTS: u64 = 3;

/// One row of the report: a provenance class, its character count, and its share.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportBand {
    pub provenance: Provenance,
    pub chars: u64,
    /// Share of `ProvenanceReport.total_chars`, in `[0.0, 1.0]`.
    pub fraction: f64,
}

/// Coarse, descriptive summary of how a document was produced (signals spec §5).
/// Descriptive, never accusatory — it characterizes provenance, not honesty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NuanceSummary {
    /// ~All observed text was typed in a tracked editor.
    FullyTyped,
    /// Predominantly typed, with some pasted spans.
    TypedWithPastes,
    /// Predominantly arrived by paste.
    MostlyPasted,
    /// Substantial content was never observed entering (written outside coverage).
    Unverified,
}

/// The banded provenance report for a record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceReport {
    pub typed_chars: u64,
    pub pasted_chars: u64,
    pub ai_tool_chars: u64,
    /// Final-document characters not observed entering via capture (pre-existing or
    /// written outside a tracked surface).
    pub unknown_chars: u64,
    /// Denominator for every band's fraction.
    pub total_chars: u64,
    /// Non-empty bands, largest first.
    pub bands: Vec<ReportBand>,
    pub summary: NuanceSummary,
}

/// Render the banded provenance report from a signed record's spans.
pub fn render_report(record: &WritingSessionRecord) -> ProvenanceReport {
    let mut typed_chars = 0u64;
    let mut pasted_chars = 0u64;
    let mut ai_tool_chars = 0u64;
    for span in &record.process.spans {
        match span.provenance {
            Provenance::Typed => typed_chars += span.chars,
            Provenance::Pasted => pasted_chars += span.chars,
            Provenance::AiTool => ai_tool_chars += span.chars,
            // An `Unknown` span (should the builder ever emit one) is folded into
            // the document-derived unknown total below, not double-counted here.
            Provenance::Unknown => {}
        }
    }

    let observed = typed_chars + pasted_chars + ai_tool_chars;
    let unknown_chars = record.document_binding.char_count.saturating_sub(observed);
    let total_chars = observed + unknown_chars;

    let bands = build_bands(total_chars, typed_chars, pasted_chars, ai_tool_chars, unknown_chars);
    let summary = summarize(total_chars, typed_chars, pasted_chars + ai_tool_chars, unknown_chars);

    ProvenanceReport {
        typed_chars,
        pasted_chars,
        ai_tool_chars,
        unknown_chars,
        total_chars,
        bands,
        summary,
    }
}

fn build_bands(
    total: u64,
    typed: u64,
    pasted: u64,
    ai_tool: u64,
    unknown: u64,
) -> Vec<ReportBand> {
    let fraction = |chars: u64| if total > 0 { chars as f64 / total as f64 } else { 0.0 };
    let mut bands: Vec<ReportBand> = [
        (Provenance::Typed, typed),
        (Provenance::Pasted, pasted),
        (Provenance::AiTool, ai_tool),
        (Provenance::Unknown, unknown),
    ]
    .into_iter()
    .filter(|(_, chars)| *chars > 0)
    .map(|(provenance, chars)| ReportBand {
        provenance,
        chars,
        fraction: fraction(chars),
    })
    .collect();
    bands.sort_by_key(|b| std::cmp::Reverse(b.chars));
    bands
}

fn summarize(total: u64, typed: u64, non_typed: u64, unknown: u64) -> NuanceSummary {
    if total == 0 || unknown * 2 >= total {
        return NuanceSummary::Unverified;
    }
    if non_typed * 2 > total {
        return NuanceSummary::MostlyPasted;
    }
    if non_typed > 0 {
        return NuanceSummary::TypedWithPastes;
    }
    let _ = typed;
    NuanceSummary::FullyTyped
}

// ---- Process-shape corroboration (Tier 2, positive-only) -------------------
//
// A deliberately humble, secondary signal (signals spec §3 Tier 2 / §6). It can
// *corroborate* a human-like drafting rhythm — planning pauses, revisions, and
// writing arriving in bursts rather than one block — but its ABSENCE is never
// evidence of AI: a careful, fluent typist also writes cleanly. So the assessment
// is only ever `IncrementalComposition` (corroborated) or `Inconclusive` — there is
// no "looks like AI" verdict, by design. The underlying signal is weak and
// spoofable (keystroke-process humanness shows ~18–48% error rates in the
// literature), so callers must present it as supporting context, never proof.

/// The qualitative process-shape outcome. Never accuses; only corroborates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessAssessment {
    /// Multiple human-drafting signals are present (weak positive corroboration).
    IncrementalComposition,
    /// Too little signal to corroborate — NOT evidence of AI.
    Inconclusive,
}

/// Weak, content-free corroboration of a human-like writing process (Tier 2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessShape {
    /// Planning pauses were present in the session.
    pub pause_rhythm: bool,
    /// The writer revised — deletions or in-place reformulations.
    pub revision_activity: bool,
    /// Text arrived in several bursts rather than one uninterrupted block.
    pub burst_segmentation: bool,
    /// How many of the three signals are present (0–3).
    pub signals_present: u8,
    pub assessment: ProcessAssessment,
}

/// Derive the positive-only process-shape corroboration from a record's existing
/// content-free stats. No new capture; never emits an AI verdict.
pub fn render_process_shape(record: &WritingSessionRecord) -> ProcessShape {
    let p = &record.process;
    let pause_rhythm = p.pauses.gt_2s >= PROCESS_SHAPE_MIN_PAUSES;
    let revision_activity = p.revisions.deletions >= 1 || p.revisions.reformulations >= 1;
    let burst_segmentation = p.bursts.count >= PROCESS_SHAPE_MIN_BURSTS;
    let signals_present =
        [pause_rhythm, revision_activity, burst_segmentation].iter().filter(|b| **b).count() as u8;

    // Require a minimum of material AND at least two corroborating signals before
    // claiming incremental composition; otherwise stay Inconclusive (never "AI").
    let enough_material = record.document_binding.char_count >= PROCESS_SHAPE_MIN_CHARS;
    let assessment = if enough_material && signals_present >= 2 {
        ProcessAssessment::IncrementalComposition
    } else {
        ProcessAssessment::Inconclusive
    };

    ProcessShape { pause_rhythm, revision_activity, burst_segmentation, signals_present, assessment }
}
