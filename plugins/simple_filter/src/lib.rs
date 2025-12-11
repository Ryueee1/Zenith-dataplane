#[no_mangle]
pub extern "C" fn on_event(source_id: u32, seq_no: u64) -> i32 {
    // Simple logic: Allow only even sequence numbers
    if seq_no % 2 == 0 {
        1 // Accept
    } else {
        0 // Drop
    }
}
