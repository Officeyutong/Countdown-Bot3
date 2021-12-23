use countdown_bot3::countdown_bot::event::notice::*;
use serde_json::{from_value, json};
#[test]
fn serde_test1() {
    from_value::<GroupMemberHonorChangeSubType>(json!("talkative")).unwrap();
    from_value::<GroupMemberHonorChangeSubType>(json!("performer")).unwrap();
    from_value::<GroupMemberHonorChangeSubType>(json!("emotion")).unwrap();
}

#[test]
fn serde_test2() {
    from_value::<GroupMuteSubType>(json!("ban")).unwrap();
    from_value::<GroupMuteSubType>(json!("lift_ban")).unwrap();
}
