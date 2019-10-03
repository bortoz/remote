use cpython::{PyModule, Python};
use error::RemoteError;

const LIKER_PY: &'static str = include_str!("./liker.py");

pub fn like(user: String, token: String) -> Result<(String, String), RemoteError> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let caller = PyModule::new(py, "liker")?;
    caller.add(py, "__builtins__", py.import("builtins")?)?;
    let locals = caller.get(py, "__dict__")?.extract(py)?;
    py.run(LIKER_PY, Some(&locals), None)?;
    let ret: (String, String) = caller.call(py, "run", (user, token), None)?.extract(py)?;
    Ok(ret)
}
