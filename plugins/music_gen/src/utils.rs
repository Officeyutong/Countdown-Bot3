pub fn command_hash(
    notes: &Vec<&str>,
    use_number: bool,
    bpm: u32,
    scale: f64,
    major: &str,
    volume: &Option<Vec<u32>>,
    inverse_beats: &Option<i64>,
) -> String {
    use sha2::Digest;
    let mut inst = sha2::Sha256::new();
    inst.update(
        format!(
            "{},{},{},{},{},{}",
            use_number,
            bpm,
            scale,
            major,
            match volume {
                Some(v) => v
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                None => "none".to_string(),
            },
            inverse_beats.unwrap_or(-1 as i64)
        )
        .as_bytes(),
    );
    for s in notes.iter() {
        inst.update(s.as_bytes());
    }
    return hex::encode(inst.finalize());
}
