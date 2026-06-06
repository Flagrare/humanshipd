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
