/*!
  Encode binary data into human-readable random words.
 */
use self::Decode::{V, C};

static VS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
static CS: [char; 14] = ['b', 'd', 'f', 'g', 'j', 'k', 'm', 'n',
    'p', 'r', 's', 't', 'v', 'z'];

#[derive(Copy, Clone)]
enum Decode {
    V(u16),
    C(u16),
}

static LUT: [Option<Decode>; 26] = [
    Some(V(0)),  // a (ASCII 97)
    Some(C(0)),  // b
    None,        // c
    Some(C(1)),  // d
    Some(V(1)),  // e
    Some(C(2)),  // f
    Some(C(3)),  // g
    None,        // h
    Some(V(2)),  // i
    Some(C(4)),  // j
    Some(C(5)),  // k
    None,        // l
    Some(C(6)),  // m
    Some(C(7)),  // n
    Some(V(3)),  // o
    Some(C(8)),  // p
    None,        // q
    Some(C(9)),  // r
    Some(C(10)), // s
    Some(C(11)), // t
    Some(V(4)),  // u
    Some(C(12)), // v
    None,        // w
    None,        // x
    None,        // y
    Some(C(13)), // z
];

fn vorud_chunk(data: u16) -> String {
    let (data, c5) = (data / CS.len() as u16, data % CS.len() as u16);
    let (data, v4) = (data / VS.len() as u16, data % VS.len() as u16);
    let (data, c3) = (data / CS.len() as u16, data % CS.len() as u16);
    let (data, v2) = (data / VS.len() as u16, data % VS.len() as u16);
    let (_, c1) =    (data / CS.len() as u16, data % CS.len() as u16);
    [CS[c1 as usize],
     VS[v2 as usize],
     CS[c3 as usize],
     VS[v4 as usize],
     CS[c5 as usize],
    ].iter().map(|&x| x).collect()
}

fn decode(c: char) -> Option<Decode> {
    let idx = c as isize - 97;
    if idx < 0 || idx >= LUT.len() as isize { return None; }
    LUT[idx as usize]
}

fn v(c: char) -> Result<u16, ()> {
    match decode(c) {
        Some(V(i)) => Ok(i),
        _ => Err(())
    }
}

fn c(c: char) -> Result<u16, ()> {
    match decode(c) {
        Some(C(i)) => Ok(i),
        _ => Err(())
    }
}

fn durov_chunk(s: &str) -> Result<u16, ()> {
    if s.len() != 5 { return Err(()); }
    let s: Vec<char> = s.chars().collect();
    let (mut ret, mut n) = (0u16, 1u16);
    ret += try!(c(s[4])) * n; n *= CS.len() as u16;
    ret += try!(v(s[3])) * n; n *= VS.len() as u16;
    ret += try!(c(s[2])) * n; n *= CS.len() as u16;
    ret += try!(v(s[1])) * n; n *= VS.len() as u16;
    ret += try!(c(s[0])) * n;

    Ok(ret)
}

/// A vorud string.
#[derive(PartialEq, Eq, Debug)]
pub struct Vorud(String);

impl Vorud {
    pub fn new(s: String) -> Result<Vorud, ()> {
        if s.len() == 0 { return Ok(Vorud("".to_string())); }
        let ch = &s[..].chars().collect::<Vec<char>>();
        let mut i = 0;
        loop {
            if i > ch.len() - 5 { return Err(()); }
            try!(c(ch[i]));
            try!(v(ch[i + 1]));
            try!(c(ch[i + 2]));
            try!(v(ch[i + 3]));
            try!(c(ch[i + 4]));
            if i == ch.len() - 5 { return Ok(Vorud(s)); }
            if ch[i + 5] != '-' { return Err(()); }
            i += 6;
        }
    }
}

/// Convert vorud into data.
pub trait FromVorud<E>: Sized {
    fn from_vorud(v: &Vorud) -> Result<Self, E>;
}

/// Convert data into vorud.
pub trait ToVorud {
    fn to_vorud(&self) -> Vorud;
}

impl<'a> ToVorud for &'a [u8] {
    fn to_vorud(&self) -> Vorud {
        let mut ret = String::new();
        for i in 0..(self.len() / 2) {
            if ret.len() > 0 { ret.push_str("-"); }
            let b0 = self[i * 2] as u16;
            let b1 = if self.len() == i * 2 + 1 { 0 } else { self[i * 2 + 1] as u16 };
            ret.push_str(&vorud_chunk(b1 + (b0 << 8))[..]);
        }
        Vorud(ret)
    }
}

impl FromVorud<()> for Vec<u8> {
    fn from_vorud(&Vorud(ref s): &Vorud) -> Result<Vec<u8>, ()> {
        let mut ret = Vec::new();
        for chunk in (&s[..]).split('-') {
            let x = try!(durov_chunk(chunk));
            ret.push((x / 0xff) as u8);
            ret.push((x % 0xff) as u8);
        }
        Ok(ret)
    }
}

impl ToVorud for u32 {
    fn to_vorud(&self) -> Vorud {
        let mut vec = Vec::new();
        vec.push(((*self >> 24) % 0xff) as u8);
        vec.push(((*self >> 16) % 0xff) as u8);
        vec.push(((*self >> 8) % 0xff) as u8);
        vec.push((*self % 0xff) as u8);
        (&vec[..]).to_vorud()
    }
}

impl FromVorud<()> for u32 {
    fn from_vorud(v: &Vorud) -> Result<u32, ()> {
        let v: Vec<u8> = try!(FromVorud::from_vorud(v));
        if v.len() != 4 { return Err(()); }
        Ok(v[3] as u32 +
            ((v[2] as u32) << 8) +
            ((v[1] as u32) << 16) +
            ((v[0] as u32) << 24))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_vorud() {
        use super::ToVorud;

        assert_eq!("babab", &super::vorud_chunk(0)[..]);
        assert_eq!("babad", &super::vorud_chunk(1)[..]);
        assert_eq!(Ok(0u16), super::durov_chunk("babab"));
        assert_eq!(Ok(1234u16), super::durov_chunk(&super::vorud_chunk(1234)[..]));
        assert_eq!("togas", &super::vorud_chunk(super::durov_chunk("togas").unwrap())[..]);
        assert_eq!(super::Vorud("babab-babab".to_string()), 0u32.to_vorud());
        assert_eq!(super::Vorud("babab-babad".to_string()), 1u32.to_vorud());
        assert_eq!(Ok(1u32), super::FromVorud::from_vorud(&super::Vorud("babab-babad".to_string())));
    }
}
