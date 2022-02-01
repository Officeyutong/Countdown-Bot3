use anyhow::anyhow;
use std::collections::HashMap;
lazy_static::lazy_static! {
    static ref BASE_MAPPING:HashMap<char,i32> = HashMap::from([
        ('C',0),
        ('D',2),
        ('E',4),
        ('F',5),
        ('G',7),
        ('A',9),
        ('B',11),
    ]);
}

fn parse_tone(major_note: &str) -> (i32, String) {
    return if major_note.starts_with("b") {
        (-1, major_note.chars().skip(1).collect())
    } else if major_note.starts_with("#") {
        (1, major_note.chars().skip(1).collect())
    } else {
        (0, major_note.to_string())
    };
}

/*
解析基准音,返回绝对音高
[b或#或忽略][音符]
*/
pub fn parse_major(major_note: &str) -> anyhow::Result<i32> {
    let (result, parsed_note) = parse_tone(major_note);
    let chr = parsed_note
        .chars()
        .nth(0)
        .ok_or(anyhow!("非法音符: \"{}\"", major_note))?;
    return Ok(BASE_MAPPING
        .get(&(chr.to_ascii_uppercase()))
        .ok_or(anyhow!("非法音符: \"{}\"", major_note))?
        + result);
}

/*
解析简谱,返回绝对音高
[b或#或忽略][音符][八度(默认为4)]
例如#12 #23
*/

pub fn parse_note(note: &str) -> anyhow::Result<i32> {
    const NOTE_LIST: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
    // let mut result = 0;
    let (mut result, parsed_note) = parse_tone(note);
    let note_char_vec = parsed_note.chars().collect::<Vec<char>>();
    if note_char_vec.is_empty() {
        return Err(anyhow!("非法音符: \"{}\"", note));
    }
    let note_chr = note_char_vec[0];
    let mut octave = 4;
    if note_char_vec.len() == 2 {
        octave = i32::from_str_radix(&note_char_vec[1].to_string(), 10)
            .map_err(|_| anyhow!("非法八度: {}", note_char_vec[1]))?;
    }
    result += NOTE_LIST
        .get(note_chr as usize - '1' as usize)
        .ok_or(anyhow!("非法音高: {}", note_chr))?;
    result += 12 * octave;
    return Ok(result);
}
// 转换 简谱+时间到五线谱+时间
pub fn transform_single_note(note: &str, major_height: i32) -> anyhow::Result<String> {
    const NOTE_LIST: [&str; 12] = [
        "c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b",
    ];
    if !note.contains(".") {
        return Ok(note.to_string());
    }
    let splitted = note
        .split(".")
        .map(|r| r.to_string())
        .collect::<Vec<String>>();
    let (mut note, duration) = (splitted[0].clone(), splitted[1].clone());
    let starred = note.ends_with("*");
    if starred {
        note.pop();
    }
    let height = major_height + parse_note(&note)?;
    return Ok(format!(
        "{}{}{}.{}",
        NOTE_LIST[(height % 12) as usize],
        height / 12,
        if starred { "*" } else { "" },
        duration
    ));
}

pub fn transform_notes(notes: &[&str], major: &str) -> anyhow::Result<Vec<String>> {
    let major_height = parse_major(major)?;
    let mut result: Vec<String> = vec![];
    for note in notes.iter() {
        let output = if !note.contains("r") {
            transform_single_note(note.trim(), major_height)?
        } else {
            note.to_string()
        };
        result.push(output);
    }
    return Ok(result);
}
