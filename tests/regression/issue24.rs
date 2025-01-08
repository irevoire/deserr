use deserr::{DeserializeError, Deserr, IntoValue, MergeWithError};
use std::ops::ControlFlow;

#[derive(Debug)]
pub struct MyError;
impl DeserializeError for MyError {
    fn error<V: IntoValue>(
        _self_: Option<Self>,
        _error: deserr::ErrorKind<V>,
        _location: deserr::ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        todo!()
    }
}

impl MergeWithError<MyError> for MyError {
    fn merge(
        _self_: Option<Self>,
        _other: MyError,
        _merge_location: deserr::ValuePointerRef,
    ) -> ControlFlow<Self, Self> {
        todo!()
    }
}

#[derive(Deserr)]
#[deserr(error = MyError)]
#[allow(unused)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[test]
fn main() {
    let result = serde_json::from_str::<serde_json::Value>(r#"{"x": 1, "y": 2, "z": 3}"#).unwrap();
    let _: Point = deserr::deserialize(result).unwrap();
}
