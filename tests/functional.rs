use std::path::Path;

#[test]
fn run_example_pipeline() {
    // Paths relative to crate root
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let params = root.join("tests").join("small_ansys_test").join("pm2_parameters.toml");
    let ct = root.join("tests").join("small_ansys_test").join("PM2.vtk");
    let mesh = root.join("tests").join("small_ansys_test").join("RFemur2.cdb");
    assert!(params.exists(), "param file missing: {:?}", params);
    assert!(ct.exists(), "ct file missing: {:?}", ct);
    assert!(mesh.exists(), "mesh file missing: {:?}", mesh);

    let out = luma::run(params.to_str().unwrap(), ct.to_str().unwrap(), mesh.to_str().unwrap())
        .expect("pipeline run");
    let out_path = root.join(out);
    assert!(out_path.exists(), "output file not created: {:?}", out_path);

    // Basic sanity: file size > 0
    let meta = std::fs::metadata(&out_path).expect("metadata");
    assert!(meta.len() > 0, "output file empty");
}

#[test]
fn test_cdb_format_parsing() {
    use luma::mesh::ansys_cdb::parse_cdb;
    
    let cdb_path = "tests/medium_ansys_test/RFemur24.cdb";
    match parse_cdb(cdb_path, &[]) {
        Ok(mesh) => {
            println!("Successfully loaded {} parts", mesh.parts.len());
            
            if let Some(format_info) = &mesh.mesh_format_info {
                if let Some(nblock_format) = &format_info.nblock_format {
                    println!("NBLOCK format: {}", nblock_format);
                } else {
                    println!("No NBLOCK format found");
                }
                if let Some(eblock_format) = &format_info.eblock_format {
                    println!("EBLOCK format: {}", eblock_format);
                } else {
                    println!("No EBLOCK format found");
                }
            } else {
                println!("No format info found");
            }
            
            // Test format parsing
            let format = mesh.mesh_format_info
                .as_ref()
                .and_then(|info| info.nblock_format.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("(3i9,6e21.13e3)");
            println!("Using format: {}", format);
        }
        Err(e) => {
            println!("Failed to load CDB file: {:?}", e);
            panic!("CDB parsing failed with error: {:?}", e);
        }
    }
}

#[test]
fn test_dicom_sequence_loading() {
    use luma::volume::load_volume;
    
    let dicom_path = "tests/small_ansys_test/pm2_dicom_files";
    
    match load_volume(dicom_path) {
        Ok(volume) => {
            println!("Successfully loaded DICOM sequence from: {}", dicom_path);
            
            // Basic validation tests
            assert!(!volume.x.is_empty(), "X coordinates should not be empty");
            assert!(!volume.y.is_empty(), "Y coordinates should not be empty");
            assert!(!volume.z.is_empty(), "Z coordinates should not be empty");
            assert!(!volume.scalars.is_empty(), "Scalar data should not be empty");
            
            // Test coordinate consistency
            let expected_voxels = volume.x.len() * volume.y.len() * volume.z.len();
            assert_eq!(
                volume.scalars.len(), 
                expected_voxels, 
                "Scalar data length {} should match coordinate grid size {}",
                volume.scalars.len(),
                expected_voxels
            );
            
            // Test that coordinates are sorted (required for trilinear interpolation)
            for i in 1..volume.x.len() {
                assert!(
                    volume.x[i] > volume.x[i-1], 
                    "X coordinates must be sorted: x[{}]={} <= x[{}]={}",
                    i-1, volume.x[i-1], i, volume.x[i]
                );
            }
            for i in 1..volume.y.len() {
                assert!(
                    volume.y[i] > volume.y[i-1], 
                    "Y coordinates must be sorted: y[{}]={} <= y[{}]={}",
                    i-1, volume.y[i-1], i, volume.y[i]
                );
            }
            for i in 1..volume.z.len() {
                assert!(
                    volume.z[i] > volume.z[i-1], 
                    "Z coordinates must be sorted: z[{}]={} <= z[{}]={}",
                    i-1, volume.z[i-1], i, volume.z[i]
                );
            }
            
            println!("Volume dimensions: {} x {} x {} = {} voxels", 
                     volume.x.len(), volume.y.len(), volume.z.len(), expected_voxels);
            println!("Scalar range: {} to {}", 
                     volume.scalars.iter().fold(f32::INFINITY, |a, &b| a.min(b)),
                     volume.scalars.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
            
        }
        Err(e) => {
            panic!("Failed to load DICOM sequence: {:?}", e);
        }
    }
}

#[test]
fn test_dicom_sequence_interpolation() {
    use luma::volume::load_volume;
    
    let dicom_path = "tests/small_ansys_test/pm2_dicom_files";
    
    let volume = load_volume(dicom_path)
        .expect("Failed to load DICOM sequence for interpolation test");
    
    // Test trilinear interpolation at known grid points
    let mid_x = volume.x.len() / 2;
    let mid_y = volume.y.len() / 2;
    let mid_z = volume.z.len() / 2;
    
    let test_x = volume.x[mid_x];
    let test_y = volume.y[mid_y];
    let test_z = volume.z[mid_z];
    
    let interpolated = volume.trilinear(test_x, test_y, test_z);
    let expected_idx = mid_x + mid_y * volume.x.len() + mid_z * volume.x.len() * volume.y.len();
    let expected = volume.scalars[expected_idx] as f64;
    
    let tolerance = 1e-6;
    assert!(
        (interpolated - expected).abs() < tolerance,
        "Interpolation at grid point should equal grid value: got {}, expected {}",
        interpolated, expected
    );
    
    // Test interpolation between points
    let interp_x = (volume.x[0] + volume.x[1]) * 0.5;
    let interp_y = (volume.y[0] + volume.y[1]) * 0.5;
    let interp_z = (volume.z[0] + volume.z[1]) * 0.5;
    
    let result = volume.trilinear(interp_x, interp_y, interp_z);
    assert!(result.is_finite(), "Interpolated value should be finite, got {}", result);
    
    println!("Interpolation tests passed successfully");
}

#[test] 
fn test_dicom_sequence_edge_cases() {
    use luma::volume::load_volume;
    
    // Test non-existent directory
    match load_volume("tests/nonexistent_dicom_dir") {
        Ok(_) => panic!("Should fail for non-existent directory"),
        Err(e) => {
            println!("Correctly failed for non-existent directory: {}", e);
        }
    }
    
    // Test empty directory (create a temporary empty directory)
    let temp_dir = std::env::temp_dir().join("empty_dicom_test");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    
    match load_volume(temp_dir.to_str().unwrap()) {
        Ok(_) => panic!("Should fail for empty directory"),
        Err(e) => {
            println!("Correctly failed for empty directory: {}", e);
            assert!(e.to_string().contains("No DICOM files found"));
        }
    }
    
    // Clean up temp directory
    std::fs::remove_dir_all(&temp_dir).ok();
    
    println!("Edge case tests passed successfully");
}

#[test]
fn test_dicom_metadata_extraction() {
    use dicom_object::open_file;
    use std::fs;
    
    let dicom_dir = "tests/small_ansys_test/pm2_dicom_files";
    
    // Get list of DICOM files
    let mut entries: Vec<_> = fs::read_dir(dicom_dir)
        .expect("Failed to read DICOM directory")
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());
    
    assert!(!entries.is_empty(), "No files found in DICOM directory");
    
    let first_file = entries.first().unwrap();
    let dicom_obj = open_file(first_file.path())
        .expect("Failed to open first DICOM file");
    
    // Test required metadata extraction
    let rows = dicom_obj.element_by_name("Rows")
        .expect("Rows tag should be present")
        .to_int::<u16>()
        .expect("Rows should be valid integer");
    
    let cols = dicom_obj.element_by_name("Columns")
        .expect("Columns tag should be present")
        .to_int::<u16>()
        .expect("Columns should be valid integer");
    
    let pixel_spacing = dicom_obj.element_by_name("PixelSpacing")
        .expect("PixelSpacing tag should be present")
        .to_multi_float64()
        .expect("PixelSpacing should be valid float array");
    
    let img_pos = dicom_obj.element_by_name("ImagePositionPatient")
        .expect("ImagePositionPatient tag should be present")
        .to_multi_float64()
        .expect("ImagePositionPatient should be valid float array");
    
    // Validate metadata ranges
    assert!(rows > 0, "Rows should be positive: {}", rows);
    assert!(cols > 0, "Columns should be positive: {}", cols);
    assert_eq!(pixel_spacing.len(), 2, "PixelSpacing should have 2 elements");
    assert_eq!(img_pos.len(), 3, "ImagePositionPatient should have 3 elements");
    assert!(pixel_spacing[0] > 0.0, "Row spacing should be positive: {}", pixel_spacing[0]);
    assert!(pixel_spacing[1] > 0.0, "Column spacing should be positive: {}", pixel_spacing[1]);
    
    // Test pixel data extraction
    let pixel_data = dicom_obj.element_by_name("PixelData")
        .expect("PixelData tag should be present")
        .to_bytes()
        .expect("PixelData should be valid byte array");
    
    let expected_bytes = (rows as usize) * (cols as usize) * 2; // 2 bytes per 16-bit pixel
    assert_eq!(
        pixel_data.len(), 
        expected_bytes,
        "Pixel data size {} should match expected size {}", 
        pixel_data.len(), 
        expected_bytes
    );
    
    println!("DICOM metadata extraction tests passed");
    println!("Image dimensions: {} x {}", rows, cols);
    println!("Pixel spacing: {:?}", pixel_spacing);
    println!("Image position: {:?}", img_pos);
}

#[test]
fn test_dicom_sequence_consistency() {
    use dicom_object::open_file;
    use std::fs;
    
    let dicom_dir = "tests/small_ansys_test/pm2_dicom_files";
    
    // Load all DICOM files and check consistency
    let mut entries: Vec<_> = fs::read_dir(dicom_dir)
        .expect("Failed to read DICOM directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    entries.sort_by_key(|e| e.path());
    
    let mut slice_positions = Vec::new();
    let mut first_rows = None;
    let mut first_cols = None;
    let mut first_spacing = None;
    
    for entry in &entries {
        if let Ok(dicom_obj) = open_file(entry.path()) {
            // Check image dimensions consistency
            let rows = dicom_obj.element_by_name("Rows")
                .ok().and_then(|e| e.to_int::<u16>().ok())
                .unwrap_or(0);
            let cols = dicom_obj.element_by_name("Columns")
                .ok().and_then(|e| e.to_int::<u16>().ok())
                .unwrap_or(0);
            
            if rows > 0 && cols > 0 {
                match (first_rows, first_cols) {
                    (None, None) => {
                        first_rows = Some(rows);
                        first_cols = Some(cols);
                    }
                    (Some(r), Some(c)) => {
                        assert_eq!(rows, r, "Inconsistent rows across slices: {} vs {}", rows, r);
                        assert_eq!(cols, c, "Inconsistent columns across slices: {} vs {}", cols, c);
                    }
                    _ => unreachable!()
                }
                
                // Check pixel spacing consistency
                if let Some(spacing) = dicom_obj.element_by_name("PixelSpacing")
                    .ok().and_then(|e| e.to_multi_float64().ok()) {
                    match first_spacing.as_ref() {
                        None => first_spacing = Some(spacing.clone()),
                        Some(first) => {
                            let tolerance = 1e-6;
                            assert!(
                                (spacing[0] - first[0]).abs() < tolerance,
                                "Inconsistent row spacing: {} vs {}", spacing[0], first[0]
                            );
                            assert!(
                                (spacing[1] - first[1]).abs() < tolerance,
                                "Inconsistent col spacing: {} vs {}", spacing[1], first[1]
                            );
                        }
                    }
                }
                
                // Collect slice positions for sorting validation
                if let Some(pos) = dicom_obj.element_by_name("ImagePositionPatient")
                    .ok().and_then(|e| e.to_multi_float64().ok()) {
                    if pos.len() >= 3 {
                        slice_positions.push(pos[2]); // Z coordinate
                    }
                }
            }
        }
    }
    
    // Verify we found valid slices
    assert!(!slice_positions.is_empty(), "No valid DICOM slices found");
    assert!(first_rows.is_some() && first_cols.is_some(), "No valid image dimensions found");
    
    // Check that slice positions are reasonable for sorting
    if slice_positions.len() > 1 {
        let mut sorted_positions = slice_positions.clone();
        sorted_positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Check for duplicates
        for i in 1..sorted_positions.len() {
            assert_ne!(
                sorted_positions[i], 
                sorted_positions[i-1],
                "Duplicate slice positions found: {}", sorted_positions[i]
            );
        }
        
        println!("Z positions range: {} to {}", 
                 sorted_positions.first().unwrap(), 
                 sorted_positions.last().unwrap());
    }
    
    println!("DICOM sequence consistency tests passed");
    println!("Found {} valid DICOM slices", slice_positions.len());
    println!("Image dimensions: {:?} x {:?}", first_rows, first_cols);
}

#[test]
fn test_small_ansys_with_dicom_integration() {
    use std::path::Path;
    
    // Test the complete pipeline using DICOM data with ANSYS mesh
    let test_dir = Path::new("tests/small_ansys_test");
      let params_path = test_dir.join("pm2_parameters.toml");
    let dicom_dir = test_dir.join("pm2_dicom_files");
    let mesh_path = test_dir.join("RFemur2.cdb");
    
    // Verify all required files exist
    assert!(params_path.exists(), "Parameters file missing: {:?}", params_path);
    assert!(dicom_dir.exists() && dicom_dir.is_dir(), "DICOM directory missing: {:?}", dicom_dir);
    assert!(mesh_path.exists(), "Mesh file missing: {:?}", mesh_path);
    
    // First test DICOM volume loading separately
    let volume_result = luma::volume::load_volume(dicom_dir.to_str().unwrap());
    match volume_result {
        Ok(volume) => {
            println!("Successfully loaded DICOM volume");
            println!("Volume dimensions: {} x {} x {}", 
                     volume.x.len(), volume.y.len(), volume.z.len());
            
            let total_voxels = volume.x.len() * volume.y.len() * volume.z.len();
            assert_eq!(volume.scalars.len(), total_voxels, 
                      "Scalar data length should match grid dimensions");
            
            // Check that we have reasonable coordinate ranges
            let x_range = volume.x.last().unwrap() - volume.x.first().unwrap();
            let y_range = volume.y.last().unwrap() - volume.y.first().unwrap();
            let z_range = volume.z.last().unwrap() - volume.z.first().unwrap();
            
            println!("Coordinate ranges: X={:.1}mm, Y={:.1}mm, Z={:.1}mm", 
                     x_range, y_range, z_range);
            
            assert!(x_range > 0.0 && y_range > 0.0 && z_range > 0.0, 
                   "All coordinate ranges should be positive");
            
            // Check scalar value range (should be reasonable for CT data)
            let min_val = volume.scalars.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = volume.scalars.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            println!("Scalar value range: {:.1} to {:.1}", min_val, max_val);
            
            // Test some interpolation points within the volume
            let mid_x = (volume.x.first().unwrap() + volume.x.last().unwrap()) / 2.0;
            let mid_y = (volume.y.first().unwrap() + volume.y.last().unwrap()) / 2.0;
            let mid_z = (volume.z.first().unwrap() + volume.z.last().unwrap()) / 2.0;
            
            let interpolated = volume.trilinear(mid_x, mid_y, mid_z);
            assert!(interpolated.is_finite(), "Interpolated value should be finite");
            println!("Interpolated value at center: {:.2}", interpolated);
        }
        Err(e) => panic!("Failed to load DICOM volume: {:?}", e)
    }
    
    // Now test the complete pipeline integration
    println!("Testing complete pipeline with DICOM + ANSYS mesh...");
    
    let pipeline_result = luma::run(
        params_path.to_str().unwrap(),
        dicom_dir.to_str().unwrap(),  // Use DICOM directory instead of VTK
        mesh_path.to_str().unwrap()
    );
    
    match pipeline_result {
        Ok(output_file) => {
            println!("Pipeline completed successfully!");
            println!("Output file: {}", output_file);
            
            // Verify output file was created and has content
            let output_path = Path::new(&output_file);
            assert!(output_path.exists(), "Output file should exist: {:?}", output_path);
            
            let metadata = std::fs::metadata(&output_path).expect("Failed to get file metadata");
            assert!(metadata.len() > 0, "Output file should not be empty");
            
            println!("Output file size: {} bytes", metadata.len());
            
            // Check that it's a valid file (basic validation)
            assert!(output_file.ends_with(".inp") || output_file.ends_with(".cdb"), 
                   "Output should be a mesh file");
            
            println!("DICOM + ANSYS integration test PASSED!");
        }
        Err(e) => {
            // Print detailed error information for debugging
            println!("Pipeline failed with error: {:?}", e);
            println!("Error details: {}", e);
            
            // Check if it's a specific type of error we can handle
            let error_msg = format!("{:?}", e);
            if error_msg.contains("No such file or directory") {
                panic!("File not found error - check that all test files exist");
            } else if error_msg.contains("DICOM") {
                panic!("DICOM-related error: {}", e);
            } else if error_msg.contains("mesh") || error_msg.contains("CDB") {
                panic!("Mesh processing error: {}", e);
            } else {
                panic!("Unexpected pipeline error: {}", e);
            }
        }
    }
}

#[test] 
fn test_dicom_vs_vtk_comparison() {
    use std::path::Path;
    
    // Compare DICOM loading vs VTK loading to ensure consistency
    let test_dir = Path::new("tests/small_ansys_test");
    let dicom_dir = test_dir.join("pm2_dicom_files");
    let vtk_file = test_dir.join("PM2.vtk");
    
    if !vtk_file.exists() {
        println!("VTK file not found, skipping comparison test");
        return;
    }
    
    // Load both volumes
    let dicom_volume = luma::volume::load_volume(dicom_dir.to_str().unwrap())
        .expect("Failed to load DICOM volume");
    let vtk_volume = luma::volume::load_volume(vtk_file.to_str().unwrap())
        .expect("Failed to load VTK volume");
    
    println!("DICOM volume dimensions: {} x {} x {}", 
             dicom_volume.x.len(), dicom_volume.y.len(), dicom_volume.z.len());
    println!("VTK volume dimensions: {} x {} x {}", 
             vtk_volume.x.len(), vtk_volume.y.len(), vtk_volume.z.len());
    
    // They don't need to be exactly the same, but should be in similar ranges
    let dicom_total = dicom_volume.x.len() * dicom_volume.y.len() * dicom_volume.z.len();
    let vtk_total = vtk_volume.x.len() * vtk_volume.y.len() * vtk_volume.z.len();
    
    println!("Total voxels - DICOM: {}, VTK: {}", dicom_total, vtk_total);
    
    // Check coordinate ranges
    let dicom_x_range = dicom_volume.x.last().unwrap() - dicom_volume.x.first().unwrap();
    let dicom_y_range = dicom_volume.y.last().unwrap() - dicom_volume.y.first().unwrap();
    let dicom_z_range = dicom_volume.z.last().unwrap() - dicom_volume.z.first().unwrap();
    
    let vtk_x_range = vtk_volume.x.last().unwrap() - vtk_volume.x.first().unwrap();
    let vtk_y_range = vtk_volume.y.last().unwrap() - vtk_volume.y.first().unwrap();
    let vtk_z_range = vtk_volume.z.last().unwrap() - vtk_volume.z.first().unwrap();
    
    println!("DICOM coordinate ranges: X={:.1}, Y={:.1}, Z={:.1}", 
             dicom_x_range, dicom_y_range, dicom_z_range);
    println!("VTK coordinate ranges: X={:.1}, Y={:.1}, Z={:.1}", 
             vtk_x_range, vtk_y_range, vtk_z_range);
    
    // Basic sanity checks - both should have reasonable coordinate ranges
    assert!(dicom_x_range > 0.0 && dicom_y_range > 0.0 && dicom_z_range > 0.0);
    assert!(vtk_x_range > 0.0 && vtk_y_range > 0.0 && vtk_z_range > 0.0);
    
    println!("DICOM vs VTK comparison completed successfully");
}

#[test]
fn test_dicom_coordinate_system_debug() {
    use std::path::Path;
    
    println!("=== DICOM Coordinate System Debug ===");
    
    let test_dir = Path::new("tests/small_ansys_test");
    let dicom_dir = test_dir.join("pm2_dicom_files");
    let vtk_file = test_dir.join("PM2.vtk");
    
    // Load both volumes
    let dicom_volume = luma::volume::load_volume(dicom_dir.to_str().unwrap())
        .expect("Failed to load DICOM volume");
    
    println!("DICOM Volume Analysis:");
    println!("  Dimensions: {} x {} x {}", 
             dicom_volume.x.len(), dicom_volume.y.len(), dicom_volume.z.len());
    
    // Print first and last few coordinate values to understand the system
    println!("  X coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
             dicom_volume.x[0], dicom_volume.x[1], dicom_volume.x[2],
             dicom_volume.x[dicom_volume.x.len()-3], dicom_volume.x[dicom_volume.x.len()-2], dicom_volume.x[dicom_volume.x.len()-1]);
    println!("  Y coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
             dicom_volume.y[0], dicom_volume.y[1], dicom_volume.y[2],
             dicom_volume.y[dicom_volume.y.len()-3], dicom_volume.y[dicom_volume.y.len()-2], dicom_volume.y[dicom_volume.y.len()-1]);
    println!("  Z coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
             dicom_volume.z[0], dicom_volume.z[1], dicom_volume.z[2],
             dicom_volume.z[dicom_volume.z.len()-3], dicom_volume.z[dicom_volume.z.len()-2], dicom_volume.z[dicom_volume.z.len()-1]);
    
    let dicom_x_range = dicom_volume.x.last().unwrap() - dicom_volume.x.first().unwrap();
    let dicom_y_range = dicom_volume.y.last().unwrap() - dicom_volume.y.first().unwrap();
    let dicom_z_range = dicom_volume.z.last().unwrap() - dicom_volume.z.first().unwrap();
    
    let dicom_x_spacing = if dicom_volume.x.len() > 1 { dicom_volume.x[1] - dicom_volume.x[0] } else { 0.0 };
    let dicom_y_spacing = if dicom_volume.y.len() > 1 { dicom_volume.y[1] - dicom_volume.y[0] } else { 0.0 };
    let dicom_z_spacing = if dicom_volume.z.len() > 1 { dicom_volume.z[1] - dicom_volume.z[0] } else { 0.0 };
    
    println!("  Coordinate ranges: X={:.3}mm, Y={:.3}mm, Z={:.3}mm", 
             dicom_x_range, dicom_y_range, dicom_z_range);
    println!("  Spacing: X={:.3}mm, Y={:.3}mm, Z={:.3}mm", 
             dicom_x_spacing, dicom_y_spacing, dicom_z_spacing);
    
    // Test some sample values
    let dicom_sample_idx = [0, dicom_volume.scalars.len()/4, dicom_volume.scalars.len()/2, dicom_volume.scalars.len()-1];
    println!("  Sample scalar values: {:?}", 
             dicom_sample_idx.iter().map(|&i| dicom_volume.scalars[i]).collect::<Vec<_>>());
    
    if vtk_file.exists() {
        let vtk_volume = luma::volume::load_volume(vtk_file.to_str().unwrap())
            .expect("Failed to load VTK volume");
        
        println!("\nVTK Volume Analysis:");
        println!("  Dimensions: {} x {} x {}", 
                 vtk_volume.x.len(), vtk_volume.y.len(), vtk_volume.z.len());
        
        println!("  X coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
                 vtk_volume.x[0], vtk_volume.x[1], vtk_volume.x[2],
                 vtk_volume.x[vtk_volume.x.len()-3], vtk_volume.x[vtk_volume.x.len()-2], vtk_volume.x[vtk_volume.x.len()-1]);
        println!("  Y coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
                 vtk_volume.y[0], vtk_volume.y[1], vtk_volume.y[2],
                 vtk_volume.y[vtk_volume.y.len()-3], vtk_volume.y[vtk_volume.y.len()-2], vtk_volume.y[vtk_volume.y.len()-1]);
        println!("  Z coordinates: [{:.3}, {:.3}, {:.3}, ..., {:.3}, {:.3}, {:.3}]",
                 vtk_volume.z[0], vtk_volume.z[1], vtk_volume.z[2],
                 vtk_volume.z[vtk_volume.z.len()-3], vtk_volume.z[vtk_volume.z.len()-2], vtk_volume.z[vtk_volume.z.len()-1]);
        
        let vtk_x_range = vtk_volume.x.last().unwrap() - vtk_volume.x.first().unwrap();
        let vtk_y_range = vtk_volume.y.last().unwrap() - vtk_volume.y.first().unwrap();
        let vtk_z_range = vtk_volume.z.last().unwrap() - vtk_volume.z.first().unwrap();
        
        let vtk_x_spacing = if vtk_volume.x.len() > 1 { vtk_volume.x[1] - vtk_volume.x[0] } else { 0.0 };
        let vtk_y_spacing = if vtk_volume.y.len() > 1 { vtk_volume.y[1] - vtk_volume.y[0] } else { 0.0 };
        let vtk_z_spacing = if vtk_volume.z.len() > 1 { vtk_volume.z[1] - vtk_volume.z[0] } else { 0.0 };
        
        println!("  Coordinate ranges: X={:.3}mm, Y={:.3}mm, Z={:.3}mm", 
                 vtk_x_range, vtk_y_range, vtk_z_range);
        println!("  Spacing: X={:.3}mm, Y={:.3}mm, Z={:.3}mm", 
                 vtk_x_spacing, vtk_y_spacing, vtk_z_spacing);
        
        // Test some sample values
        let vtk_sample_idx = [0, vtk_volume.scalars.len()/4, vtk_volume.scalars.len()/2, vtk_volume.scalars.len()-1];
        println!("  Sample scalar values: {:?}", 
                 vtk_sample_idx.iter().map(|&i| vtk_volume.scalars[i]).collect::<Vec<_>>());
        
        println!("\nComparison:");
        println!("  Dimension ratio (DICOM/VTK): {:.3} x {:.3} x {:.3}",
                 dicom_volume.x.len() as f64 / vtk_volume.x.len() as f64,
                 dicom_volume.y.len() as f64 / vtk_volume.y.len() as f64,
                 dicom_volume.z.len() as f64 / vtk_volume.z.len() as f64);
        
        println!("  Range ratio (DICOM/VTK): {:.3} x {:.3} x {:.3}",
                 dicom_x_range / vtk_x_range,
                 dicom_y_range / vtk_y_range,
                 dicom_z_range / vtk_z_range);
        
        println!("  Spacing ratio (DICOM/VTK): {:.3} x {:.3} x {:.3}",
                 dicom_x_spacing / vtk_x_spacing,
                 dicom_y_spacing / vtk_y_spacing,
                 dicom_z_spacing / vtk_z_spacing);
        
        // Check coordinate origin differences
        println!("  Origin difference (DICOM - VTK): [{:.3}, {:.3}, {:.3}]",
                 dicom_volume.x[0] - vtk_volume.x[0],
                 dicom_volume.y[0] - vtk_volume.y[0],
                 dicom_volume.z[0] - vtk_volume.z[0]);
    } else {
        println!("\nVTK file not found, skipping comparison");
    }
    
     println!("\n=== End Coordinate System Debug ===");
}
