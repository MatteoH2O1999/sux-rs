use anyhow::Result;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::io::Seek;
use std::io::Write;
use sux::prelude::*;

#[test]
fn test_serdes() {
    let u = 10_000;
    let n = 1_000;
    let mut rng = SmallRng::seed_from_u64(0);

    let mut values = (0..n).map(|_| rng.gen_range(0..u)).collect::<Vec<_>>();

    values.sort();

    // create the builder for the "in memory" elias-fano
    let mut efb = EliasFanoBuilder::new(u, n);
    // push the values
    for value in values.iter() {
        efb.push(*value).unwrap();
    }
    // Finish the creation of elias-fano
    let ef = efb.build();
    println!("{} {}", ef.mem_size(), ef.mem_used());
    println!("{}", EliasFanoBuilder::mem_upperbound(u, n) / 8);

    let tmp_file = std::env::temp_dir().join("test_serdes_ef.bin");
    {
        let mut file = std::io::BufWriter::new(std::fs::File::create(&tmp_file).unwrap());
        ef.serialize(&mut file).unwrap();
    }

    let mut file = std::fs::File::open(&tmp_file).unwrap();
    let file_len = file.seek(std::io::SeekFrom::End(0)).unwrap();
    let mmap = unsafe {
        mmap_rs::MmapOptions::new(file_len as _)
            .unwrap()
            .with_file(file, 0)
            .map()
            .unwrap()
    };

    let ef = <EliasFano<BitMap<&[u64]>, CompactArray<&[u64]>>>::deserialize(&mmap)
        .unwrap()
        .0;

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(ef.get(idx).unwrap(), *value);
    }

    let ef = map::<_, EliasFano<BitMap<&[u64]>, CompactArray<&[u64]>>>(&tmp_file, &Flags::empty())
        .unwrap();

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(ef.get(idx).unwrap(), *value);
    }

    let ef = map::<_, EliasFano<BitMap<&[u64]>, CompactArray<&[u64]>>>(
        &tmp_file,
        &Flags::TRANSPARENT_HUGE_PAGES,
    )
    .unwrap();

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(ef.get(idx).unwrap(), *value);
    }

    let ef = load::<_, EliasFano<BitMap<&[u64]>, CompactArray<&[u64]>>>(&tmp_file, &Flags::empty())
        .unwrap();

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(ef.get(idx).unwrap(), *value);
    }
}

#[test]

fn test_slices() -> Result<()> {
    let tmp_file = std::env::temp_dir().join("test_serdes_slices.bin");
    let s: Vec<u8> = (0..100).collect();
    {
        let mut file = std::io::BufWriter::new(std::fs::File::create(&tmp_file).unwrap());
        file.write(&s)?;
    }

    assert_eq!(
        s.as_slice(),
        &load_slice::<_, u8>(&tmp_file, &Flags::empty())?.as_ref()[0..100]
    );
    assert_eq!(
        s.as_slice(),
        &map_slice::<_, u8>(&tmp_file, &Flags::empty())?.as_ref()[0..100]
    );

    let t = bytemuck::cast_slice::<u8, u32>(s.as_slice());

    assert_eq!(
        t,
        &load_slice::<_, u32>(&tmp_file, &Flags::empty())?.as_ref()[0..25]
    );
    assert_eq!(
        t,
        &map_slice::<_, u32>(&tmp_file, &Flags::empty())?.as_ref()[0..25]
    );

    Ok(())
}