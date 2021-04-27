# erro-rs

This is a crate to allow specifying the `Error`s which a function may return without having to implement an `enum` representing those errors yourself. It functions using the proc-macro `#[errors(…)]`.

## Example

The following example shows how returning multiple `Error`s can be shortened using this macro:

```rust
use std::{fmt, fs, path::Path};
use erro_rs::errors;

fn main() {
    match read_int("/tmp/foo") {
        Ok(i) => println!("Was an `Ok`: {}", i),
        Err(ReadIntError::StdIo(e)) => println!("`io::Error`: {}", e),
        Err(ReadIntError::StdNumParseInt(e)) => println!("`ParseIntError`: {}", e),
    };
}

#[errors(std::io::Error, std::num::ParseIntError)]
fn read_int(path: impl AsRef<Path>) -> i128 {
    let content = fs::read_to_string(path)?;
    let number = content.parse::<i128>()?;
    Ok(number)
}
```

The above is equivalent to:

```rust
use std::{error::Error, fmt, fs, io, num::ParseIntError, path::Path};

fn main() {
    match read_int("/tmp/foo") {
        Ok(i) => println!("Was an `Ok`: {}", i),
        Err(ReadIntError::StdIo(e)) => println!("`io::Error`: {}", e),
        Err(ReadIntError::StdNumParseInt(e)) => println!("`ParseIntError`: {}", e),
    };
}

fn read_int(path: impl AsRef<Path>) -> Result<i128, ReadIntError> {
    let content = fs::read_to_string(path)?;
    let number = content.parse::<i128>()?;
    Ok(number)
}

#[derive(Debug)]
enum ReadIntError {
    StdIo(io::Error),
    StdNumParseInt(ParseIntError),
}

impl fmt::Display for ReadIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StdIo(e) => write!(f, "{}", e),
            Self::StdNumParseInt(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for ReadIntError {
    fn from(e: io::Error) -> Self {
        Self::StdIo(e)
    }
}

impl From<ParseIntError> for ReadIntError {
    fn from(e: ParseIntError) -> Self {
        Self::StdNumParseInt(e)
    }
}

impl Error for ReadIntError {}
```
