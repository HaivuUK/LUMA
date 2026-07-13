// This is a simple test to demonstrate that log_status messages
// are only printed in debug builds, not in release builds.

fn main() {
    println!("Testing log_status behavior:");
    println!("- This is a regular println! that always appears");
    
    luma::log_info("This is an info message (always visible)");
    luma::log_status("This is a status message (debug only)");
    luma::log_success("This is a success message (always visible)");
    luma::log_error("This is an error message (always visible)");

    println!("Test complete!");
}