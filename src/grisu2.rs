use diyfp::{ self, DiyFp };

#[inline]
unsafe fn grisu_round(buffer: &mut i64, delta: u64, mut rest: u64, ten_kappa: u64, wp_w: u64) {
    while rest < wp_w && delta - rest >= ten_kappa &&
           (rest + ten_kappa < wp_w || // closer
            wp_w - rest > rest + ten_kappa - wp_w) {
        *buffer -= 1;
        rest += ten_kappa;
    }
}

#[inline]
fn count_decimal_digit32(n: u32) -> i16 {
    if n < 10 { 1 }
    else if n < 100 { 2 }
    else if n < 1000 { 3 }
    else if n < 10000 { 4 }
    else if n < 100000 { 5 }
    else if n < 1000000 { 6 }
    else if n < 10000000 { 7 }
    else if n < 100000000 { 8 }
    // Will not reach 10 digits in digit_gen()
    else { 9 }
}

unsafe fn digit_gen(w: DiyFp, mp: DiyFp, mut delta: u64, mut k: i16) -> (i64, i16) {
    static POW10: [u32; 10] = [ 1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000 ];
    let one = DiyFp::new(1u64 << -mp.e, mp.e);
    let wp_w = mp - w;
    let mut p1 = (mp.f >> -one.e) as u32;
    let mut p2 = mp.f & (one.f - 1);
    let mut kappa = count_decimal_digit32(p1); // kappa in [0, 9]

    let mut buffer = p1 as i64;

    while kappa > 0 {
        match kappa {
            9 => { p1 %=  100000000; }
            8 => { p1 %=   10000000; }
            7 => { p1 %=    1000000; }
            6 => { p1 %=     100000; }
            5 => { p1 %=      10000; }
            4 => { p1 %=       1000; }
            3 => { p1 %=        100; }
            2 => { p1 %=         10; }
            1 => { p1 =           0; }
            _ => {}
        }
        kappa = kappa.wrapping_sub(1);
        let tmp = ((p1 as u64) << -one.e) + p2;
        if tmp <= delta {
            k += kappa;

            grisu_round(&mut buffer, delta, tmp, (POW10[kappa as usize] as u64) << -one.e, wp_w.f);
            return (buffer, k);
        }
    }

    loop {
        p2 = (p2 << 3) + (p2 << 1);
        delta = (delta << 3) + (delta << 1);
        let d = (p2 >> -one.e) as u8;
        if d != 0 || buffer != 0 {
            buffer = (buffer << 3) + (buffer << 1) + d as i64;
        }
        p2 &= one.f - 1;
        kappa = kappa.wrapping_sub(1);
        if p2 < delta {
            k += kappa;
            let index = -(kappa as isize);

            grisu_round(&mut buffer, delta, p2, one.f, wp_w.f * if index < 9 { POW10[-(kappa as isize) as usize] as u64 } else { 0 });
            return (buffer, k);
        }
    }
}

#[inline]
pub fn convert(float: f64) -> (i64, i16) {
    unsafe {
        let v = DiyFp::from_f64(float);
        let (w_m, w_p) = v.normalized_boundaries();
        let (c_mk, k) = diyfp::get_cached_power(w_p.e);
        let w = v.normalize() * c_mk;
        let mut wp = w_p * c_mk;
        let mut wm = w_m * c_mk;
        wm.f += 1;
        wp.f -= 1;

        digit_gen(w, wp, wp.f - wm.f, k as i16)
    }
}
