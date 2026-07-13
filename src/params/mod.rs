use std::fs;
use std::path::Path;
use anyhow::{Result, bail, Context};
use serde::{Serialize, Deserialize};
use serde::de::Deserializer;
use log::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
	pub integration: Option<String>, // None means V1 behaviour
	#[serde(alias = "gapValue")]
	pub gap_value: Option<f64>,
	#[serde(alias = "numMaterials")]
	pub num_materials: Option<usize>,
	#[serde(default = "default_grouping_density", alias = "groupingDensity")]
	pub grouping_density: String, // "mean" or "max" or "mid" or "min" or "median"
	#[serde(alias = "intSteps")]
	pub int_steps: usize,
	#[serde(alias = "rhoQCTa")]
	pub rho_qct_a: f64,
	#[serde(alias = "rhoQCTb")]
	pub rho_qct_b: f64,
	#[serde(default, alias = "calibrationCorrect")]
	pub calibration_correct: bool,
	#[serde(alias = "minVal")]
	pub min_val: f64,
	pub poisson: f64,
	#[serde(default = "default_num_e_param", alias = "numEparam")]
	pub num_e_param: String, // single | triple
	#[serde(alias = "numCTparam")]
	pub num_ct_param: Option<String>, // single | triple when calibration_correct
	// thresholds
	#[serde(alias = "rhoThresh1")]
	pub rho_thresh1: Option<f64>,
	#[serde(alias = "rhoThresh2")]
	pub rho_thresh2: Option<f64>,
	#[serde(alias = "Ethresh1")]
	pub ethresh1: Option<f64>,
	#[serde(alias = "Ethresh2")]
	pub ethresh2: Option<f64>,
	// rho ash params (1..3)
	#[serde(alias = "rhoAsha1")]
	pub rho_asha1: Option<f64>,
	#[serde(alias = "rhoAshb1")]
	pub rho_ashb1: Option<f64>,
	#[serde(alias = "rhoAsha2")]
	pub rho_asha2: Option<f64>,
	#[serde(alias = "rhoAshb2")]
	pub rho_ashb2: Option<f64>,
	#[serde(alias = "rhoAsha3")]
	pub rho_asha3: Option<f64>,
	#[serde(alias = "rhoAshb3")]
	pub rho_ashb3: Option<f64>,
	// modulus params (Ea Eb Ec sets 1..3)
	#[serde(alias = "Ea1")]
	pub ea1: f64,
	#[serde(alias = "Eb1")]
	pub eb1: f64,
	#[serde(alias = "Ec1")]
	pub ec1: f64,
	#[serde(alias = "Ea2")]
	pub ea2: Option<f64>,
	#[serde(alias = "Eb2")]
	pub eb2: Option<f64>,
	#[serde(alias = "Ec2")]
	pub ec2: Option<f64>,
	#[serde(alias = "Ea3")]
	pub ea3: Option<f64>,
	#[serde(alias = "Eb3")]
	pub eb3: Option<f64>,
	#[serde(alias = "Ec3")]
	pub ec3: Option<f64>,
	#[serde(default = "default_integration_scheme", alias = "integrationScheme")]
	pub integration_scheme: String, // dense | voxel
	#[serde(default = "default_back_calculation", alias = "enableBackCalculation", alias = "backCalculation")]
	pub back_calculation: bool, // Enable density back calculation feature

	#[serde(default, deserialize_with = "deserialize_ignore")]
	pub ignore: Vec<String>,
	// mesh transformation parameters
	#[serde(alias = "meshTranslateX")]
	pub mesh_translate_x: Option<f64>,
	#[serde(alias = "meshTranslateY")]
	pub mesh_translate_y: Option<f64>,
	#[serde(alias = "meshTranslateZ")]
	pub mesh_translate_z: Option<f64>,
	#[serde(alias = "meshRotateX")]
	pub mesh_rotate_x: Option<f64>, // rotation in degrees
	#[serde(alias = "meshRotateY")]
	pub mesh_rotate_y: Option<f64>, // rotation in degrees
	#[serde(alias = "meshRotateZ")]
	pub mesh_rotate_z: Option<f64>, // rotation in degrees
	// histogram export options
	#[serde(default, alias = "histogramExport", alias = "histogram_export")]
	pub histogram_export: bool,
	#[serde(alias = "histogramDir", alias = "histogram_dir")]
	pub histogram_dir: Option<String>,
}

