use toml::Value;
use toml::value::Index;
use std::ops::Div;
use std::ops::BitOr;

/// Resolve path into a `toml::Value` tree.
/// Return `None` if the path if invalid.
/// Note the input is aslo `Option`, for implement detail reason.
fn path<'tr, B>(v: Option<&'tr Value>, p: B) -> Option<&'tr Value>
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
    fn apply<'tr>(&self, v: &'tr Value) -> Option<&'tr Value> {
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
    fn path<'tr>(&'tr self) -> TomlOpt<'tr>;
    fn pathto<'tr>(&'tr self, p: &str) -> TomlOpt<'tr>;
}

impl PathOperator for Value {
    fn path<'tr>(&'tr self) -> TomlOpt<'tr> {
        TomlOpt::path(self)
    }
    fn pathto<'tr>(&'tr self, p: &str) -> TomlOpt<'tr> {
        let valop = p.build_path().apply(self);
        TomlOpt { valop }
    }
}

/// Wrapper of `toml::Value` for operator overload.
/// Must reference an existed toml tree, `Option::None` to refer non-exist node.
#[derive(Copy, Clone)]
pub struct TomlOpt<'tr> {
    valop: Option<&'tr Value>,
}

impl<'tr> TomlOpt<'tr> {
    /// As constructor, to build path operand object from a `toml::Value` node.
    pub fn path(v: &'tr Value) -> Self {
        Self { valop: Some(v) }
    }
    
    /// As unwrapper, to get the underling `Option<&toml::Value>`.
    pub fn unpath(&self) -> Option<&'tr Value> {
        self.valop
    }
}

/// path operator, visit toml tree by string key for table or index for array.
impl<'tr, Rhs> Div<Rhs> for TomlOpt<'tr>
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
impl<'tr> BitOr<String> for TomlOpt<'tr>
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

impl<'tr> BitOr<&'static str> for TomlOpt<'tr>
{
    type Output = &'tr str;

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

impl<'tr> BitOr<i64> for TomlOpt<'tr>
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

impl<'tr> BitOr<f64> for TomlOpt<'tr>
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

impl<'tr> BitOr<bool> for TomlOpt<'tr>
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
mod tests; // { move to tests.rs }
