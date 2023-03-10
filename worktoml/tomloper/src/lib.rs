use toml::Value;
use toml::value::Index;
use std::ops::Div;
use std::ops::BitOr;
use std::ops::Shl;
use std::ops::ShlAssign;

/// Resolve path into a `toml::Value` tree.
/// Return `None` if the path if invalid.
/// Note the input is aslo `Option`, for symmetrical implementation reason.
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

fn path_mut<'tr, B>(v: Option<&'tr mut Value>, p: B) -> Option<&'tr mut Value>
where B: PathBuilder + Index + Copy
{
    if v.is_none() {
        return None;
    }

    let v = v.unwrap();

    // Note: use immutable version of get() to determiner path is valid first,
    // otherwise get_mut() and aplly_mut() would trow E0499 as mut ref twice.
    let target = v.get(p);
    if target.is_some() {
        return v.get_mut(p);
    }
    else {
        let path_segment = p.build_path();
        if path_segment.paths.len() > 1 {
            return path_segment.apply_mut(v);
        }
        else {
            return None;
        }
    }
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
struct PathSegment
{
    paths: Vec<String>,
}

impl PathSegment
{
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

    fn apply_mut<'tr>(&self, v: &'tr mut Value) -> Option<&'tr mut Value> {
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
                target = target.unwrap().get_mut(index);
            }
            else {
                target = target.unwrap().get_mut(p);
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
pub trait PathOperator
{
    fn path<'tr>(&'tr self) -> TomlOpt<'tr>;
    fn pathto<'tr>(&'tr self, p: &str) -> TomlOpt<'tr>;

    fn path_mut<'tr>(&'tr mut self) -> TomlOptMut<'tr>;
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> TomlOptMut<'tr>;
}

impl PathOperator for Value
{
    fn path<'tr>(&'tr self) -> TomlOpt<'tr> {
        TomlOpt::path(self)
    }
    fn pathto<'tr>(&'tr self, p: &str) -> TomlOpt<'tr> {
        let valop = p.build_path().apply(self);
        TomlOpt { valop }
    }

    fn path_mut<'tr>(&'tr mut self) -> TomlOptMut<'tr> {
        TomlOptMut::path(self)
    }
    fn pathto_mut<'tr>(&'tr mut self, p: &str) -> TomlOptMut<'tr> {
        let valop = p.build_path().apply_mut(self);
        TomlOptMut { valop }
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
        TomlOpt { valop: path(self.valop, rhs) }
    }
}

