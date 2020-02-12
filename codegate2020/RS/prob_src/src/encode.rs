// Reed-Solomon (48, 16)
// This code can correct errors up to 16bit, you can recover the whole words
// only with the parity data.

use crate::poly::Poly;
use crate::gf256::GF256;

const BSIZE: usize = 16;
const NSYM: usize= BSIZE * 2;

fn generate_poly() -> Poly<GF256> {
    (0 .. NSYM).fold(Poly::new(vec![GF256::new(1)]),
               |acc, i| {
                    acc * Poly::new(vec![GF256::new(1),
                                         GF256::pow(GF256::new(2), i)])}
        )
}

// https://en.wikipedia.org/wiki/Reed%E2%80%93Solomon_error_correction#Systematic_encoding_procedure
fn encode_blk<'a>(g: &Poly<GF256>, chunk: &'a [u8]) -> Vec<u8> {
    // c = msg % G
    let mut out: Vec<GF256> = chunk.iter().map(|x| GF256::new(*x)).collect();
    out.resize(BSIZE + NSYM, GF256::new(0));
    // https://en.wikipedia.org/wiki/Synthetic_division#Expanded_synthetic_division
    for i in 0 .. BSIZE {
        if !out[i].is_zero() {
            for j in 0 .. NSYM {
                out[i + j + 1] = out[i + j + 1] + g[j + 1] * out[i]
            }
        }
    }
    out.iter()
       .skip(BSIZE)
       .map(|x| x.value()).collect()
}

pub fn encode(inp: &String) -> Vec<u8> {
    let g = generate_poly();
    let mut bvec: Vec<u8> = inp.bytes()
                               .map(|x| x as u8)
                               .collect();
    let bcnt = (bvec.len() + BSIZE - 1) / BSIZE;
    bvec.resize(bcnt * BSIZE, 0);

    bvec.chunks(BSIZE)
        .flat_map(|v| encode_blk(&g, v))
        .collect()
}
