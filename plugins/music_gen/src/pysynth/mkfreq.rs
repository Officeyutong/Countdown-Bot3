use std::collections::BTreeMap;
const KEYS_S: [&str; 12] = [
    "a", "a#", "b", "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#",
];
const KEYS_F: [&str; 12] = [
    "a", "bb", "b", "c", "db", "d", "eb", "e", "f", "gb", "g", "ab",
];
const KEYS_E: [&str; 12] = [
    "a", "bb", "cb", "b#", "db", "d", "eb", "fb", "e#", "gb", "g", "ab",
];

pub fn getfreq() -> (BTreeMap<String, f64>, BTreeMap<String, usize>) {
    let mut pitchhz = BTreeMap::<String, f64>::new();
    let mut keynum = BTreeMap::<String, usize>::new();

    for k in 0..88 {
        let freq = 27.5 * (2.0f64).powf(k as f64 / 12.);
        let oct = (k + 9) / 12;
        {
            let note = format!("{}{}", KEYS_S[k % 12], oct);
            pitchhz.insert(note.clone(), freq);
            keynum.insert(note, k);
        }
        {
            let note = format!("{}{}", KEYS_F[k % 12], oct);
            pitchhz.insert(note.clone(), freq);
            keynum.insert(note, k);
        }
        {
            let note = format!("{}{}", KEYS_E[k % 12], oct);
            pitchhz.insert(note.clone(), freq);
            keynum.insert(note, k);
        }
    }
    return (pitchhz, keynum);
}
