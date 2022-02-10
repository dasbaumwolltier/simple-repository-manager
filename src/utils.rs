use std::{error, io};

#[macro_export]
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub fn to_io_error<R, E>(result: Result<R, E>) -> Result<R, io::Error>
    where E: Into<Box<dyn error::Error + Send + Sync>> {
    result.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[test]
fn test_to_io_error() {
    let io_ok = to_io_error::<&str, &str>(Ok("Ok"));
    let io_error = to_io_error::<&str, &str>(Err("Err"));

    assert_eq!(io_ok.unwrap(), "Ok");
    assert_eq!(format!("{}", io_error.unwrap_err().into_inner().unwrap().as_ref()), "Err")
}
