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