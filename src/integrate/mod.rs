use crate::mesh::{ElementKind};
use crate::params::Params;
use crate::volume::Volume;
use anyhow::Result;
use rayon::prelude::*;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use log::{debug, warn, error};

static PROGRESS_COUNTER: once_cell::sync::Lazy<std::sync::Mutex<Option<Arc<AtomicUsize>>>> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

pub fn set_progress_counter(c: Option<Arc<AtomicUsize>>) { *PROGRESS_COUNTER.lock().unwrap() = c; }

#[derive(Debug, Clone)]
pub struct ElementResult { pub part_index: usize, pub element_index: usize, pub modulus: f64 }

pub fn assign_materials(mesh: &crate::mesh::Mesh, vol: &Volume, p: &Params) -> Result<Vec<Vec<f64>>> {
	debug!("Start integration: parts={} mode={:?} scheme={} steps={}", mesh.parts.len(), p.integration, p.integration_scheme, p.int_steps);
	debug!("Volume: nx={} ny={} nz={} scalars={} range=[{:.3},{:.3}]", vol.x.len(), vol.y.len(), vol.z.len(), vol.scalars.len(), vol.scalars.iter().cloned().fold(f32::INFINITY,f32::min), vol.scalars.iter().cloned().fold(f32::NEG_INFINITY,f32::max));
	let mut per_part: Vec<Vec<f64>> = Vec::new();
	for (pi, part) in mesh.parts.iter().enumerate() {
		debug!("Part {} '{}' elements={} nodes={} ignore={} kind_sample={:?}", pi, part.name.as_deref().unwrap_or(&"Unnamed"), part.elements.len(), part.nodes.len(), part.ignore, part.elements.first().map(|e| &e.kind));
		if part.ignore { per_part.push(vec![]); continue; }
		let nodes_clone = &part.nodes;
		let idx = &part.node_index;
		let counter_arc_opt = PROGRESS_COUNTER.lock().unwrap().clone();
		let res: Vec<f64> = part.elements.par_iter().map(|e| {
			if e.id % 1000 == 0 { debug!("Part {} element progress id={}", pi, e.id); }
			// Pre-allocate with exact capacity to avoid reallocations
			let mut pts: Vec<[f64;3]> = Vec::with_capacity(e.nodes.len());
			// Use the node_index HashMap for O(1) lookups instead of linear search
			for nid in &e.nodes { 
				match idx.get(nid).and_then(|i| nodes_clone.get(*i)) { 
					Some(n)=> pts.push([n.x,n.y,n.z]), 
					None => { 
						warn!("Missing node {} in element {} part {}", nid, e.id, pi); 
						return f64::NAN; 
					} 
				} 
			}
			// Pass pre-gathered points directly to avoid duplicate node lookups
			let val = integrate_element_with_points(e.kind.clone(), &pts, vol, p);
			// Fast path progress counter pointer retrieval (avoid lock every element): copy Arc once outside map? kept simple first.
			if let Some(c) = &counter_arc_opt { c.fetch_add(1, Ordering::Relaxed); }
			if !val.is_finite() { warn!("Non-finite result element {} part {} => {}", e.id, pi, val); }
			val
		}).collect();
		per_part.push(res);
	}
	debug!("Integration complete");
	Ok(per_part)
}

fn integrate_element_with_points(kind: ElementKind, pts: &[[f64;3]], vol: &Volume, p: &Params) -> f64 {
	// Points are already gathered, no need for node lookups
	// catch potential panics inside integration (e.g., index issues)
	let res = std::panic::catch_unwind(|| {
		match kind {
			ElementKind::Tet4 => integrate_tet4(&pts, p, vol),
			ElementKind::Tet10 => integrate_tet10(&pts, p, vol),
			ElementKind::Hex8 => integrate_hex8(&pts, p, vol),
			ElementKind::Wedge6 => integrate_wedge6(&pts, p, vol),
		}
	});
	match res { Ok(v) => v, Err(_) => { error!("Panic integrating element -> NaN"); f64::NAN } }
}

fn integrate_element(kind: ElementKind, node_ids: &Vec<u32>, nodes: &Vec<crate::mesh::Node>, vol: &Volume, p: &Params) -> f64 {
	// gather coordinates safely
	let mut pts: Vec<[f64;3]> = Vec::with_capacity(node_ids.len());
	for nid in node_ids { match nodes.iter().find(|n| n.id == *nid) { Some(n)=> pts.push([n.x,n.y,n.z]), None => { error!("Missing node id {} -> marking element invalid", nid); return f64::NAN; } } }
	integrate_element_with_points(kind, &pts, vol, p)
}

pub fn process(mesh: &crate::mesh::Mesh, vol: &Volume, params: &Params) -> Result<Vec<Vec<f64>>> {
	assign_materials(mesh, vol, params)
}

pub fn process_with_progress<F>(mesh: &crate::mesh::Mesh, vol: &Volume, params: &Params, tick: F) -> Result<Vec<Vec<f64>>>
where F: Fn(usize) + Send + Sync + 'static + Clone {
	let total: usize = mesh.parts.iter().filter(|p| !p.ignore).map(|p| p.elements.len()).sum();
	let counter_arc_master = PROGRESS_COUNTER.lock().unwrap().clone();
	let counter_arc = counter_arc_master.clone();
	// spawn a lightweight thread to poll counter and invoke callback
	let done_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
	let done_clone = done_flag.clone();
	let tick_bg = tick.clone();
	std::thread::spawn(move || {
		use std::time::Duration; let mut last = 0usize;
		loop {
			if done_clone.load(std::sync::atomic::Ordering::Relaxed) { break; }
			if let Some(c) = &counter_arc { 
				let v = c.load(std::sync::atomic::Ordering::Relaxed); 
				if v != last { 
					tick_bg(v); 
					last = v; 
					if v >= total { break; } 
				} 
			}
			std::thread::sleep(Duration::from_millis(10)); // More frequent polling for real-time updates
		}
	});
	let res = assign_materials(mesh, vol, params);
	done_flag.store(true, std::sync::atomic::Ordering::Relaxed);
	if let Some(c) = &counter_arc_master { let v = c.load(std::sync::atomic::Ordering::Relaxed); tick(v); }
	res
}

