use toml::Value;
use toml::value::Index;
use std::ops::Div;
use std::ops::BitOr;

/// Resolve path into a `toml::Value` tree.
/// Return `None` if the path if invalid.
/// Note the input is aslo `Option`, for implement detail reason.
fn path<'a, B>(v: Option<&'a Value>, p: B) -> Option<&'a Value>
where B: PathBuilder + Index + Copy
{
    if v.is_none() {
        return None;
    }

    let v = v.unwrap();
    let from_index = v.get(p);
    if from_index.is_some() {
        return from_index;
    }

    let path_segment = p.build_path();
    if path_segment.paths.len() > 1 {
        return path_segment.apply(v);
    }

    return None;
}

/// Determin whether a string is all numeric digital.
/// The numeric path is treated as index of toml array.
fn is_string_numeric(s: &str) -> bool {
    for c in s.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    return true;
}

/// Path segment break on slash(/) or dot(.).
/// eg: `table.subtable.key` or `table/subtable/key` or `array/index/key`
struct PathSegment {
    paths: Vec<String>,
}

impl PathSegment {
    fn apply<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        let mut target = Some(v);
        for p in &self.paths {
            if target.is_none() {
                return None;
            }
            if p.is_empty() {
                continue;
            }
            else if is_string_numeric(&p) {
                let index: usize = p.parse().unwrap();
                target = target.unwrap().get(index);
            }
            else {
                target = target.unwrap().get(p);
            }
        }
        return target;
    }
}

trait PathBuilder {
    fn build_path(&self) -> PathSegment {
        PathSegment { paths: Vec::new() }
    }
}

/// split string to get path segment vector.
impl PathBuilder for &str {
    fn build_path(&self) -> PathSegment {
        let paths = self
            .split(|c| c == '/' || c == '.')
            .map(|s| s.to_string())
            .collect();
        PathSegment { paths }
    }
}

/// usize index only act path on it's own, but cannot split to more path segment.
impl PathBuilder for usize {}

/// Adopter for `toml::Value` to use operator overload. 
trait PathOperator {
    type Output<'a> where Self: 'a;
    fn path<'a>(&'a self) -> Self::Output<'a>;
    fn pathto<'a>(&'a self, p: &str) -> Self::Output<'a>;
}

impl PathOperator for Value {
    type Output<'a> = TomlOpt<'a>;
    fn path<'a>(&'a self) -> Self::Output<'a> {
        TomlOpt::path(self)
    }
    fn pathto<'a>(&'a self, p: &str) -> Self::Output<'a> {
        let valop = p.build_path().apply(self);
        TomlOpt { valop }
    }
}

/// Wrapper of `toml::Value` for operator overload.
/// Must reference an existed toml tree, `Option::None` to refer non-exist node.
#[derive(Copy, Clone)]
pub struct TomlOpt<'a> {
    valop: Option<&'a Value>,
}

impl<'a> TomlOpt<'a> {
    /// As constructor, to build path operand object from a `toml::Value` node.
    pub fn path(v: &'a Value) -> Self {
        Self { valop: Some(v) }
    }
    
    /// As unwrapper, to get the underling `Option<&toml::Value>`.
    pub fn unpath(&self) -> Option<&'a Value> {
        self.valop
    }
}

/// path operator, visit toml tree by string key for table or index for array.
impl<'a, Rhs> Div<Rhs> for TomlOpt<'a>
where Rhs: PathBuilder + Index + Copy
{
    type Output = Self;

    fn div(self, rhs: Rhs) -> Self::Output {
        let valop = path(self.valop, rhs);
        return TomlOpt { valop };
    }
}

/// pipe operator, get primitive scalar value for leaf node in toml tree.
/// return rhs if the node is mistype.
/// support | &str, String, i64, f64, bool,
/// not support datetime type of toml.
/// Note: pipe operator(|) is the vertical form of path operator(/),
/// and usually stand on the end of path chain.
/// eg. `let scalar = toml.path() / "path" / "to" / "leaf" | "default-value"; `
impl<'a> BitOr<String> for TomlOpt<'a>
{
    type Output = String;

    fn bitor(self, rhs: String) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_str() {
            Some(s) => s.to_string(),
            None => rhs
        }
    }
}

impl<'a> BitOr<&'static str> for TomlOpt<'a>
{
    type Output = &'a str;

    fn bitor(self, rhs: &'static str) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_str() {
            Some(v) => v,
            None => rhs
        }
    }
}

impl<'a> BitOr<i64> for TomlOpt<'a>
{
    type Output = i64;

    fn bitor(self, rhs: i64) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_integer() {
            Some(v) => v,
            None => rhs
        }
    }
}

impl<'a> BitOr<f64> for TomlOpt<'a>
{
    type Output = f64;

    fn bitor(self, rhs: f64) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_float() {
            Some(v) => v,
            None => rhs
        }
    }
}

impl<'a> BitOr<bool> for TomlOpt<'a>
{
    type Output = bool;

    fn bitor(self, rhs: bool) -> Self::Output {
        if self.valop.is_none() {
            return rhs;
        }
        match self.valop.unwrap().as_bool() {
            Some(v) => v,
            None => rhs
        }
    }
}

#[cfg(test)]
mod tests {
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

        let port = v.path() / "host" / "port" | 0;
        assert_eq!(port, 8080);

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
}
