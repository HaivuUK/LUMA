use anyhow::{Result, anyhow};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use dicom_object::{open_file, DicomAttribute, DicomObject};
use dicom_pixeldata::PixelDecoder;

use super::{Volume};

#[derive(Debug, Clone)]
pub struct DicomSeries {
    series_instance_uid: String,
    series_description: Option<String>,
    series_number: Option<i32>,
    files: Vec<PathBuf>,
}

impl DicomSeries {
    fn label(&self) -> String {
        let mut parts = Vec::new();
        if let Some(number) = self.series_number {
            parts.push(format!("Series {number}"));
        }
        if let Some(description) = &self.series_description {
            parts.push(description.clone());
        }
        if parts.is_empty() {
            parts.push(self.series_instance_uid.clone());
        }
        format!("{} [{} files]", parts.join(" - "), self.files.len())
    }
}

#[derive(Debug)]
struct DicomSlice {
    z_position: f64,
    instance_number: i32,
    rows: usize,
    cols: usize,
    pixel_spacing: [f64; 2],
    image_position: [f64; 3],
    pixels: Vec<f32>,
}

fn collect_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        files.push(path.to_path_buf());
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            collect_files_recursive(&child, files)?;
        } else if child.is_file() {
            files.push(child);
        }
    }

    Ok(())
}

fn read_string_tag<T: dicom_object::DicomObject>(obj: &T, name: &str) -> Option<String> {
    Some(obj.attr_by_name(name)
        .ok()?
        .to_primitive_value()
        .ok()?
        .to_str()
        .trim()
        .to_string())
}

fn read_i32_tag<T: dicom_object::DicomObject>(obj: &T, name: &str) -> Option<i32> {
    obj.attr_by_name(name).ok()?.to_i32().ok()
}

fn read_f64_tag<T: dicom_object::DicomObject>(obj: &T, name: &str) -> Option<f64> {
    let vals = obj.attr_by_name(name).ok()?.to_primitive_value().ok()?.to_multi_float64().ok()?;
    vals.first().copied()
}

pub fn discover_dicom_series(root: &Path) -> Result<Vec<DicomSeries>> {
    let mut paths = Vec::new();
    collect_files_recursive(root, &mut paths)?;

    let mut series_map = std::collections::BTreeMap::<String, DicomSeries>::new();

    for path in paths {
        let Ok(obj) = open_file(&path) else { continue; };
        let Some(series_uid) = read_string_tag(&obj, "SeriesInstanceUID") else { continue; };
        let series_description = read_string_tag(&obj, "SeriesDescription");
        let series_number = read_i32_tag(&obj, "SeriesNumber");

        series_map.entry(series_uid.clone()).or_insert_with(|| DicomSeries {
            series_instance_uid: series_uid,
            series_description,
            series_number,
            files: Vec::new(),
        }).files.push(path);
    }

    let mut series: Vec<DicomSeries> = series_map.into_values().collect();
    series.sort_by(|a, b| {
        a.series_number
            .cmp(&b.series_number)
            .then_with(|| a.series_description.cmp(&b.series_description))
            .then_with(|| a.series_instance_uid.cmp(&b.series_instance_uid))
    });

    Ok(series)
}

fn choose_dicom_series(series: &[DicomSeries]) -> Result<usize> {
    println!("Multiple DICOM series were found:");
    for (index, item) in series.iter().enumerate() {
        println!("  {}. {}", index + 1, item.label());
    }

    loop {
        print!("Select a series to load [1-{}]: ", series.len());
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        if bytes_read == 0 {
            return Err(anyhow!("No series selected from terminal"));
        }

        if let Ok(choice) = input.trim().parse::<usize>()
            && (1..=series.len()).contains(&choice) {
                return Ok(choice - 1);
            }

        println!("Invalid selection, please enter a number between 1 and {}.", series.len());
    }
}