fn integrate_tet4(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// E mode applies equations at each point, HU mode averages HU first
	match p.integration.as_deref() {
		Some("E") => integrate_tet4_e_mode(pts, p, vol),
		_ => integrate_tet4_hu_mode(pts, p, vol), // HU mode and None/legacy
	}
}

fn integrate_tet4_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// E mode: Apply full equation chain at each integration point, then average
	
	// Select integration method based on scheme
	match p.integration_scheme.as_str() {
		"voxel" => voxel_aware_integration_e_mode(pts, p, vol),
		"dense" => dense_tet_sampling_e_mode(pts, p, vol),
		_ => dense_tet_sampling_e_mode(pts, p, vol), // Default to dense
	}
}

fn integrate_tet4_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// HU mode: Average HU values first, then apply equation chain once
	
	// Select integration method based on scheme
	match p.integration_scheme.as_str() {
		"voxel" => voxel_aware_integration_hu_mode(pts, p, vol),
		"dense" => dense_tet_sampling_hu_mode(pts, p, vol),
		_ => dense_tet_sampling_hu_mode(pts, p, vol), // Default to dense
	}
}

/// Dense tetrahedral sampling for E-mode (explicit user control)
fn dense_tet_sampling_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use proper tetrahedral sampling with barycentric coordinates
	// Simple rejection sampling - mathematically correct and unbiased
	let mut sum = 0.0;
	let mut count = 0;
	
	// Generate uniformly distributed points in tetrahedral barycentric space
	// Increase sampling density to get more points despite rejection
	let sampling_factor = ((p.int_steps as f64 * 1.5) as usize).max(p.int_steps);
	
	for i in 0..sampling_factor {
		for j in 0..sampling_factor {
			for k in 0..sampling_factor {
				// Convert to barycentric coordinates (u, v, w) where u+v+w <= 1
				let u = (i as f64 + 0.5) / (sampling_factor as f64);
				let v = (j as f64 + 0.5) / (sampling_factor as f64);
				let w = (k as f64 + 0.5) / (sampling_factor as f64);
				
				// Check if point is inside tetrahedral simplex
				if u + v + w <= 1.0 {
					let t = 1.0 - u - v - w; // Fourth barycentric coordinate
					
					// Convert barycentric to Cartesian coordinates
					let x = u * pts[0][0] + v * pts[1][0] + w * pts[2][0] + t * pts[3][0];
					let y = u * pts[0][1] + v * pts[1][1] + w * pts[2][1] + t * pts[3][1];
					let z = u * pts[0][2] + v * pts[1][2] + w * pts[2][2] + t * pts[3][2];
					
					let hu = vol.trilinear(x, y, z);
					sum += pipeline_e_mode(hu, p);
					count += 1;
				}
			}
		}
	}
	
	if count > 0 { sum / count as f64 } else { 0.0 }
}

/// Dense tetrahedral sampling for HU-mode (explicit user control)
fn dense_tet_sampling_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use proper tetrahedral sampling with barycentric coordinates
	let mut sum_hu = 0.0;
	let mut count = 0;
	
	// Generate uniformly distributed points in tetrahedral barycentric space
	let sampling_factor = ((p.int_steps as f64 * 1.5) as usize).max(p.int_steps);
	
	for i in 0..sampling_factor {
		for j in 0..sampling_factor {
			for k in 0..sampling_factor {
				// Convert to barycentric coordinates (u, v, w) where u+v+w <= 1
				let u = (i as f64 + 0.5) / (sampling_factor as f64);
				let v = (j as f64 + 0.5) / (sampling_factor as f64);
				let w = (k as f64 + 0.5) / (sampling_factor as f64);
				
				// Check if point is inside tetrahedral simplex
				if u + v + w <= 1.0 {
					let t = 1.0 - u - v - w; // Fourth barycentric coordinate
					
					// Convert barycentric to Cartesian coordinates
					let x = u * pts[0][0] + v * pts[1][0] + w * pts[2][0] + t * pts[3][0];
					let y = u * pts[0][1] + v * pts[1][1] + w * pts[2][1] + t * pts[3][1];
					let z = u * pts[0][2] + v * pts[1][2] + w * pts[2][2] + t * pts[3][2];
					
					sum_hu += vol.trilinear(x, y, z) as f64;
					count += 1;
				}
			}
		}
	}
	
	if count > 0 {
		let avg_hu = sum_hu / count as f64;
		pipeline_hu_mode(avg_hu, p)
	} else { 
		0.0 
	}
}

/// Voxel-aware integration method that reduces partial volume errors
/// Uses voxel spacing information to adaptively sample based on element size relative to voxel size
fn voxel_aware_integration_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	let element_size = estimate_element_size(pts);
	let voxel_size = (vol.voxel_spacing[0] + vol.voxel_spacing[1] + vol.voxel_spacing[2]) / 3.0;
	
	// Determine sampling strategy based on element size relative to voxel size
	if element_size < voxel_size * 0.5 {
		// Element much smaller than voxel: use centroid sampling to avoid noise
		let c = [
			(pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]) / 4.0,
			(pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]) / 4.0,
			(pts[0][2] + pts[1][2] + pts[2][2] + pts[3][2]) / 4.0,
		];
		let hu = vol.trilinear(c[0], c[1], c[2]);
		pipeline_e_mode(hu, p)
	} else if element_size > voxel_size * 3.0 {
		// Element much larger than voxel: use dense sampling to capture variation
		voxel_aligned_dense_sampling_e_mode(pts, p, vol)
	} else {
		// Element size comparable to voxel: use structured sampling
		voxel_structured_sampling_e_mode(pts, p, vol)
	}
}

