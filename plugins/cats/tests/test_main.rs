use cats::tencent_cloud::make_header;
/*
cargo test -- --nocapture
*/
#[test]
fn test1() {
    let ret = make_header("aaabbbcccddd", 12345678, "xyzw", "aaaaa");
    println!("{:#?}", ret);
}
