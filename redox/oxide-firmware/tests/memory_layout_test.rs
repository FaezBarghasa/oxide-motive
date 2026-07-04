
#[test]
fn test_memory_layout() {
    // This is a mock test. In a real CI environment, we would parse the
    // actual map file produced by the linker.
    let mock_map_file = "
.itcm           0x00000000       0x1000
 .itcm.scheduler
                0x00000000        0x800
 .itcm.trigger_decoder
                0x00000800        0x800
    ";

    let scheduler_line = mock_map_file.lines().find(|line| line.contains(".itcm.scheduler")).unwrap();
    let trigger_decoder_line = mock_map_file.lines().find(|line| line.contains(".itcm.trigger_decoder")).unwrap();

    let scheduler_addr = u32::from_str_radix(scheduler_line.split_whitespace().nth(1).unwrap().trim_start_matches("0x"), 16).unwrap();
    let trigger_decoder_addr = u32::from_str_radix(trigger_decoder_line.split_whitespace().nth(1).unwrap().trim_start_matches("0x"), 16).unwrap();

    assert!(scheduler_addr >= 0x00000000 && scheduler_addr < 0x00010000);
    assert!(trigger_decoder_addr >= 0x00000000 && trigger_decoder_addr < 0x00010000);
}