fn voxel_aware_integration_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	let element_size = estimate_element_size(pts);
	let voxel_size = (vol.voxel_spacing[0] + vol.voxel_spacing[1] + vol.voxel_spacing[2]) / 3.0;
	
	if element_size < voxel_size * 0.5 {
		// Small element: centroid sampling
		let c = [
			(pts[0][0] + pts[1][0] + pts[2][0] + pts[3][0]) / 4.0,
			(pts[0][1] + pts[1][1] + pts[2][1] + pts[3][1]) / 4.0,
			(pts[0][2] + pts[1][2] + pts[2][2] + pts[3][2]) / 4.0,
		];
		let hu = vol.trilinear(c[0], c[1], c[2]);
		pipeline_hu_mode(hu, p)
	} else if element_size > voxel_size * 3.0 {
		// Large element: dense sampling then average HU
		voxel_aligned_dense_sampling_hu_mode(pts, p, vol)
	} else {
		// Medium element: structured sampling
		voxel_structured_sampling_hu_mode(pts, p, vol)
	}
}

/// Dense sampling aligned with voxel boundaries to reduce partial volume effects
fn voxel_aligned_dense_sampling_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	let [min_x, max_x] = [pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max)];
	let [min_y, max_y] = [pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max)];
	let [min_z, max_z] = [pts.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[2]).fold(f64::NEG_INFINITY, f64::max)];
	
	// Calculate sampling steps based on voxel spacing
	let steps_x = ((max_x - min_x) / vol.voxel_spacing[0]).ceil().max(2.0) as usize;
	let steps_y = ((max_y - min_y) / vol.voxel_spacing[1]).ceil().max(2.0) as usize;
	let steps_z = ((max_z - min_z) / vol.voxel_spacing[2]).ceil().max(2.0) as usize;
	
	let mut sum_modulus = 0.0;
	let mut count = 0;
	
	for i in 0..steps_x {
		for j in 0..steps_y {
			for k in 0..steps_z {
				let x = min_x + (i as f64 + 0.5) * (max_x - min_x) / steps_x as f64;
				let y = min_y + (j as f64 + 0.5) * (max_y - min_y) / steps_y as f64;
				let z = min_z + (k as f64 + 0.5) * (max_z - min_z) / steps_z as f64;
				
				if point_in_tetrahedron([x, y, z], pts) {
					let hu = vol.trilinear(x, y, z);
					sum_modulus += pipeline_e_mode(hu, p);
					count += 1;
				}
			}
		}
	}
	
	if count > 0 { sum_modulus / count as f64 } else { 0.0 }
}

fn voxel_aligned_dense_sampling_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	let [min_x, max_x] = [pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max)];
	let [min_y, max_y] = [pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max)];
	let [min_z, max_z] = [pts.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p[2]).fold(f64::NEG_INFINITY, f64::max)];
	
	let steps_x = ((max_x - min_x) / vol.voxel_spacing[0]).ceil().max(2.0) as usize;
	let steps_y = ((max_y - min_y) / vol.voxel_spacing[1]).ceil().max(2.0) as usize;
	let steps_z = ((max_z - min_z) / vol.voxel_spacing[2]).ceil().max(2.0) as usize;
	
	let mut sum_hu = 0.0;
	let mut count = 0;
	
	for i in 0..steps_x {
		for j in 0..steps_y {
			for k in 0..steps_z {
				let x = min_x + (i as f64 + 0.5) * (max_x - min_x) / steps_x as f64;
				let y = min_y + (j as f64 + 0.5) * (max_y - min_y) / steps_y as f64;
				let z = min_z + (k as f64 + 0.5) * (max_z - min_z) / steps_z as f64;
				
				if point_in_tetrahedron([x, y, z], pts) {
					sum_hu += vol.trilinear(x, y, z) as f64;
					count += 1;
				}
			}
		}
	}
	
	if count > 0 { 
		let avg_hu = sum_hu / count as f64;
		pipeline_hu_mode(avg_hu, p)
	} else { 0.0 }
}

/// Structured sampling that considers voxel alignment
fn voxel_structured_sampling_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use 8-point sampling aligned with element boundaries
	let sampling_points = [
		// Corner-like points
		[0.138196601, 0.138196601, 0.138196601, 0.585410197], // Near vertex 0
		[0.585410197, 0.138196601, 0.138196601, 0.138196601], // Near vertex 1
		[0.138196601, 0.585410197, 0.138196601, 0.138196601], // Near vertex 2
		[0.138196601, 0.138196601, 0.585410197, 0.138196601], // Near vertex 3
		// Face-centered points
		[0.25, 0.25, 0.25, 0.25], // Centroid
		[0.4, 0.2, 0.2, 0.2],     // Offset sampling
		[0.2, 0.4, 0.2, 0.2],
		[0.2, 0.2, 0.4, 0.2],
	];
	
	let mut sum_modulus = 0.0;
	for weights in &sampling_points {
		let x = weights[0] * pts[0][0] + weights[1] * pts[1][0] + weights[2] * pts[2][0] + weights[3] * pts[3][0];
		let y = weights[0] * pts[0][1] + weights[1] * pts[1][1] + weights[2] * pts[2][1] + weights[3] * pts[3][1];
		let z = weights[0] * pts[0][2] + weights[1] * pts[1][2] + weights[2] * pts[2][2] + weights[3] * pts[3][2];
		
		let hu = vol.trilinear(x, y, z);
		sum_modulus += pipeline_e_mode(hu, p);
	}
	
	sum_modulus / sampling_points.len() as f64
}

