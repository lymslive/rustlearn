use tomloper::PathOperator;

// No need to use the intermedia struct
// use tomloper::TomlOpt;

fn main()
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
    let v: toml::Value = str_toml.parse().unwrap();

    println!("original toml content:");
    println!("{str_toml}");

    println!("read by path:");

    let root = v.path();
    let ip = root / "ip" | "";
    println!("/ip = {ip}");

    let host = root / "host";
    let ip = host / "ip" | "";
    println!("/host/ip = {ip}");
    let port = host / "port" | 0;
    println!("/host/port = {port}");

    let name = v.path() / "service" / 0 / "name" | "";
    println!("/service/0/name = {name}");
    let desc = v.path() / "service" / 0 / "desc" | "";
    println!("/service/0/desc = {desc}");

    let name = v.pathto("service/1/name") | "";
    println!("/service/1/name = {name}");
    let desc = v.pathto("service.1.desc") | "";
    println!("/service/1/desc = {desc}");

    let int = root / "misc" / "int" | 0;
    let float = root / "misc" / "float" | 0.0;
    let tf = root / "misc" / "bool" | false;
    println!("/misc/int = {int}");
    println!("/misc/float = {float}");
    println!("/misc/bool = {tf}");
}