use std::fmt::{Display, Write};

#[derive(Debug)]
pub struct Bytes(pub Vec<u8>);

#[derive(Debug)]
pub struct Frame(pub Vec<u8>);

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0
            .iter()
            .fold(String::new(), |mut acc, &b| {
                write!(acc, "{:02X} ", b).unwrap();
                acc
            })
            .trim_end()
            .to_string();
        write!(f, "{}", s)
    }
}