fn voxel_structured_sampling_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	let sampling_points = [
		[0.138196601, 0.138196601, 0.138196601, 0.585410197],
		[0.585410197, 0.138196601, 0.138196601, 0.138196601],
		[0.138196601, 0.585410197, 0.138196601, 0.138196601],
		[0.138196601, 0.138196601, 0.585410197, 0.138196601],
		[0.25, 0.25, 0.25, 0.25],
		[0.4, 0.2, 0.2, 0.2],
		[0.2, 0.4, 0.2, 0.2],
		[0.2, 0.2, 0.4, 0.2],
	];
	
	let mut sum_hu = 0.0;
	for weights in &sampling_points {
		let x = weights[0] * pts[0][0] + weights[1] * pts[1][0] + weights[2] * pts[2][0] + weights[3] * pts[3][0];
		let y = weights[0] * pts[0][1] + weights[1] * pts[1][1] + weights[2] * pts[2][1] + weights[3] * pts[3][1];
		let z = weights[0] * pts[0][2] + weights[1] * pts[1][2] + weights[2] * pts[2][2] + weights[3] * pts[3][2];
		
		sum_hu += vol.trilinear(x, y, z) as f64;
	}
	
	let avg_hu = sum_hu / sampling_points.len() as f64;
	pipeline_hu_mode(avg_hu, p)
}

/// Check if a point is inside a tetrahedron using barycentric coordinates
fn point_in_tetrahedron(pt: [f64; 3], tet: &[[f64; 3]]) -> bool {
	// Convert to barycentric coordinates
	let v0 = [tet[1][0] - tet[0][0], tet[1][1] - tet[0][1], tet[1][2] - tet[0][2]];
	let v1 = [tet[2][0] - tet[0][0], tet[2][1] - tet[0][1], tet[2][2] - tet[0][2]];
	let v2 = [tet[3][0] - tet[0][0], tet[3][1] - tet[0][1], tet[3][2] - tet[0][2]];
	let vp = [pt[0] - tet[0][0], pt[1] - tet[0][1], pt[2] - tet[0][2]];
	
	// Compute determinant for volume
	let det = v0[0] * (v1[1] * v2[2] - v1[2] * v2[1]) - 
	          v0[1] * (v1[0] * v2[2] - v1[2] * v2[0]) + 
	          v0[2] * (v1[0] * v2[1] - v1[1] * v2[0]);
	          
	if det.abs() < 1e-12 { return false; } // Degenerate tetrahedron
	
	// Solve for barycentric coordinates using Cramer's rule
	let inv_det = 1.0 / det;
	
	let d1 = vp[0] * (v1[1] * v2[2] - v1[2] * v2[1]) - 
	         vp[1] * (v1[0] * v2[2] - vp[2] * v2[0]) +
	         vp[2] * (v1[0] * v2[1] - v1[1] * v2[0]);
	let u = d1 * inv_det;
	
	let d2 = v0[0] * (vp[1] * v2[2] - vp[2] * v2[1]) - 
	         v0[1] * (vp[0] * v2[2] - vp[2] * v2[0]) + 
	         v0[2] * (vp[0] * v2[1] - vp[1] * v2[0]);
	let v = d2 * inv_det;
	
	let d3 = v0[0] * (v1[1] * vp[2] - v1[2] * vp[1]) - 
	         v0[1] * (v1[0] * vp[2] - v1[2] * vp[0]) + 
	         v0[2] * (v1[0] * vp[1] - v1[1] * vp[0]);
	let w = d3 * inv_det;
	
	// Check if point is inside tetrahedron
	u >= 0.0 && v >= 0.0 && w >= 0.0 && (u + v + w) <= 1.0
}

fn estimate_element_size(pts: &[[f64;3]]) -> f64 {
	// Estimate characteristic size of element by computing average edge length
	let mut total_length = 0.0;
	let mut edge_count = 0;
	
	// Calculate all edge lengths for tetrahedron (6 edges)
	for i in 0..4 {
		for j in (i+1)..4 {
			let dx = pts[i][0] - pts[j][0];
			let dy = pts[i][1] - pts[j][1]; 
			let dz = pts[i][2] - pts[j][2];
			total_length += (dx*dx + dy*dy + dz*dz).sqrt();
			edge_count += 1;
		}
	}
	
	total_length / edge_count as f64
}

fn integrate_tet10(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Proper Tet10 integration with quadratic shape functions
	// Tet10 node ordering: 4 corners (0,1,2,3) + 6 mid-edges (4,5,6,7,8,9)
	match p.integration.as_deref() {
		Some("E") => integrate_tet10_e_mode(pts, p, vol),
		_ => integrate_tet10_hu_mode(pts, p, vol),
	}
}

