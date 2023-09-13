/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use anyhow::Result;
use epserde::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use sux::prelude::compact_array::CompactArray;
use sux::prelude::*;

#[test]
fn test_epserde() -> Result<()> {
    let mut rng = SmallRng::seed_from_u64(0);

    let mut v = CompactArray::<Vec<usize>>::new(4, 200);
    for i in 0..200 {
        v.set(i, rng.gen_range(0..(1 << 4)));
    }

    let tmp_file = std::env::temp_dir().join("test_serdes_ef.bin");
    let mut file = std::io::BufWriter::new(std::fs::File::create(&tmp_file)?);
    v.serialize(&mut file)?;
    drop(file);

    let w = <CompactArray<Vec<usize>>>::mmap(&tmp_file, epserde::des::Flags::empty()).unwrap();

    for i in 0..200 {
        assert_eq!(v.get(i), w.get(i));
    }

    Ok(())
}