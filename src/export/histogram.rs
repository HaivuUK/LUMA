use crate::integrate::group_moduli;
use crate::params::Params;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct HistogramBucket {
    pub value: f64,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistogramBinning {
    pub gap_value: Option<f64>,
    pub num_materials: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistogramData {
    pub total: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub binning: HistogramBinning,
    pub buckets: Vec<HistogramBucket>,
}

#[derive(Debug, Clone, Default)]
pub struct HistogramOverrides {
    pub export: Option<bool>,
    pub view: Option<bool>,
    pub export_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HistogramOptions {
    pub export: bool,
    pub view: bool,
    pub export_dir: Option<String>,
}

impl HistogramOptions {
    pub fn from_params(params: &Params, overrides: Option<HistogramOverrides>) -> Self {
        let mut options = HistogramOptions {
            export: params.histogram_export,
            view: true, // Always enable view if visualizer is used
            export_dir: params.histogram_dir.clone(),
        };

        if let Some(override_cfg) = overrides {
            if let Some(export) = override_cfg.export {
                options.export = export;
            }
            if let Some(view) = override_cfg.view {
                options.view = view;
            }
            if override_cfg.export_dir.is_some() {
                options.export_dir = override_cfg.export_dir;
            }
        }

        options
    }
}

pub fn build_material_histogram(raw_moduli: &[f64], params: &Params) -> HistogramData {
    let grouped_moduli = group_moduli(raw_moduli, params);
    build_histogram_from_grouped(&grouped_moduli, params.gap_value, params.num_materials)
}

pub fn build_assigned_material_histogram(moduli: &[f64]) -> HistogramData {
    // If it's already assigned, the moduli are already grouped.
    // Count exact distinct values or round them slightly to avoid float precision issues.
    build_histogram_from_grouped(moduli, None, None)
}

fn build_histogram_from_grouped(grouped_moduli: &[f64], gap_value: Option<f64>, num_materials: Option<usize>) -> HistogramData {
    let mut map: BTreeMap<i64, (f64, usize)> = BTreeMap::new();

    for &modulus in grouped_moduli {
        let key = (modulus * 1e9).round() as i64;
        let entry = map.entry(key).or_insert((modulus, 0));
        entry.1 += 1;
    }

    let total = map.values().map(|(_, count)| *count).sum::<usize>();
    let mut buckets = Vec::with_capacity(map.len());

    for (_, (value, count)) in map {
        let percentage = if total > 0 {
            (count as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        buckets.push(HistogramBucket {
            value,
            count,
            percentage,
        });
    }

    let (min, max) = if buckets.is_empty() {
        (0.0, 0.0)
    } else {
        let min = buckets
            .iter()
            .map(|b| b.value)
            .fold(f64::INFINITY, f64::min);
        let max = buckets
            .iter()
            .map(|b| b.value)
            .fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    };

    let  weighted_sum = buckets.iter().map(|b| b.value * b.count as f64).sum::<f64>();
    let mean = if total > 0 {
        weighted_sum / total as f64
    } else {
        0.0
    };

    HistogramData {
        total,
        min,
        max,
        mean,
        binning: HistogramBinning {
            gap_value,
            num_materials,
        },
        buckets,
    }
}

pub fn histogram_output_paths(output_mesh_path: &str, export_dir: Option<&str>) -> Result<(PathBuf, PathBuf)> {
    let mesh_path = Path::new(output_mesh_path);
    let base_name = mesh_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("histogram");

    let output_dir = if let Some(dir) = export_dir {
        PathBuf::from(dir)
    } else {
        mesh_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    };

    let csv_path = output_dir.join(format!("{}_histogram.csv", base_name));
    let json_path = output_dir.join(format!("{}_histogram.json", base_name));
    Ok((csv_path, json_path))
}

pub fn write_histogram_outputs(output_mesh_path: &str, export_dir: Option<&str>, histogram: &HistogramData) -> Result<(PathBuf, PathBuf)> {
    let (csv_path, json_path) = histogram_output_paths(output_mesh_path, export_dir)?;
    if let Some(parent) = csv_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut csv = String::from("value,count,percentage\n");
    for bucket in &histogram.buckets {
        csv.push_str(&format!("{:.6},{},{}\n", bucket.value, bucket.count, bucket.percentage));
    }
    std::fs::write(&csv_path, csv)?;

    let json = serde_json::to_string_pretty(histogram)?;
    std::fs::write(&json_path, json)?;

    Ok((csv_path, json_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Params;
    use crate::volume::image_nrrd::MetadataValue::String;

    fn sample_params() -> Params {
        Params {
            integration: None,
            gap_value: Some(1.0),
            num_materials: None,
            grouping_density: "mean".to_string(),
            int_steps: 1,
            rho_qct_a: 0.0,
            rho_qct_b: 1.0,
            calibration_correct: false,
            min_val: 0.000001,
            poisson: 0.3,
            num_e_param: "single".to_string(),
            num_ct_param: None,
            rho_thresh1: None,
            rho_thresh2: None,
            ethresh1: None,
            ethresh2: None,
            rho_asha1: None,
            rho_ashb1: None,
            rho_asha2: None,
            rho_ashb2: None,
            rho_asha3: None,
            rho_ashb3: None,
            ea1: 0.0,
            eb1: 1.0,
            ec1: 1.0,
            ea2: None,
            eb2: None,
            ec2: None,
            ea3: None,
            eb3: None,
            ec3: None,
            integration_scheme: "dense".to_string(),
            back_calculation: true,
            ignore: vec![],
            mesh_translate_x: None,
            mesh_translate_y: None,
            mesh_translate_z: None,
            mesh_rotate_x: None,
            mesh_rotate_y: None,
            mesh_rotate_z: None,
            histogram_export: false,

            histogram_dir: None,
        }
    }

    #[test]
    fn build_histogram_counts() {
        let params = sample_params();
        let raw = vec![1.0, 1.0, 2.0];
        let histogram = build_material_histogram(&raw, &params);
        assert_eq!(histogram.total, 3);
        assert_eq!(histogram.buckets.len(), 2);
        assert_eq!(histogram.buckets[0].count, 2);
        assert_eq!(histogram.buckets[1].count, 1);
    }

    #[test]
    fn histogram_paths_use_base_name() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(Path::new("meshMAT.inp"));
        let (csv, json) = histogram_output_paths(temp_path.to_str().unwrap(), None).expect("paths");
        assert!(csv.ends_with("meshMAT_histogram.csv"));
        assert!(json.ends_with("meshMAT_histogram.json"));
    }
}