fn integrate_tet10_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use adaptive sampling for high-density regions
	let mut sum = 0.0;
	let mut count = 0;
	
	// First pass: coarse sampling to detect density range
	let coarse_samples = p.int_steps.max(4);
	let mut min_hu = f64::INFINITY;
	let mut max_hu = f64::NEG_INFINITY;
	let mut sample_hus = Vec::new();
	
	let coarse_step = 1.0 / (coarse_samples as f64);
	for i in 0..coarse_samples {
		for j in 0..coarse_samples {
			for k in 0..coarse_samples {
				let u = (i as f64 + 0.5) * coarse_step;
				let v = (j as f64 + 0.5) * coarse_step;
				let w = (k as f64 + 0.5) * coarse_step;
				
				if u + v + w <= 1.0 {
					let l = 1.0 - u - v - w;
					let point = tet10_shape(u, v, w, l, pts);
					let hu = vol.trilinear(point[0], point[1], point[2]);
					min_hu = min_hu.min(hu);
					max_hu = max_hu.max(hu);
					sample_hus.push(hu);
				}
			}
		}
	}
	
	// Determine sampling density based on HU range and gradients
	let hu_range = max_hu - min_hu;
	let high_density_threshold = 1500.0; // Cortical bone threshold
	
	// Use higher sampling for high-density or high-gradient regions
	let adaptive_factor = if max_hu > high_density_threshold || hu_range > 500.0 {
		2.0 // Double sampling for high-density/high-gradient regions
	} else if hu_range > 200.0 {
		1.5 // 50% more sampling for moderate gradients
	} else {
		1.0 // Normal sampling for homogeneous regions
	};
	
	let samples = ((p.int_steps as f64 * 1.3 * adaptive_factor) as usize).max(p.int_steps);
	let step = 1.0 / (samples as f64);
	
	// Main integration with adaptive sampling density
	for i in 0..samples {
		for j in 0..samples {
			for k in 0..samples {
				let u = (i as f64 + 0.5) * step;
				let v = (j as f64 + 0.5) * step;
				let w = (k as f64 + 0.5) * step;
				
				if u + v + w <= 1.0 {
					let l = 1.0 - u - v - w;
					let point = tet10_shape(u, v, w, l, pts);
					let hu = vol.trilinear(point[0], point[1], point[2]);
					sum += pipeline_e_mode(hu, p);
					count += 1;
				}
			}
		}
	}
	
	if count > 0 { sum / count as f64 } else { 0.0 }
}

fn integrate_tet10_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// HU mode with adaptive sampling
	let mut sum_hu = 0.0;
	let mut count = 0;
	
	// First pass: detect density range
	let coarse_samples = p.int_steps.max(4);
	let mut min_hu = f64::INFINITY;
	let mut max_hu = f64::NEG_INFINITY;
	
	let coarse_step = 1.0 / (coarse_samples as f64);
	for i in 0..coarse_samples {
		for j in 0..coarse_samples {
			for k in 0..coarse_samples {
				let u = (i as f64 + 0.5) * coarse_step;
				let v = (j as f64 + 0.5) * coarse_step;
				let w = (k as f64 + 0.5) * coarse_step;
				
				if u + v + w <= 1.0 {
					let l = 1.0 - u - v - w;
					let point = tet10_shape(u, v, w, l, pts);
					let hu = vol.trilinear(point[0], point[1], point[2]);
					min_hu = min_hu.min(hu);
					max_hu = max_hu.max(hu);
				}
			}
		}
	}
	
	// Adaptive sampling based on density range
	let hu_range = max_hu - min_hu;
	let high_density_threshold = 1500.0;
	
	let adaptive_factor = if max_hu > high_density_threshold || hu_range > 500.0 {
		2.0
	} else if hu_range > 200.0 {
		1.5
	} else {
		1.0
	};
	
	let samples = ((p.int_steps as f64 * 1.3 * adaptive_factor) as usize).max(p.int_steps);
	let step = 1.0 / (samples as f64);
	
	for i in 0..samples {
		for j in 0..samples {
			for k in 0..samples {
				let u = (i as f64 + 0.5) * step;
				let v = (j as f64 + 0.5) * step;
				let w = (k as f64 + 0.5) * step;
				
				if u + v + w <= 1.0 {
					let l = 1.0 - u - v - w;
					let point = tet10_shape(u, v, w, l, pts);
					let hu = vol.trilinear(point[0], point[1], point[2]);
					sum_hu += hu;
					count += 1;
				}
			}
		}
	}
	
	if count > 0 {
		let avg_hu = sum_hu / count as f64;
		pipeline_hu_mode(avg_hu, p)
	} else { 0.0 }
}

// Tet10 quadratic shape functions
// Node layout: corners 0,1,2,3 + mid-edges 4(0-1), 5(1-2), 6(2-0), 7(0-3), 8(1-3), 9(2-3)
fn tet10_shape(u: f64, v: f64, w: f64, l: f64, pts: &[[f64;3]]) -> [f64;3] {
	// Quadratic shape functions for 10-node tetrahedron
	let n = [
		// Corner nodes (linear parts)
		l * (2.0*l - 1.0), // N0
		u * (2.0*u - 1.0), // N1  
		v * (2.0*v - 1.0), // N2
		w * (2.0*w - 1.0), // N3
		// Mid-edge nodes (quadratic parts)
		4.0*l*u,          // N4 (edge 0-1)
		4.0*u*v,          // N5 (edge 1-2)
		4.0*v*l,          // N6 (edge 2-0) 
		4.0*l*w,          // N7 (edge 0-3)
		4.0*u*w,          // N8 (edge 1-3)
		4.0*v*w,          // N9 (edge 2-3)
	];
	
	let mut x = [0.0; 3];
	for i in 0..10 {
		x[0] += n[i] * pts[i][0];
		x[1] += n[i] * pts[i][1]; 
		x[2] += n[i] * pts[i][2];
	}
	x
}

fn integrate_hex8(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	match p.integration.as_deref() {
		Some("E") => integrate_hex8_e_mode(pts, p, vol),
		_ => integrate_hex8_hu_mode(pts, p, vol),
	}
}