fn load_dicom_series(files: &[PathBuf]) -> Result<Volume> {
    let mut slices = Vec::new();

    for path in files {
        let obj = open_file(path)?;
        let rows: usize = obj.attr_by_name("Rows")?.to_u16()? as usize;
        let cols: usize = obj.attr_by_name("Columns")?.to_u16()? as usize;
        let pix_spacing = obj.attr_by_name("PixelSpacing")?.to_primitive_value()?.to_multi_float64()?;
        let img_pos = obj.attr_by_name("ImagePositionPatient")?.to_primitive_value()?.to_multi_float64()?;

        if pix_spacing.len() < 2 {
            return Err(anyhow!("PixelSpacing does not contain two values in {:?}", path));
        }
        if img_pos.len() < 3 {
            return Err(anyhow!("ImagePositionPatient does not contain three values in {:?}", path));
        }

        let decoded = obj.decode_pixel_data()?;
        let pixels: Vec<i32> = decoded.to_vec_frame::<i32>(0)?;
        if pixels.len() != rows * cols {
            return Err(anyhow!(
				"Unexpected pixel count in {:?}: got {}, expected {}",
				path,
				pixels.len(),
				rows * cols,
			));
        }

        // Read DICOM tags affecting pixel interpretation
        let bits_stored = read_i32_tag(&obj, "BitsStored");
        let photometric = read_string_tag(&obj, "PhotometricInterpretation");

        // Convert raw pixels to f32 and apply photometric inversion (MONOCHROME1)
        let mut converted: Vec<f32> = pixels.into_iter().map(|v| v as f32).collect();
        if photometric.as_deref() == Some("MONOCHROME1") {
            let max_val = bits_stored.map(|b| ((1u64 << (b as u64)) - 1) as f32).unwrap_or_else(|| {
                // fallback: use max observed value
                converted.iter().copied().fold(f32::NEG_INFINITY, f32::max)
            });
            for v in &mut converted { *v = max_val - *v; }
        }

        let instance_number = read_i32_tag(&obj, "InstanceNumber").unwrap_or(0);
        slices.push(DicomSlice {
            z_position: img_pos[2],
            instance_number,
            rows,
            cols,
            pixel_spacing: [pix_spacing[0], pix_spacing[1]],
            image_position: [img_pos[0], img_pos[1], img_pos[2]],
            pixels: converted,
        });
    }

    if slices.is_empty() {
        return Err(anyhow!("No readable DICOM slices found in selected series"));
    }

    slices.sort_by(|a, b| {
        a.z_position
            .total_cmp(&b.z_position)
            .then_with(|| a.instance_number.cmp(&b.instance_number))
    });

    let rows = slices[0].rows;
    let cols = slices[0].cols;
    let pixel_spacing = slices[0].pixel_spacing;
    let first_position = slices[0].image_position;

    for slice in &slices {
        if slice.rows != rows || slice.cols != cols {
            return Err(anyhow!("Inconsistent DICOM slice dimensions within the selected series"));
        }
        if (slice.pixel_spacing[0] - pixel_spacing[0]).abs() > f64::EPSILON
            || (slice.pixel_spacing[1] - pixel_spacing[1]).abs() > f64::EPSILON
        {
            return Err(anyhow!("Inconsistent PixelSpacing within the selected series"));
        }
    }

    let x0 = first_position[0];
    let y0 = first_position[1];
    let dx = pixel_spacing[1];
    let dy = pixel_spacing[0];
    let x: Vec<f64> = (0..cols).map(|i| x0 + (i as f64) * dx).collect();
    let y: Vec<f64> = (0..rows).map(|i| y0 + (i as f64) * dy).collect();
    let z: Vec<f64> = slices.iter().map(|slice| slice.z_position).collect();

    let mut scalars = Vec::with_capacity(rows * cols * slices.len());
    for slice in &slices {
        scalars.extend(slice.pixels.iter().copied());
    }

    Volume::new(x, y, z, scalars)
}

pub fn load_dicom_dir(path: &str) -> Result<Volume> {
    let root = Path::new(path);
    let series = discover_dicom_series(root)?;
    if series.is_empty() {
        return Err(anyhow!("No DICOM files found"));
    }

    let selected_index = if series.len() == 1 {
        0
    } else {
        choose_dicom_series(&series)?
    };

    load_dicom_series(&series[selected_index].files)
}