use ds_drawer_plugin::sam::*;
#[test]
fn test1() {
    let mut pool = SAMPool::default();
    // pool.join_string("aabba", 1);
    pool.join_string("abbbba", 1);
    pool.join_string("aaabb", 2);
    
    pool.collect();
    println!("{}",String::from_utf8(pool.generate_graph()).unwrap());
    // for b in pool.nodes.iter() {
    //     println!("{}", b);
    // }
}