fn integrate_hex8_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use hexahedral integration with sampling density similar to tetrahedra
	let mut sum = 0.0;
	let mut count = 0;
	
	// Use fewer samples to match tetrahedral density more closely
	let samples_per_dim = (p.int_steps as f64).cbrt().ceil() as usize + 1;
	let step = 2.0 / (samples_per_dim as f64);
	let start = -1.0 + step / 2.0;
	
	for i in 0..samples_per_dim {
		for j in 0..samples_per_dim {
			for k in 0..samples_per_dim {
				let r = start + i as f64 * step;
				let s = start + j as f64 * step;
				let t = start + k as f64 * step;
				
				// Use hexahedral shape functions for accurate interpolation
				let point = hex_shape(r, s, t, pts);
				let hu = vol.trilinear(point[0], point[1], point[2]);
				sum += pipeline_e_mode(hu, p);
				count += 1;
			}
		}
	}
	
	if count > 0 { sum / count as f64 } else { 0.0 }
}

fn integrate_hex8_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use hexahedral integration with sampling density similar to tetrahedra
	let mut sum_hu = 0.0;
	let mut count = 0;
	
	let samples_per_dim = (p.int_steps as f64).cbrt().ceil() as usize + 1;
	let step = 2.0 / (samples_per_dim as f64);
	let start = -1.0 + step / 2.0;
	
	for i in 0..samples_per_dim {
		for j in 0..samples_per_dim {
			for k in 0..samples_per_dim {
				let r = start + i as f64 * step;
				let s = start + j as f64 * step;
				let t = start + k as f64 * step;
				
				let point = hex_shape(r, s, t, pts);
				sum_hu += vol.trilinear(point[0], point[1], point[2]);
				count += 1;
			}
		}
	}
	
	if count > 0 {
		let avg_hu = sum_hu / count as f64;
		pipeline_hu_mode(avg_hu, p)
	} else { 0.0 }
}

fn integrate_wedge6(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	match p.integration.as_deref() {
		Some("E") => integrate_wedge6_e_mode(pts, p, vol),
		_ => integrate_wedge6_hu_mode(pts, p, vol),
	}
}

fn integrate_wedge6_e_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use wedge integration with appropriate sampling density
	let mut sum = 0.0;
	let mut count = 0;
	
	// Use sampling density similar to tetrahedral elements
	let samples_base = (p.int_steps as f64 * 1.2) as usize; // Slightly higher for wedge
	let samples_height = (p.int_steps as f64 * 0.8) as usize; // Fewer height samples
	
	let step_base = 1.0 / (samples_base as f64);
	let step_height = 2.0 / (samples_height as f64);
	
	for i in 0..samples_base {
		for j in 0..samples_base {
			for k in 0..samples_height {
				let r = (i as f64 + 0.5) * step_base;
				let s = (j as f64 + 0.5) * step_base;
				let t = (k as f64 + 0.5) * step_height - 1.0; // Map t to [-1,1]
				
				// Check if point is inside wedge (r+s <= 1)
				if r + s <= 1.0 {
					let point = wedge_shape(r, s, t, pts);
					let hu = vol.trilinear(point[0], point[1], point[2]);
					sum += pipeline_e_mode(hu, p);
					count += 1;
				}
			}
		}
	}
	
	if count > 0 { sum / count as f64 } else { 0.0 }
}

fn integrate_wedge6_hu_mode(pts: &[[f64;3]], p: &Params, vol: &Volume) -> f64 {
	// Use wedge integration with appropriate sampling density
	let mut sum_hu = 0.0;
	let mut count = 0;
	
	let samples_base = (p.int_steps as f64 * 1.2) as usize;
	let samples_height = (p.int_steps as f64 * 0.8) as usize;
	
	let step_base = 1.0 / (samples_base as f64);
	let step_height = 2.0 / (samples_height as f64);
	
	for i in 0..samples_base {
		for j in 0..samples_base {
			for k in 0..samples_height {
				let r = (i as f64 + 0.5) * step_base;
				let s = (j as f64 + 0.5) * step_base;
				let t = (k as f64 + 0.5) * step_height - 1.0;
				
				if r + s <= 1.0 {
					let point = wedge_shape(r, s, t, pts);
					sum_hu += vol.trilinear(point[0], point[1], point[2]);
					count += 1;
				}
			}
		}
	}
	
	if count > 0 {
		let avg_hu = sum_hu / count as f64;
		pipeline_hu_mode(avg_hu, p)
	} else { 0.0 }
}

