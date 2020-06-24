use std::{io, fs};

// Inspired by a Github Gist.
// Honestly, it's kind of silly that Rust doesn't provide an abstraction
// over this mess already like nearly every other language does.
// Cite: https://gist.github.com/ayosec/2ee0993247e003b42c5c
pub enum Input {
    Standard(std::io::Stdin),
    File(std::fs::File)
}

#[allow(dead_code)]
impl Input {
    pub fn stdin() -> Input {
        Input::Standard(io::stdin())
    }
    pub fn file(path: String) -> io::Result<Input> {
        Ok(Input::File(fs::File::open(path)?))
    }
    pub fn from_arg(arg: Option<String>) -> io::Result<Input> {
        Ok(match arg {
            None       => Input::stdin(),
            Some(path) => Input::file(path)?
        })
    }
}

impl io::Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::Standard(ref mut s) => s.read(buf),
            Input::File(ref mut f)     => f.read(buf),
        }
    }
}