/// pipe operator, get primitive scalar value for leaf node in toml tree.
/// return rhs as default if the node is mistype.
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
        match self.valop {
            Some(v) => v.as_str().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<i64> for TomlOpt<'tr>
{
    type Output = i64;
    fn bitor(self, rhs: i64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_integer().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<f64> for TomlOpt<'tr>
{
    type Output = f64;
    fn bitor(self, rhs: f64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_float().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<bool> for TomlOpt<'tr>
{
    type Output = bool;
    fn bitor(self, rhs: bool) -> Self::Output {
        match self.valop {
            Some(v) => v.as_bool().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// Mutable version of wrapper of `toml::Value` for operator overload.
/// Must reference an existed toml tree, `Option::None` to refer non-exist node.
pub struct TomlOptMut<'tr> {
    valop: Option<&'tr mut Value>,
}

impl<'tr> TomlOptMut<'tr> {
    /// As constructor, to build path operand object from a `toml::Value` node.
    pub fn path(v: &'tr mut Value) -> Self {
        Self { valop: Some(v) }
    }

    /// As unwrapper, to get the underling `Option<&mut toml::Value>`.
    pub fn unpath(&self) -> &Option<&'tr mut Value> {
        &self.valop
    }

    /// Assign any supported value to toml.
    /// But canno overload operator=, will choose <<= instead.
    pub fn assign<T>(&mut self, rhs: T) where Value: From<T> {
        if let Some(ref mut v) = self.valop {
            **v = Value::from(rhs);
        }
    }
}

/// mutable path operator, hope to change the node it point to.
impl<'tr, Rhs> Div<Rhs> for TomlOptMut<'tr>
where Rhs: PathBuilder + Index + Copy
{
    type Output = Self;

    fn div(self, rhs: Rhs) -> Self::Output {
        TomlOptMut { valop: path_mut(self.valop, rhs) }
    }
}

/// pipe operator for TomlOptMut version.
/// todo: refactor the repeat.
impl<'tr> BitOr<String> for TomlOptMut<'tr>
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

impl<'tr> BitOr<&'static str> for TomlOptMut<'tr>
{
    type Output = &'tr str;
    fn bitor(self, rhs: &'static str) -> Self::Output {
        match self.valop {
            Some(v) => v.as_str().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<i64> for TomlOptMut<'tr>
{
    type Output = i64;
    fn bitor(self, rhs: i64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_integer().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<f64> for TomlOptMut<'tr>
{
    type Output = f64;
    fn bitor(self, rhs: f64) -> Self::Output {
        match self.valop {
            Some(v) => v.as_float().unwrap_or(rhs),
            None => rhs,
        }
    }
}

impl<'tr> BitOr<bool> for TomlOptMut<'tr>
{
    type Output = bool;
    fn bitor(self, rhs: bool) -> Self::Output {
        match self.valop {
            Some(v) => v.as_bool().unwrap_or(rhs),
            None => rhs,
        }
    }
}

/// operator << to put a value into toml leaf node, which data type must match.
impl<'tr> Shl<&str> for &mut TomlOptMut<'tr>
{
    type Output = Self;
    fn shl(self, rhs: &str) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            if v.is_str() {
                **v = Value::String(rhs.to_string());
            }
        }
        self
    }
}

impl<'tr> Shl<String> for &mut TomlOptMut<'tr>
{
    type Output = Self;
    fn shl(self, rhs: String) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            if v.is_str() {
                **v = Value::String(rhs);
            }
        }
        self
    }
}

impl<'tr> Shl<i64> for &mut TomlOptMut<'tr>
{
    type Output = Self;
    fn shl(self, rhs: i64) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            if v.is_integer() {
                **v = Value::Integer(rhs);
            }
        }
        self
    }
}

impl<'tr> Shl<f64> for &mut TomlOptMut<'tr>
{
    type Output = Self;
    fn shl(self, rhs: f64) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            if v.is_float() {
                **v = Value::Float(rhs);
            }
        }
        self
    }
}

impl<'tr> Shl<bool> for &mut TomlOptMut<'tr>
{
    type Output = Self;
    fn shl(self, rhs: bool) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            if v.is_bool() {
                **v = Value::Boolean(rhs);
            }
        }
        self
    }
}

/// operator: toml/array/node << [v1, v2, v3, ...]
impl<'tr, T: Copy> Shl<&[T]> for &mut TomlOptMut<'tr> where Value: From<T>
{
    type Output = Self;
    fn shl(self, rhs: &[T]) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            for item in rhs {
                match v {
                    Value::Array(arr) => { arr.push(Value::from(*item)); },
                    _ => {}
                }
            }
        }
        self
    }
}

/// operator << to push one value tuple into toml array, eg:
/// `toml/array/node << (v,)`
/// use single tuple to distinguish push scalar to leaf node.
impl<'tr, T> Shl<(T,)> for &mut TomlOptMut<'tr> where Value: From<T>
{
    type Output = Self;
    fn shl(self, rhs: (T,)) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            match v {
                Value::Array(arr) => { arr.push(Value::from(rhs.0)); },
                _ => {}
            }
        }
        self
    }
}

/// operator << to push key-value pair (tuple) into toml array, eg:
/// `toml/table/node << (k, v)`
impl<'tr, K: ToString, T> Shl<(K, T)> for &mut TomlOptMut<'tr> where Value: From<T>
{
    type Output = Self;
    fn shl(self, rhs: (K, T)) -> Self::Output {
        if let Some(ref mut v) = self.valop {
            match v {
                Value::Table(table) => { table.insert(rhs.0.to_string(), Value::from(rhs.1)); },
                _ => {}
            }
        }
        self
    }
}

/// operator <<= re-assign to an node unconditionally, may change it data type.
/// donot use chained <<= because it is right associated.
impl<'tr, T> ShlAssign<T> for TomlOptMut<'tr> where Value: From<T> 
{
    fn shl_assign(&mut self, rhs: T) {
        self.assign(rhs);
    }
}

#[cfg(test)]
mod tests; // { move to tests.rs }