// E mode: Apply equation chain at each integration point, then average the results
fn pipeline_e_mode(hu: f64, p: &Params) -> f64 { 
	// Apply full equation chain: HU -> qct -> ash -> modulus
	// Ensure we handle invalid/extreme HU values properly
	if !hu.is_finite() { return p.min_val; }
	
	let q = (p.rho_qct_a + p.rho_qct_b * hu).max(p.min_val);
	
	let ash = if !p.calibration_correct { 
		q 
	} else {
		match p.num_ct_param.as_deref() {
			Some("single") => {
				let a = p.rho_asha1.unwrap_or(0.0);
				let b = p.rho_ashb1.unwrap_or(1.0);
				(a + b * q).max(p.min_val)
			},
			Some("triple") => {
				let q1 = p.rho_thresh1.unwrap_or(0.0);
				let q2 = p.rho_thresh2.unwrap_or(q1 + 0.1); // Ensure q2 > q1
				let (a, b) = if q < q1 { 
					(p.rho_asha1.unwrap_or(0.0), p.rho_ashb1.unwrap_or(1.0)) 
				} else if q <= q2 { 
					(p.rho_asha2.unwrap_or(0.0), p.rho_ashb2.unwrap_or(1.0)) 
				} else { 
					(p.rho_asha3.unwrap_or(0.0), p.rho_ashb3.unwrap_or(1.0)) 
				};
				(a + b * q).max(p.min_val)
			},
			_ => q
		}
	};
	
	// Calculate modulus
	if p.num_e_param == "single" {
		// Ensure ash is valid before power calculation
		if ash <= 0.0 || !ash.is_finite() { return p.min_val; }
		let res = p.ea1 + p.eb1 * ash.powf(p.ec1);
		if res.is_finite() && res > 0.0 { res } else { p.min_val }
	} else {
		// Triple parameter mode - FIXED: Compare modulus values to ethresh, not ash values
		if ash <= 0.0 || !ash.is_finite() { return p.min_val; }
		
		// Calculate modulus with first parameter set
		let e1 = p.ea1 + p.eb1 * ash.powf(p.ec1);
		if !e1.is_finite() { return p.min_val; }
		
		// Use modulus thresholds to determine which parameter set to use
		let eth1 = p.ethresh1.unwrap_or(1000.0); // Default modulus threshold
		let eth2 = p.ethresh2.unwrap_or(eth1 + 1000.0); // Ensure eth2 > eth1
		
		let res = if e1 < eth1 {
			e1  // Use first parameter set result
		} else if e1 <= eth2 {
			// Use second parameter set
			let e2 = p.ea2.unwrap_or(0.0) + p.eb2.unwrap_or(0.0) * ash.powf(p.ec2.unwrap_or(1.0));
			if e2.is_finite() { e2 } else { e1 }
		} else {
			// Use third parameter set  
			let e3 = p.ea3.unwrap_or(0.0) + p.eb3.unwrap_or(0.0) * ash.powf(p.ec3.unwrap_or(1.0));
			if e3.is_finite() { e3 } else { e1 }
		};
		
		if res.is_finite() && res > 0.0 { res.max(p.min_val) } else { p.min_val }
	}
}

// HU mode: Average HU values first, then apply equation chain once  
fn pipeline_hu_mode(hu: f64, p: &Params) -> f64 {
	// Same as E mode but conceptually different - HU is already averaged
	pipeline_e_mode(hu, p)
}

// Legacy pipeline function for backward compatibility

fn pipeline(hu: f64, p: &Params) -> f64 { 
	match p.integration.as_deref() { 
		Some("E") => pipeline_e_mode(hu, p),
		_ => pipeline_hu_mode(hu, p),
	}
}

fn hex_shape(r:f64,s:f64,t:f64, pts:&[[f64;3]]) -> [f64;3] { // linear hex interpolation
	let n = [ ((1.0-r)*(1.0-s)*(1.0-t))/8.0, ((1.0+r)*(1.0-s)*(1.0-t))/8.0, ((1.0+r)*(1.0+s)*(1.0-t))/8.0, ((1.0-r)*(1.0+s)*(1.0-t))/8.0, ((1.0-r)*(1.0-s)*(1.0+t))/8.0, ((1.0+r)*(1.0-s)*(1.0+t))/8.0, ((1.0+r)*(1.0+s)*(1.0+t))/8.0, ((1.0-r)*(1.0+s)*(1.0+t))/8.0 ];
	let mut x=[0.0;3]; for (i,nw) in n.iter().enumerate() { x[0]+=nw*pts[i][0]; x[1]+=nw*pts[i][1]; x[2]+=nw*pts[i][2]; } x }
fn wedge_shape(r:f64,s:f64,t:f64, pts:&[[f64;3]]) -> [f64;3] { // linear wedge
	let w = [ (1.0-r-s)*(1.0-t), s*(1.0-t), r*(1.0-t), (1.0-r-s)*t, s*t, r*t ];
	let mut x=[0.0;3]; for i in 0..6 { x[0]+=w[i]*pts[i][0]; x[1]+=w[i]*pts[i][1]; x[2]+=w[i]*pts[i][2]; } x }