fn default_grouping_density() -> String { "mean".to_string() }

fn default_integration_scheme() -> String { "dense".to_string() }

fn default_num_e_param() -> String { "single".to_string() }

fn default_back_calculation() -> bool { true }

fn deserialize_ignore<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
	D: Deserializer<'de>
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum IgnoreField {
		String(String),
		List(Vec<String>),
	}

	let value = Option::<IgnoreField>::deserialize(deserializer)?;
	let items = match value {
		None => Vec::new(),
		Some(IgnoreField::String(raw)) => raw
			.split(';')
			.map(|s| s.trim().to_string())
			.filter(|s| !s.is_empty())
			.collect(),
		Some(IgnoreField::List(list)) => list,
	};
	Ok(items)
}

fn flatten_toml_tables(root: &toml::value::Table) -> Result<toml::value::Table> {
	let mut flat = toml::value::Table::new();
	for (key, value) in root {
		if let toml::Value::Table(section) = value {
			for (sub_key, sub_value) in section {
				if matches!(sub_value, toml::Value::Table(_)) {
					bail!("Nested TOML tables are not supported (found {key}.{sub_key})");
				}
				if flat.contains_key(sub_key) {
					bail!("Duplicate parameter key '{sub_key}' across TOML sections");
				}
				flat.insert(sub_key.clone(), sub_value.clone());
			}
		} else {
			if flat.contains_key(key) {
				bail!("Duplicate parameter key '{key}' in TOML root");
			}
			flat.insert(key.clone(), value.clone());
		}
	}
	Ok(flat)
}

impl Params {
	pub fn parse(path: &str) -> Result<Self> {
		let ext = Path::new(path)
			.extension()
			.and_then(|s| s.to_str())
			.unwrap_or("")
			.to_ascii_lowercase();
		if ext != "toml" {
			warn!("Parameter file is expected to be TOML (.toml). Legacy .txt is no longer supported.");
			bail!("Invalid parameter file extension: .{ext}");
		}
		Self::parse_toml(path)
	}

	pub fn parse_toml(path: &str) -> Result<Self> {
		let raw = fs::read_to_string(path)
			.with_context(|| format!("Failed to read TOML params file: {path}"))?;
		let value: toml::Value = toml::from_str(&raw)
			.with_context(|| format!("Failed to parse TOML params file: {path}"))?;
		let table = value.as_table().ok_or_else(|| anyhow::anyhow!("TOML root must be a table"))?;
		let flat = flatten_toml_tables(table)?;
		let params: Params = toml::Value::Table(flat)
			.try_into()
			.with_context(|| format!("Failed to deserialize TOML params file: {path}"))?;
		params.validate()?;
		Ok(params)
	}

	fn validate(&self) -> Result<()> {
		match (self.gap_value, self.num_materials) {
			(Some(_), Some(_)) => bail!("gap_value and num_materials are mutually exclusive"),
			(None, None) => bail!("Either gap_value or num_materials must be set"),
			_ => {}
		}
		if let Some(gap) = self.gap_value && gap < 0.0 { bail!("gapValue must be >= 0"); }	
		if let Some(num) = self.num_materials && num == 0 { bail!("num_materials must be > 0"); }
		if self.int_steps == 0 { bail!("intSteps must be > 0"); }
		if self.num_e_param == "triple" {
			for f in [self.ea2, self.eb2, self.ec2, self.ea3, self.eb3, self.ec3] { if f.is_none() { bail!("Triple modulus parameters incomplete"); } }
			if self.ethresh1.is_none() || self.ethresh2.is_none() { bail!("Ethresh1 and Ethresh2 required for triple modulus"); }
		}
		if self.calibration_correct {
			if self.num_ct_param.is_none() { bail!("numCTparam required when calibrationCorrect = true"); }
			if self.num_ct_param.as_deref() == Some("single") {
				if self.rho_asha1.is_none() || self.rho_ashb1.is_none() { bail!("rhoAsha1/rhoAshb1 required"); }
			} else {
				for f in [self.rho_thresh1, self.rho_thresh2, self.rho_asha1, self.rho_ashb1, self.rho_asha2, self.rho_ashb2, self.rho_asha3, self.rho_ashb3] { if f.is_none() { bail!("Triple calibration parameters incomplete"); } }
			}
		}
		Ok(())
	}

