#[macro_use]
extern crate serde_derive;

mod remote {
    pub struct S(pub u16);
}

#[derive(Deserialize)] //~ ERROR: mismatched types
#[serde(remote = "remote::S")]
struct S(u8); //~^^ expected u16, found u8

fn main() {}
