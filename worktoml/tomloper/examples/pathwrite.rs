use tomloper::PathOperator;

fn main()
{
    let str_toml = include_str!("./sample.toml");
    let mut v: toml::Value = str_toml.parse().unwrap();

    println!("original toml content:");
    println!("{str_toml}");

    println!("modify by path:");

    let mut node = v.path_mut() / "ip";
    let _ = &mut node << "127.0.0.2";

    // push key-val pair to table
    let mut node = v.path_mut() / "host";
    let _ = &mut node << ("newkey1", 1) << ("newkey2", "2");

    // push scalar to leaf node, replace it
    node = node / "port";
    let _ = &mut node << 8888;

    // push single tuple to array
    node = v.path_mut() / "host" /"protocol";
    let _ = &mut node << (8989,) << ("xyz",);

    // <<= can change node type while << cannot
    node = v.path_mut() / "misc" / "bool";
    node <<= "false";

    println!("{}", v);
}
