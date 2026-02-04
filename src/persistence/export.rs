use crate::simulation::AggregatedResults;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_results_json(results: &AggregatedResults, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(results)
        .map_err(|e| format!("Failed to serialize results: {}", e))?;

    std::fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

pub fn export_results_csv(results: &AggregatedResults, path: &Path) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

    // Write header
    writeln!(file, "Metric,P5,P25,Median,P75,P95,Mean,StdDev,Min,Max")
        .map_err(|e| format!("Failed to write: {}", e))?;

    // Write stats
    write_stats_row(&mut file, "Effective Monthly Cost", &results.effective_monthly_stats)?;
    write_stats_row(&mut file, "Effective Total Cost", &results.effective_cost_stats)?;
    write_stats_row(&mut file, "Final Equity", &results.equity_stats)?;

    // Write raw values section
    writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file, "Simulation,Effective Monthly,Effective Total,Final Equity")
        .map_err(|e| format!("Failed to write: {}", e))?;

    for (i, ((monthly, total), equity)) in results
        .effective_monthly_values
        .iter()
        .zip(results.effective_cost_values.iter())
        .zip(results.equity_values.iter())
        .enumerate()
    {
        writeln!(file, "{},{:.2},{:.2},{:.2}", i + 1, monthly, total, equity)
            .map_err(|e| format!("Failed to write: {}", e))?;
    }

    Ok(())
}

fn write_stats_row(
    file: &mut File,
    name: &str,
    stats: &crate::simulation::PercentileStats,
) -> Result<(), String> {
    writeln!(
        file,
        "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
        name,
        stats.p5,
        stats.p25,
        stats.p50,
        stats.p75,
        stats.p95,
        stats.mean,
        stats.std_dev,
        stats.min,
        stats.max
    )
    .map_err(|e| format!("Failed to write: {}", e))
}

pub fn export_results_json_dialog(results: &AggregatedResults) -> Option<Result<(), String>> {
    let file = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .set_file_name("simulation_results.json")
        .save_file()?;

    Some(export_results_json(results, &file))
}

pub fn export_results_csv_dialog(results: &AggregatedResults) -> Option<Result<(), String>> {
    let file = rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name("simulation_results.csv")
        .save_file()?;

    Some(export_results_csv(results, &file))
}
