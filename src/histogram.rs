pub fn histogram(bytes: &[u8]) -> [usize; 256] {
    if bytes.len() < 100 {
        histogram_simple(bytes)
    } else {
        histogram_parallel(bytes)
    }
}

fn histogram_simple(bytes: &[u8]) -> [usize; 256] {
    let mut histogram = [0; 256];
    for &v in bytes {
        histogram[v as usize] += 1;
    }
    histogram
}

fn histogram_parallel(bytes: &[u8]) -> [usize; 256] {
    // Summing multiple 32 bit histograms is faster than a 64 bit histogram.
    let mut total = [0; 256];
    for bytes in bytes.chunks(u32::MAX as usize) {
        for (i, &v) in histogram_parallel_u32(bytes).iter().enumerate() {
            total[i] += v as usize;
        }
    }
    total
}

// Based on https://github.com/facebook/zstd/blob/1518570c62b95136b6a69714012957cae5487a9a/lib/compress/hist.c#L66
fn histogram_parallel_u32(bytes: &[u8]) -> [u32; 256] {
    let mut histograms = [[0; 256]; 4];

    let (chunks, remainder) = bytes.split_at(bytes.len() / 16 * 16);
    let chunks16: &[[[u8; 4]; 4]] = bytemuck::cast_slice(chunks);
    for chunk16 in chunks16 {
        for chunk4 in chunk16 {
            let c = u32::from_ne_bytes(*chunk4);
            histograms[0][c as u8 as usize] += 1;
            histograms[1][(c >> 8) as u8 as usize] += 1;
            histograms[2][(c >> 16) as u8 as usize] += 1;
            histograms[3][(c >> 24) as usize] += 1;
        }
    }
    for &v in remainder {
        histograms[0][v as usize] += 1;
    }

    let (dst, src) = histograms.split_at_mut(1);
    let dst = &mut dst[0];
    for i in 0..256 {
        for src in src.iter() {
            dst[i] += src[i];
        }
    }
    *dst
}
