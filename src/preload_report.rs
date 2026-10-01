// SPDX-License-Identifier: MIT

//! Shared result wording for directory walks and rclone recursive refreshes.

#[derive(Debug, Clone, PartialEq)]
pub struct PreloadReport {
    pub duration_seconds: f64,
    pub limit_seconds: f64,
    pub maximum_depth: Option<u8>,
    pub excluded_paths: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreloadOutcome {
    Completed(PreloadReport),
    TimedOut(PreloadReport),
    Skipped,
    Cancelled,
}

impl PreloadOutcome {
    pub fn notice(&self, name: &str) -> Option<String> {
        let status = match self {
            Self::Completed(report) => match report.maximum_depth {
                Some(depth) => format!("complete through depth {depth}"),
                None => "complete".into(),
            },
            Self::TimedOut(_) => "time limit reached; scan incomplete".into(),
            Self::Skipped => "skipped (excluded system directory)".into(),
            Self::Cancelled => return None,
        };
        Some(format!("{name}: preload {status}."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> PreloadReport {
        PreloadReport {
            duration_seconds: 62.0,
            limit_seconds: 60.0,
            maximum_depth: Some(3),
            excluded_paths: 3,
        }
    }

    #[test]
    fn timeout_is_brief_and_omits_timing_details() {
        let text = PreloadOutcome::TimedOut(report()).notice("VPS").unwrap();
        assert_eq!(text, "VPS: preload time limit reached; scan incomplete.");
    }

    #[test]
    fn completion_describes_bounded_and_recursive_scopes() {
        let mut report = report();
        let text = PreloadOutcome::Completed(report.clone())
            .notice("SMB")
            .unwrap();
        assert_eq!(text, "SMB: preload complete through depth 3.");
        report.maximum_depth = None;
        report.excluded_paths = 0;
        let text = PreloadOutcome::Completed(report)
            .notice("Google Drive")
            .unwrap();
        assert_eq!(text, "Google Drive: preload complete.");
        assert!(!text.contains("exclusions"));
    }

    #[test]
    fn skipped_is_not_completion_and_cancellation_stays_silent() {
        let text = PreloadOutcome::Skipped.notice("VPS").unwrap();
        assert!(text.contains("skipped"));
        assert!(!text.contains("completed"));
        assert_eq!(PreloadOutcome::Cancelled.notice("VPS"), None);
    }
}
