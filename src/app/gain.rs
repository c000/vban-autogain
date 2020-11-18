use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

fn i16_to_f32(mut s: &[u8]) -> Vec<f32> {
    let mut v = Vec::with_capacity(s.len() / 2);
    while let Ok(i) = s.read_i16::<LittleEndian>() {
        let a = i as f32;
        v.push(a);
    }
    v
}

pub fn auto_gain_i16(mut s: &mut [u8], gain: f32, gain_acc: &mut f32) {
    let sample_num = s.len() / 4;
    let gain_mul = gain.powf(sample_num as f32);
    *gain_acc *= gain_mul;

    let samples = i16_to_f32(s);
    let max_sample = samples
        .iter()
        .map(|v| v.abs())
        .fold(0.0 / 0.0, |x, y| y.max(x));
    let max_gain = i16::max_value() as f32 / max_sample;
    *gain_acc = gain_acc.min(max_gain);

    samples
        .iter()
        .map(|v| v * *gain_acc)
        .for_each(|v| s.write_i16::<LittleEndian>(v.floor() as i16).unwrap());
}
