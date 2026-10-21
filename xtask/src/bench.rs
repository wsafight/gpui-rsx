//! Check Criterion's relative-change confidence intervals against a regression budget.
use std::path::{Path, PathBuf};

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut threshold = 0.10;
    let mut directory = std::env::var_os("CRITERION_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("target"))
                .join("criterion")
        });
    directory = root.join(directory);
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--threshold" => {
                threshold = iter
                    .next()
                    .ok_or("--threshold needs a value")?
                    .parse::<f64>()
                    .map_err(|_| "invalid threshold")?;
                if !threshold.is_finite() || threshold < 0.0 {
                    return Err("threshold must be a finite non-negative fraction".into());
                }
            }
            "--criterion-dir" => {
                directory = PathBuf::from(iter.next().ok_or("--criterion-dir needs a value")?)
            }
            other => return Err(format!("unknown bench-check option `{other}`")),
        }
    }
    let mut files = Vec::new();
    collect_changes(&directory, &mut files)?;
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "no Criterion comparisons in {}; run cargo bench with --save-baseline before running with --baseline",
            directory.display()
        ));
    }
    let mut regressions = Vec::new();
    for file in &files {
        let source =
            std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let (estimate, lower) =
            relative_change(&source).map_err(|e| format!("{}: {e}", file.display()))?;
        let name = file
            .strip_prefix(&directory)
            .unwrap_or(file)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .display();
        eprintln!(
            "{name}: {:+.2}% (lower confidence bound {:+.2}%)",
            estimate * 100.0,
            lower * 100.0
        );
        if lower > threshold {
            regressions.push(format!(
                "{name}: {:+.2}% (lower bound {:+.2}%)",
                estimate * 100.0,
                lower * 100.0
            ));
        }
    }
    if regressions.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "benchmark regression exceeds {:.1}%:\n{}",
            threshold * 100.0,
            regressions.join("\n")
        ))
    }
}

fn collect_changes(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        // Do not follow symlinks while scanning benchmark outputs.
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            collect_changes(&entry.path(), files)?;
        } else if entry.file_name() == "estimates.json"
            && dir.file_name().is_some_and(|n| n == "change")
        {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn relative_change(source: &str) -> Result<(f64, f64), String> {
    let json: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    let mean = &json["mean"];
    let estimate = mean["point_estimate"]
        .as_f64()
        .ok_or("missing mean.point_estimate")?;
    let lower = mean["confidence_interval"]["lower_bound"]
        .as_f64()
        .ok_or("missing mean confidence lower bound")?;
    let upper = mean["confidence_interval"]["upper_bound"]
        .as_f64()
        .ok_or("missing mean confidence upper bound")?;
    if !estimate.is_finite() || !lower.is_finite() || !upper.is_finite() || lower > upper {
        return Err("invalid mean confidence interval".into());
    }
    Ok((estimate, lower))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_relative_changes_and_rejects_absolute_or_missing_estimates() {
        assert_eq!(relative_change(r#"{"mean":{"point_estimate":0.2,"confidence_interval":{"lower_bound":0.12,"upper_bound":0.28}}}"#).unwrap(), (0.2, 0.12));
        assert!(relative_change(r#"{"slope":{"point_estimate":120}}"#).is_err());
        assert!(relative_change(r#"{"mean":{"point_estimate":0.2,"confidence_interval":{"lower_bound":0.3,"upper_bound":0.1}}}"#).is_err());
    }

    #[test]
    fn rejects_invalid_thresholds_before_reading_results() {
        for value in ["-1", "NaN", "inf", "oops"] {
            assert!(
                run(
                    Path::new("/nonexistent"),
                    &["--threshold".into(), value.into()]
                )
                .is_err()
            );
        }
    }

    #[test]
    fn fails_regressions_but_accepts_noise_and_improvements() {
        let directory = std::env::temp_dir().join(format!("rsx-bench-test-{}", std::process::id()));
        let change = directory.join("example/change");
        std::fs::create_dir_all(&change).unwrap();
        for (point, lower, upper, fails) in [
            (0.20, 0.12, 0.28, true),
            (0.20, 0.08, 0.32, false),
            (-0.20, -0.25, -0.15, false),
        ] {
            let data = serde_json::json!({"mean": {"point_estimate": point, "confidence_interval": {"lower_bound": lower, "upper_bound": upper}}});
            std::fs::write(change.join("estimates.json"), data.to_string()).unwrap();
            let result = run(
                Path::new("."),
                &[
                    "--criterion-dir".into(),
                    directory.to_string_lossy().into_owned(),
                ],
            );
            assert_eq!(result.is_err(), fails, "{result:?}");
        }
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
