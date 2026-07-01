use oxide_protocol::{
    framing::{decode_frame, encode_frame},
    HostToMcu, McuToHost, SensorData,
};
use heapless::Vec;

#[test]
fn test_encode_decode_host_to_mcu() {
    let mut buf = [0u8; 256];

    let sync_request = HostToMcu::SyncRequest;
    let encoded_len = encode_frame(&sync_request, &mut buf).unwrap();
    let decoded: HostToMcu = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(sync_request, decoded);

    let schedule_event = HostToMcu::ScheduleEvent {
        channel: 1,
        timestamp_us: 123456789,
        duration_us: 1000,
    };
    let encoded_len = encode_frame(&schedule_event, &mut buf).unwrap();
    let decoded: HostToMcu = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(schedule_event, decoded);

    let config_update = HostToMcu::ConfigUpdate;
    let encoded_len = encode_frame(&config_update, &mut buf).unwrap();
    let decoded: HostToMcu = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(config_update, decoded);

    let heartbeat = HostToMcu::Heartbeat;
    let encoded_len = encode_frame(&heartbeat, &mut buf).unwrap();
    let decoded: HostToMcu = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(heartbeat, decoded);
}

#[test]
fn test_encode_decode_mcu_to_host() {
    let mut buf = [0u8; 256];

    let sync_response = McuToHost::SyncResponse;
    let encoded_len = encode_frame(&sync_response, &mut buf).unwrap();
    let decoded: McuToHost = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(sync_response, decoded);

    let mut sensors: Vec<SensorData, 32> = Vec::new();
    sensors.push(SensorData { id: 1, raw_value: 1023, status: 0 }).unwrap();
    sensors.push(SensorData { id: 2, raw_value: 512, status: 1 }).unwrap();

    let telemetry_batch = McuToHost::TelemetryBatch {
        timestamp_us: 987654321,
        sensors,
    };
    let encoded_len = encode_frame(&telemetry_batch, &mut buf).unwrap();
    let decoded: McuToHost = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(telemetry_batch, decoded);

    let ack = McuToHost::Ack;
    let encoded_len = encode_frame(&ack, &mut buf).unwrap();
    let decoded: McuToHost = decode_frame(&buf[1..encoded_len]).unwrap();
    assert_eq!(ack, decoded);
}

#[test]
fn test_decode_corrupted_frame() {
    let mut buf = [0u8; 256];
    let sync_request = HostToMcu::SyncRequest;
    let encoded_len = encode_frame(&sync_request, &mut buf).unwrap();

    // Corrupt the buffer
    buf[3] = !buf[3];

    let decoded: Result<HostToMcu, _> = decode_frame(&buf[1..encoded_len]);
    assert!(decoded.is_err());
}
