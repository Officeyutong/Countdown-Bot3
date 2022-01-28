use rand::{prelude::ThreadRng, seq::SliceRandom, Rng};

use crate::bullshit_data::{前面垫话, 名人名言, 后面垫话, 论述};
fn 来点名人名言(rng: &mut ThreadRng) -> String {
    let mut 名言 = 名人名言.choose(rng).unwrap().to_string();
    名言 = 名言.replace("曾经说过", 前面垫话.choose(rng).unwrap());
    名言 = 名言.replace("这不禁令我深思", 后面垫话.choose(rng).unwrap());
    return 名言;
}
fn 来点论述(rng: &mut ThreadRng, 主题: &str) -> String {
    let mut 句子 = 论述.choose(rng).unwrap().to_string();
    句子 = 句子.replace("主题", 主题);
    return 句子;
}
fn 增加段落(段落_: String) -> String {
    let mut 段落 = 段落_;
    // if 段落.is_empty() {
    //     return String::new();
    // }
    if 段落.ends_with(" ") {
        段落.pop();
    }
    return format!("　　{}。 ", 段落);
}
pub fn generate_bullshit(主题: &str) -> String {
    let mut rng = rand::thread_rng();
    // const 同余乘数: u64 = 214013;
    // const 同余加数: u64 = 2531011;
    // const 同余模: u64 = 1 << 32;
    // let mut 随机种子 = rng.gen_range(10000000..100000000) as u64;
    // let mut 同余发生器 = || -> f64 {
    //     随机种子 = (随机种子 * 同余乘数 + 同余加数) % 同余模;
    //     return 随机种子 as f64 / 同余模 as f64;
    // };
    // let mut 随便取一句 = |列表: &[&str]| {
    //     let 坐标 = 下取整(&同余发生器() * 列表.len() as f64) as usize;
    //     return String::from(列表[坐标]);
    // };
    // let 随便取一个数 = || {
    //     let 最小值: u64 = 0;
    //     let 最大值: u64 = 100;
    //     let 随机函数 = 同余发生器;
    //     let 数字 = (随机函数() * (最大值 - 最小值) as f64) as u64 + 最大值;
    //     return 数字;
    // };
    // let 来点名人名言 = || {

    // };

    let mut 文章: Vec<String> = vec![];
    let mut 段落 = String::new();
    let mut 文章长度: u64 = 0;
    while 文章长度 < 340 * 3 {
        let 随机数 = rng.gen_range(0..=100);
        if 随机数 < 5 && 段落.len() > 200 * 3 {
            段落 = 增加段落(段落);
            文章.push(段落);
            段落 = String::new();
        } else if 随机数 < 20 {
            let 句子 = 来点名人名言(&mut rng);
            文章长度 += 句子.len() as u64;
            段落.push_str(句子.as_str());
        } else {
            let 句子 = 来点论述(&mut rng, 主题);
            文章长度 += 句子.len() as u64;
            段落.push_str(句子.as_str());
        }
    }
    段落 = 增加段落(段落);
    文章.push(段落);
    return 文章.join("");
}
