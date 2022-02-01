use music_gen::notes::{parse_major, parse_note, transform_single_note};

#[test]
fn normal_things() {
    assert_eq!(parse_major("#c").unwrap(), 1);
    assert_eq!(parse_major("#g").unwrap(), 8);
    assert_eq!(parse_major("bg").unwrap(), 6);
    assert_eq!(parse_major("bG").unwrap(), 6);
    assert_eq!(parse_major("E").unwrap(), 4);
    assert_eq!(parse_major("#E").unwrap(), 5);
}
#[test]
#[should_panic]
fn test_panic() {
    parse_major("1").unwrap();
}

#[test]
fn parse_note_normal() {
    assert_eq!(parse_note("1").unwrap(), 48);
    assert_eq!(parse_note("2").unwrap(), 50);
    assert_eq!(parse_note("#2").unwrap(), 51);
    assert_eq!(parse_note("b3").unwrap(), 51);
    assert_eq!(parse_note("#12").unwrap(), 25);
    assert_eq!(parse_note("b23").unwrap(), 37);
    assert_eq!(parse_note("#1").unwrap(), 49);
}

#[test]
#[should_panic]
fn parse_note_test_panic1() {
    parse_note("8").unwrap();
}
#[test]
#[should_panic]
fn parse_note_test_panic2() {
    parse_note("#").unwrap();
}

#[test]
fn transform_single_note_normal() {
    assert_eq!(transform_single_note("1.5", 7).unwrap(), "g4.5");
    assert_eq!(transform_single_note("6.5", 4).unwrap(), "c#5.5");
    assert_eq!(transform_single_note("#7*.5", 3).unwrap(), "d#5*.5");
    assert_eq!(transform_single_note("b1*.5", 8).unwrap(), "g4*.5");
}