	pub fn rho_qct(&self, hu: f64) -> f64 { (self.rho_qct_a + self.rho_qct_b * hu).max(1e-6) }

	pub fn rho_ash(&self, q: f64) -> f64 {
		if !self.calibration_correct { return q.max(1e-6); }
		match self.num_ct_param.as_deref() {
			Some("single") => {
				let a = self.rho_asha1.unwrap();
				let b = self.rho_ashb1.unwrap();
				(a + b * q).max(1e-6)
			},
			Some("triple") => {
				let q1 = self.rho_thresh1.unwrap();
				let q2 = self.rho_thresh2.unwrap();
				let (a,b) = if q < q1 { (self.rho_asha1.unwrap(), self.rho_ashb1.unwrap()) }
							 else if q <= q2 { (self.rho_asha2.unwrap(), self.rho_ashb2.unwrap()) }
							 else { (self.rho_asha3.unwrap(), self.rho_ashb3.unwrap()) };
				(a + b * q).max(1e-6)
			},
			_ => q.max(1e-6)
		}
	}

	pub fn modulus(&self, ash: f64) -> f64 {
		if self.num_e_param == "single" {
			let res = self.ea1 + self.eb1 * ash.powf(self.ec1);
			return res.max(1e-6);
		}
		let e1 = self.ea1 + self.eb1 * ash.powf(self.ec1);
		let eth1 = self.ethresh1.unwrap();
		let eth2 = self.ethresh2.unwrap();
		if ash < eth1 {
			e1.max(1e-6)
		} else if ash <= eth2 {
			let res = self.ea2.unwrap() + self.eb2.unwrap() * ash.powf(self.ec2.unwrap());
			res.max(1e-6)
		} else {
			let res = self.ea3.unwrap() + self.eb3.unwrap() * ash.powf(self.ec3.unwrap());
			res.max(1e-6)
		}
	}

