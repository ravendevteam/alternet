use super::*;

pub struct Unset;

impl std::fmt::Display for Unset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", 0)
    }
}


pub struct T {

}


trait Num {
    fn to_n(self: Box<Self>) -> u8;
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
#[derive(Eq)]
pub struct Octet<const T: u8>;

impl<const T: u8> Num for Octet<T> {
    const N: u8 = T;
}

impl<const A: u8, B> From<B> for Octet<A>
where
    B: Into<u8> {
    fn from(value: B) -> Self {
        let ret: u8 = value.into();
        match ret {
            0 => A
        }
        Self(ret)
    }
}

pub struct Ipv4<
    const A: u8,
    const B: u8,
    const C: u8,
    const D: u8
>(
    Octet<A>,
    Octet<B>,
    Octet<C>,
    Octet<D>
);

impl<A, B, C, D> From<(A, B, C, D)> for Ipv4 
where
    A: Into<Octet>,
    B: Into<Octet>,
    C: Into<Octet>,
    D: Into<Octet> {
    fn from(value: (A, B, C, D)) -> Self {
        let a: Octet = value.0.into();
        let b: Octet = value.1.into();
        let c: Octet = value.2.into();
        let d: Octet = value.3.into();
        Self(a, b, c, d)
    }
}


pub trait Common {}
impl Common for Ipv6 {}
impl Common for Ipv6<Hextet> {}
impl Common for Ipv6<Hextet, Hextet> {}
// ... ...

#[derive(derive_more::Debug)]
#[derive(derive_more::Display)]
#[derive(derive_more::Deref)]
#[derive(derive_more::DerefMut)]
#[derive(derive_more::From)]
#[from(u16)]
pub struct Hextet(u16);

pub struct Ipv6<
    A = Unset,
    B = Unset,
    C = Unset,
    D = Unset,
    E = Unset,
    F = Unset,
    G = Unset,
    H = Unset
>(A, B, C, D, E, F, G, H);

impl<A, B, C, D, E, F, G, H> std::fmt::Display for Ipv6<A, B, C, D, E, F, G, H> 
where
    A: std::fmt::LowerHex,
    B: std::fmt::LowerHex,
    C: std::fmt::LowerHex,
    D: std::fmt::LowerHex,
    E: std::fmt::LowerHex,
    F: std::fmt::LowerHex,
    G: std::fmt::LowerHex,
    H: std::fmt::LowerHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h
        ) = self;
        write!(f,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h
        )
    }
}