use super::*;

fn load_test_toml() -> Value
{
    let str_toml = r#"
ip = "127.0.0.1"
[host]
ip = "127.0.1.1"
port = 8080
protocol = ["tcp", "udp", "mmp"]
[[service]]
name = "serv_1"
desc = "first server"
[[service]]
name = "serv_2"
desc = "another server"
[misc]
int = 1234
float = 3.14
bool = true
"#;
    let v: Value = str_toml.parse().unwrap();
    return v;
}

#[test]
fn path_test() {
    let v = load_test_toml();
    assert_eq!(path(Some(&v), "ip").unwrap().as_str(), Some("127.0.0.1"));
    assert_eq!(v["ip"].as_str(), Some("127.0.0.1"));

    let op = TomlOpt::path(&v);
    let ip = op / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.0.1"));

    let ip = op / "host" / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.1.1"));

    let host = TomlOpt::path(&v) / "host";
    let ip = host / "ip";
    assert_eq!(ip.valop.unwrap().as_str(), Some("127.0.1.1"));
    let port = host / "port";
    assert_eq!(port.valop.unwrap().as_integer(), Some(8080));

    let proto = host / "protocol" / 1;
    assert_eq!(proto.valop.unwrap().as_str(), Some("udp"));

    let proto = v.path() / "host" / "protocol" / 2;
    assert_eq!(proto.unpath().unwrap().as_str(), Some("mmp"));

    let proto = v.path() / "host/protocol/2";
    assert_eq!(proto.unpath().unwrap().as_str(), Some("mmp"));

    let proto = v.pathto("/host/protocol/2").unpath();
    assert_eq!(proto.unwrap().as_str(), Some("mmp"));

    let server = v.path() / "service" / 0 / "name";
    assert_eq!(server.unpath().unwrap().as_str(), Some("serv_1"));

    // path() produce immutable reference, cannot write or assign
    // let server = server.unpath().unwrap();
    // *server = Value::String(String::from("serv 1"));

    let mut mv = load_test_toml();
    let ip = mv.get_mut("ip").unwrap();
    *ip = Value::String(String::from("127.0.0.2"));
    assert_eq!((mv.path() / "ip").unpath().unwrap().as_str(), Some("127.0.0.2"));
}

#[test]
fn path_none_test() {
    let v = load_test_toml();

    let root = v.path();
    assert_eq!(root.unpath().is_none(), false);

    let node = root / "ip";
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "IP";
    assert_eq!(node.unpath().is_none(), true);

    let node = root / "host" /"protocol";
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "host" /"protocol" / 1;
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "host" /"protocol" / 3;
    assert_eq!(node.unpath().is_none(), true);

    let node = root / "service" / 0;
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "service" / 0 / "description";
    assert_eq!(node.unpath().is_none(), true);
    let node = root / "service" / 0 / "desc";
    assert_eq!(node.unpath().is_none(), false);
    let node = root / "service" / 2;
    assert_eq!(node.unpath().is_none(), true);
}

#[test]
fn path_build_test() {
    let pseg = "".build_path();
    dbg!(&pseg.paths);
    assert_eq!(pseg.paths.is_empty(), false);
    assert_eq!(pseg.paths, vec![""]);

    let pseg = "/".build_path();
    dbg!(&pseg.paths);
    assert_eq!(pseg.paths.is_empty(), false);
    assert_eq!(pseg.paths, vec!["", ""]);

    let pseg = "//".build_path();
    assert_eq!(pseg.paths, vec!["", "", ""]);

    let pseg = "/path/to/leaf".build_path();
    assert_eq!(pseg.paths, vec!["", "path", "to", "leaf"]);

    let pseg = "path/to/leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to//leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "", "leaf"]);

    let pseg = "path.to.leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to.leaf".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf"]);

    let pseg = "path/to.leaf/".build_path();
    assert_eq!(pseg.paths, vec!["path", "to", "leaf", ""]);
}

#[test]
fn pipe_test() {
    let v = load_test_toml();

    // pipe ending slash operator to get inner scalar primitive value
    let ip = v.path() / "ip" | "";
    assert_eq!(ip, "127.0.0.1");

    let ip = v.path() / "host" / "ip" | "";
    assert_eq!(ip, "127.0.1.1");

    let ip_default = "127";
    let ip = v.path() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");

    // pipe convertor accept &'static str or String,
    // but ofcourse &str is more efficient
    let ip_default: String = String::from("127");
    let ip = v.path() / "host" / "ip" | ip_default;
    assert_eq!(ip, "127.0.1.1");
    // ip_default is moved by | operator, and cannot use non-static &String
    // println!("{ip_default}");

    let port = v.path() / "host" / "port" | 0;
    assert_eq!(port, 8080);

    let port_default = 80;
    let port = v.path() / "host" / "port" | port_default;
    assert_eq!(port, 8080);
    assert_eq!(port_default, 80); // simple primitive wont moved

    // can save intermedia tmp value
    let misc = v.path() / "misc";
    let value = misc / "int" | 0;
    assert_eq!(value, 1234);
    let value = misc / "float" | 0.0;
    assert_eq!(value, 3.14);
    let value = misc / "bool" | false;
    assert_eq!(value, true);

    // path ignore repeated slash or dot
    let value = v.pathto("/misc/int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc/int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc/int/") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("misc.int") | 0;
    assert_eq!(value, 1234);
    let value = v.pathto("/misc/./int/") | 0;
    assert_eq!(value, 1234);
}