// Grouping / binning analogous to Python _limit_num_materials
pub fn group_moduli(moduli: &[f64], params: &Params) -> Vec<f64> {
	if let Some(gap) = params.gap_value && gap == 0.0 {
		return moduli.iter().map(|m| m.max(params.min_val)).collect();
	}

	// Clamp all values to min_val first
	let clamped_moduli: Vec<f64> = moduli.iter().map(|&m| m.max(params.min_val)).collect();

	match (params.gap_value, params.num_materials) {
		(Some(gap_value), None) => {
			// Equal width gap value binning
			let min_clamped = clamped_moduli.iter().cloned().fold(f64::INFINITY, f64::min);
			let max_clamped = clamped_moduli.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
			if max_clamped <= min_clamped {
				return vec![min_clamped; moduli.len()];
			}

			let bin_start = (min_clamped / gap_value).floor() * gap_value;
			let mut bin_end = (max_clamped / gap_value).ceil() * gap_value;
			if (bin_end - max_clamped).abs() < 1e-9 {
				bin_end += gap_value;
			}
			let num_bins = ((bin_end - bin_start) / gap_value).round() as i32;
			if num_bins <= 0 {
				return clamped_moduli;
			}

			const BIN_EPS: f64 = 1e-9;
			let mut bin_contents: Vec<Vec<f64>> = vec![Vec::new(); num_bins as usize];

			let to_bin_idx = |value: f64| -> usize {
				let normalised = (value - bin_start) / gap_value;
				let raw = (normalised + BIN_EPS).floor();
				let mut bin_idx = raw as i32;
				if bin_idx < 0 { bin_idx = 0; }
				if bin_idx >= num_bins { bin_idx = num_bins - 1; }
				bin_idx as usize
			};

			for &clamped_m in &clamped_moduli {
				let bin_idx = to_bin_idx(clamped_m);
				bin_contents[bin_idx].push(clamped_m);
			}

			let mut bin_representatives: Vec<f64> = Vec::with_capacity(num_bins as usize);
			for (idx, values) in bin_contents.iter().enumerate() {
				let representative = if values.is_empty() {
					bin_start + (idx as f64 + 0.5) * gap_value
				} else {
					match params.grouping_density.as_str() {
						"max" => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
						"min" => values.iter().cloned().fold(f64::INFINITY, f64::min),
						"mid" => bin_start + (idx as f64 + 0.5) * gap_value,
						"median" => {
							let mut sorted = values.clone();
							sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
							let len = sorted.len();
							if len % 2 == 0 {
								(sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
							} else {
								sorted[len / 2]
							}
						},
						_ => values.iter().sum::<f64>() / values.len() as f64,
					}
				};
				bin_representatives.push(representative);
			}

			clamped_moduli.iter().map(|&m| bin_representatives[to_bin_idx(m)]).collect()
		}
		(None, Some(num_materials)) => {
			// Exact number of material binning
			// worst idea to impliment, I cannot make this give exactly the number requested.
			let len = clamped_moduli.len();
			if len == 0 { return Vec::new(); }

			// Dedup
			let mut unique_values = clamped_moduli.clone();
			unique_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
			unique_values.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

			let k = (num_materials).min(unique_values.len());
			if k <= 1 {
				let mean = clamped_moduli.iter().sum::<f64>() / len as f64;
				return vec![mean; len];
			}

			let min_val = unique_values[0];
			let max_val = unique_values[unique_values.len() - 1];
			let mut centroids = vec![0.0; k];
			for (i, centroid) in centroids.iter_mut().enumerate().take(k) {
				let fraction = i as f64 / (k - 1) as f64;
				*centroid = min_val + fraction * (max_val - min_val);
			}

			let mut assignments = vec![0; len];

			for _ in 0..50 {
				for (i, &val) in clamped_moduli.iter().enumerate() {
					let mut min_dist = f64::INFINITY;
					let mut closest_centroid = 0;
					for (c_idx, &centroid) in centroids.iter().enumerate() {
						let dist = (val - centroid).abs();
						if dist < min_dist {
							min_dist = dist;
							closest_centroid = c_idx;
						}
					
					}
					assignments[i] = closest_centroid;
				}

				let mut new_sums = vec![0.0; k];
				let mut new_counts = vec![0; k];
				for (i, &val) in clamped_moduli.iter().enumerate() {
					let c_idx = assignments[i];
					new_sums[c_idx] += val;
					new_counts[c_idx] += 1;
				}

				let mut repaired = false;
				for c_idx in 0..k { 
					if new_counts[c_idx] == 0 {
						let mut max_residual = -1.0;
						let mut fallback_val = min_val;
						for (i, &val) in clamped_moduli.iter().enumerate() {
							let dist = (val - centroids[assignments[i]]).abs();
							if dist > max_residual {
								max_residual = dist;
								fallback_val = val;
							}
						}
						centroids[c_idx] = fallback_val;
						repaired = true;
						break;
					}
				}

				if repaired { continue; }

				let mut changed = false;
				for c_idx in 0..k { 
					let target = new_sums[c_idx] / new_counts[c_idx] as f64;
					if (centroids[c_idx] - target).abs() > 1e-9 {
						centroids[c_idx] = target;
						changed = true;
					}
				}

				if !changed { break; }
			}

			let mut cluster_contents: Vec<Vec<f64>> = vec![Vec::new(); k];
			for (i, &val) in clamped_moduli.iter().enumerate() {
				cluster_contents[assignments[i]].push(val);
			}

			let mut cluster_representatives = vec![0.0; k];
			for (c_idx, values) in cluster_contents.iter().enumerate() {
				if values.is_empty() { 
					cluster_representatives[c_idx] = centroids[c_idx];
					continue;
				}
				let mut sorted_vals = values.clone();
				sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
				let b_len = sorted_vals.len();

				cluster_representatives[c_idx] = match params.grouping_density.as_str() {
					"max" => *sorted_vals.last().unwrap(),
					"min" => *sorted_vals.first().unwrap(),
					"mid" => (*&sorted_vals.first().unwrap() + *&sorted_vals.last().unwrap()) / 2.0,
					"median" => {
						if b_len % 2 == 0 {
							(sorted_vals[b_len / 2 - 1] + sorted_vals[b_len / 2]) / 2.0
						} else {
							sorted_vals[b_len / 2]
						}
					},
					_ => sorted_vals.iter().sum::<f64>() / b_len as f64,
				};
			}

			clamped_moduli.iter().map(|&val| cluster_representatives[assignments[clamped_moduli.iter().position(|&x| x == val).unwrap()]]).collect()
		}
		_ => clamped_moduli,
	}
}

/// Apply density back calculation to a vector of elastic modulus values
/// This function takes elastic modulus values and optionally applies density back calculation
/// to improve the predicted values using the existing density-elasticity relationship parameters
pub fn apply_density_back_calculation(moduli: &[f64], params: &Params) -> Vec<f64> {
	if params.num_e_param == "triple" && params.ea2.is_some() && params.eb2.is_some() && params.ec2.is_some() {
		moduli.iter().map(|&elasticity| {
			// Apply density back calculation using existing parameters
			let density = params.density_back_calculation(elasticity);
			
			// Recalculate modulus from the back-calculated density to improve accuracy
			params.modulus(density)
		}).collect()
	} else {
		// Return original values if triple parameters not available
		moduli.to_vec()
	}
}