	/// Density back calculation from elasticity using the density-elasticity relationship E = a + b * Rho^c
	/// Uses the existing modulus parameters (ea, eb, ec) and thresholds to calculate density from elasticity
	/// This improves the predicted elastic modulus values for bone material assignment
	pub fn density_back_calculation(&self, elasticity: f64) -> f64 {
		if self.num_e_param == "single" {
			// Single interval: density = ((E - a) / b)^(1/c)
			let a = self.ea1;
			let b = self.eb1;
			let c = self.ec1;
			
			// Ensure we don't get negative values under the root
			let numerator = (elasticity - a) / b;
			if numerator <= 0.0 {
				return self.min_val.max(1e-6); // Return minimum density if calculation would be invalid
			}
			
			numerator.powf(1.0 / c).max(self.min_val).max(1e-6)
		} else {
			// Three intervals: determine which interval based on elasticity thresholds
			// First calculate the elasticity thresholds at the density boundaries
			let rho1 = self.ethresh1.unwrap(); // Using ethresh1 as rho1 threshold
			let rho2 = self.ethresh2.unwrap(); // Using ethresh2 as rho2 threshold
			
			// Calculate elasticity values at the density thresholds using each interval's parameters
			let e1 = self.ea1 + self.eb1 * rho1.powf(self.ec1);
			let e2 = if let (Some(ea2), Some(eb2), Some(ec2)) = (self.ea2, self.eb2, self.ec2) {
				ea2 + eb2 * rho2.powf(ec2)
			} else {
				// Fallback to first interval if second interval parameters not available
				self.ea1 + self.eb1 * rho2.powf(self.ec1)
			};
			
			// Determine which interval to use based on input elasticity
			if elasticity < e1 {
				// Use first interval parameters (Rho < Rho1)
				let a = self.ea1;
				let b = self.eb1;
				let c = self.ec1;
				
				let numerator = (elasticity - a) / b;
				if numerator <= 0.0 {
					return self.min_val.max(1e-6);
				}
				numerator.powf(1.0 / c).max(self.min_val).max(1e-6)
			} else if elasticity < e2 {
				// Use second interval parameters (Rho1 <= Rho <= Rho2)
				if let (Some(ea2), Some(eb2), Some(ec2)) = (self.ea2, self.eb2, self.ec2) {
					let numerator = (elasticity - ea2) / eb2;
					if numerator <= 0.0 {
						return self.min_val.max(1e-6);
					}
					numerator.powf(1.0 / ec2).max(self.min_val).max(1e-6)
				} else {
					// Fallback to first interval if parameters not available
					let numerator = (elasticity - self.ea1) / self.eb1;
					if numerator <= 0.0 {
						return self.min_val.max(1e-6);
					}
					numerator.powf(1.0 / self.ec1).max(self.min_val).max(1e-6)
				}
			} else {
				// Use third interval parameters (Rho > Rho2)
				if let (Some(ea3), Some(eb3), Some(ec3)) = (self.ea3, self.eb3, self.ec3) {
					let numerator = (elasticity - ea3) / eb3;
					if numerator <= 0.0 {
						return self.min_val.max(1e-6);
					}
					numerator.powf(1.0 / ec3).max(self.min_val).max(1e-6)
				} else {
					// Fallback to second interval or first interval if parameters not available
					if let (Some(ea2), Some(eb2), Some(ec2)) = (self.ea2, self.eb2, self.ec2) {
						let numerator = (elasticity - ea2) / eb2;
						if numerator <= 0.0 {
							return self.min_val.max(1e-6);
						}
						numerator.powf(1.0 / ec2).max(self.min_val).max(1e-6)
					} else {
						let numerator = (elasticity - self.ea1) / self.eb1;
						if numerator <= 0.0 {
							return self.min_val.max(1e-6);
						}
						numerator.powf(1.0 / self.ec1).max(self.min_val).max(1e-6)
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Params;
	use std::io::Write;

	#[test]
	fn parse_toml_smoke() {
		let mut file = tempfile::NamedTempFile::new().expect("temp file");
		writeln!(
			file,
			"[luma_options]\n\
			gap_value = 50\n\
			int_steps = 8\n\
			min_val = 0.000001\n\
			poisson = 0.3\n\
			ea1 = 0\n\
			eb1 = 14664\n\
			ec1 = 1.49\n\
			ignore = [\"ACL\", \"pin\"]\n\
			\n\
			[ct_calibration_coefficients]\n\
			rho_qct_a = -0.01222\n\
			rho_qct_b = 0.0007079\n"
		).expect("write toml");

		let params = Params::parse_toml(file.path().to_str().unwrap()).expect("parse toml");
		assert_eq!(params.grouping_density, "mean");
		assert_eq!(params.integration_scheme, "dense");
		assert_eq!(params.num_e_param, "single");
		assert!(params.back_calculation);
		assert_eq!(params.ignore, vec!["ACL".to_string(), "pin".to_string()]);
		assert_eq!(params.gap_value, Some(50.0));
	}

	#[test]
	fn parse_toml_num_materials_only() {
		let mut file = tempfile::NamedTempFile::new().expect("temp file");
		writeln!(
			file,
			"[luma_options]\n\
			num_materials = 8\n\
			int_steps = 8\n\
			min_val = 0.000001\n\
			poisson = 0.3\n\
			ea1 = 0\n\
			eb1 = 14664\n\
			ec1 = 1.49\n\
			\n\
			[ct_calibration_coefficients]\n\
			rho_qct_a = -0.01222\n\
			rho_qct_b = 0.0007079\n"
		).expect("write toml");

		let params = Params::parse_toml(file.path().to_str().unwrap()).expect("parse toml");
		assert_eq!(params.num_materials, Some(8));
		assert_eq!(params.gap_value, None);
	}

	#[test]
	fn parse_toml_mutually_exclusive_grouping() {
		let mut file = tempfile::NamedTempFile::new().expect("temp file");
		writeln!(
			file,
			"[luma_options]\n\
			gap_value = 10\n\
			num_materials = 8\n\
			int_steps = 8\n\
			min_val = 0.000001\n\
			poisson = 0.3\n\
			ea1 = 0\n\
			eb1 = 14664\n\
			ec1 = 1.49\n\
			\n\
			[ct_calibration_coefficients]\n\
			rho_qct_a = -0.01222\n\
			rho_qct_b = 0.0007079\n"
		).expect("write toml");

		let err = Params::parse_toml(file.path().to_str().unwrap()).unwrap_err();
		assert!(err.to_string().contains("mutually exclusive"));
	}
}