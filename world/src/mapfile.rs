use std::io;
use calx_grid::Prefab;
use terrain;

pub fn save_prefab<W: io::Write>(out: &mut W, map: &Prefab<(terrain::Id, Vec<String>)>) -> io::Result<()> {
    unimplemented!();
}

pub fn load_prefab<I: io::Read>(i: &mut I) -> io::Result<Prefab<(terrain::Id, Vec<String>)>> {
    unimplemented!();
}
