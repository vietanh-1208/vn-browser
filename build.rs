use std::env;
use std::fs;
use std::path::PathBuf;

fn render_dib(size: u32) -> Vec<u8> {
    let (w, h) = (size, size);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let mut blend = |x: i32, y: i32, c: [u8; 4], alpha: f32| {
        if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h && alpha > 0.0 {
            let a = (alpha.clamp(0.0, 1.0) * (c[3] as f32 / 255.0)).clamp(0.0, 1.0);
            let i = ((y as u32 * w + x as u32) * 4) as usize;
            let dst_a = rgba[i + 3] as f32 / 255.0;
            let out_a = a + dst_a * (1.0 - a);
            if out_a > 0.0 {
                for ch in 0..3 {
                    let src_c = c[ch] as f32 / 255.0;
                    let dst_c = rgba[i + ch] as f32 / 255.0;
                    let out_c = (src_c * a + dst_c * dst_a * (1.0 - a)) / out_a;
                    rgba[i + ch] = (out_c * 255.0).round() as u8;
                }
                rgba[i + 3] = (out_a * 255.0).round() as u8;
            }
        }
    };

    let s = size as f32 / 64.0;
    let cx = size as f32 * 0.5;
    let cy = size as f32 * 0.5;
    let outer_r = 29.0 * s;
    let inner_r = 18.0 * s;

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let dx = fx - cx;
            let dy = fy - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= outer_r + 0.8 {
                let cov = (outer_r + 0.5 - dist).clamp(0.0, 1.0);
                let ang = dy.atan2(dx);
                let t = (fx + fy) / (2.0 * w as f32);
                // Browser tricolor-inspired emerald -> cyan -> royal blue ring
                let bg = if ang < -0.5 {
                    [16, 185, 129, 255] // emerald
                } else if ang < 1.6 {
                    [14, 165, 233, 255] // sky cyan
                } else {
                    [37, 99, 235, 255]  // deep blue
                };
                let rim = [
                    (bg[0] as f32 * (0.9 + 0.1 * (1.0 - t))).min(255.0) as u8,
                    (bg[1] as f32 * (0.9 + 0.1 * (1.0 - t))).min(255.0) as u8,
                    (bg[2] as f32 * (0.9 + 0.1 * (1.0 - t))).min(255.0) as u8,
                    255,
                ];
                blend(x, y, rim, cov);

                // Crisp white inner globe ring
                if dist <= inner_r + 0.8 {
                    let icov = (inner_r + 0.5 - dist).clamp(0.0, 1.0);
                    blend(x, y, [255, 255, 255, 255], icov);
                    let core_r = 13.5 * s;
                    if dist <= core_r + 0.8 {
                        let ccov = (core_r + 0.5 - dist).clamp(0.0, 1.0);
                        let g = (140.0 + 75.0 * (1.0 - fy / h as f32)) as u8;
                        blend(x, y, [14, g, 225, 255], ccov);
                    }
                }
            }
        }
    }

    let mut stroke = |x0: f32, y0: f32, x1: f32, y1: f32, t: f32, c: [u8; 4]| {
        let min_x = (x0.min(x1) - t).floor() as i32;
        let max_x = (x0.max(x1) + t).ceil() as i32;
        let min_y = (y0.min(y1) - t).floor() as i32;
        let max_y = (y0.max(y1) + t).ceil() as i32;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len2 = (dx * dx + dy * dy).max(0.001);
        let rad = t * 0.5;
        for py in min_y..=max_y {
            for px in min_x..=max_x {
                let fx = px as f32 + 0.5;
                let fy = py as f32 + 0.5;
                let proj = (((fx - x0) * dx + (fy - y0) * dy) / len2).clamp(0.0, 1.0);
                let nx = x0 + proj * dx;
                let ny = y0 + proj * dy;
                let dist = ((fx - nx).powi(2) + (fy - ny).powi(2)).sqrt();
                if dist <= rad + 0.8 {
                    blend(px, py, c, (rad + 0.5 - dist).clamp(0.0, 1.0));
                }
            }
        }
    };

    // Globe equator & meridian lines inside center sphere
    let white = [255, 255, 255, 235];
    stroke(cx - 12.5 * s, cy, cx + 12.5 * s, cy, 2.2 * s, white);
    stroke(cx, cy - 12.5 * s, cx, cy + 12.5 * s, 2.2 * s, white);

    let mut dib = Vec::with_capacity(40 + (w * h * 4) as usize + ((w.div_ceil(32) * 4) * h) as usize);
    dib.extend_from_slice(&40u32.to_le_bytes());
    dib.extend_from_slice(&(w as i32).to_le_bytes());
    dib.extend_from_slice(&((h * 2) as i32).to_le_bytes());
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&(w * h * 4).to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());

    for y in (0..h).rev() {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            dib.push(rgba[i + 2]);
            dib.push(rgba[i + 1]);
            dib.push(rgba[i]);
            dib.push(rgba[i + 3]);
        }
    }
    let mask_stride = (w.div_ceil(32) * 4) as usize;
    dib.resize(dib.len() + mask_stride * h as usize, 0);
    dib
}

fn build_grp_icon(sizes: &[u32], dibs: &[Vec<u8>]) -> Vec<u8> {
    let mut g = Vec::new();
    g.extend_from_slice(&0u16.to_le_bytes());
    g.extend_from_slice(&1u16.to_le_bytes());
    g.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    for (idx, (&sz, dib)) in sizes.iter().zip(dibs.iter()).enumerate() {
        let b = if sz >= 256 { 0 } else { sz as u8 };
        g.push(b);
        g.push(b);
        g.push(0);
        g.push(0);
        g.extend_from_slice(&1u16.to_le_bytes());
        g.extend_from_slice(&32u16.to_le_bytes());
        g.extend_from_slice(&(dib.len() as u32).to_le_bytes());
        g.extend_from_slice(&((idx + 1) as u16).to_le_bytes());
    }
    g
}

fn build_coff_rsrc_object(dibs: &[Vec<u8>], grp: &[u8]) -> Vec<u8> {
    let n_icons = dibs.len();
    let dir_size = |n: usize| 16 + n * 8;
    let total_dir_bytes = dir_size(2) + dir_size(n_icons) + dir_size(1) + n_icons * dir_size(1) + dir_size(1);
    let total_entry_bytes = (n_icons + 1) * 16;
    let data_base = total_dir_bytes + total_entry_bytes;

    let mut data_offsets = Vec::with_capacity(n_icons + 1);
    let mut cur = data_base;
    for d in dibs {
        data_offsets.push(cur);
        cur += (d.len() + 3) & !3;
    }
    data_offsets.push(cur);
    let grp_len_aligned = (grp.len() + 3) & !3;
    let rsrc_len = cur + grp_len_aligned;

    let mut rsrc = vec![0u8; rsrc_len];
    let write_u16 = |buf: &mut [u8], off: usize, v: u16| buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
    let write_u32 = |buf: &mut [u8], off: usize, v: u32| buf[off..off + 4].copy_from_slice(&v.to_le_bytes());

    let root_off = 0usize;
    let rt_icon_off = dir_size(2);
    let rt_grp_off = rt_icon_off + dir_size(n_icons);
    let lang_base = rt_grp_off + dir_size(1);
    let entry_base = total_dir_bytes;

    write_u16(&mut rsrc, root_off + 14, 2);
    write_u32(&mut rsrc, root_off + 16, 3);
    write_u32(&mut rsrc, root_off + 20, 0x8000_0000 | rt_icon_off as u32);
    write_u32(&mut rsrc, root_off + 24, 14);
    write_u32(&mut rsrc, root_off + 28, 0x8000_0000 | rt_grp_off as u32);

    write_u16(&mut rsrc, rt_icon_off + 14, n_icons as u16);
    for i in 0..n_icons {
        let e = rt_icon_off + 16 + i * 8;
        let l = lang_base + i * dir_size(1);
        write_u32(&mut rsrc, e, (i + 1) as u32);
        write_u32(&mut rsrc, e + 4, 0x8000_0000 | l as u32);
    }

    write_u16(&mut rsrc, rt_grp_off + 14, 1);
    let l_grp = lang_base + n_icons * dir_size(1);
    write_u32(&mut rsrc, rt_grp_off + 16, 1);
    write_u32(&mut rsrc, rt_grp_off + 20, 0x8000_0000 | l_grp as u32);

    let mut reloc_offsets = Vec::with_capacity(n_icons + 1);
    for i in 0..=n_icons {
        let l = lang_base + i * dir_size(1);
        let de = entry_base + i * 16;
        write_u16(&mut rsrc, l + 14, 1);
        write_u32(&mut rsrc, l + 16, 0x0409);
        write_u32(&mut rsrc, l + 20, de as u32);
        write_u32(&mut rsrc, de, data_offsets[i] as u32);
        reloc_offsets.push(de as u32);
        let sz = if i < n_icons { dibs[i].len() } else { grp.len() };
        write_u32(&mut rsrc, de + 4, sz as u32);
    }

    for (i, d) in dibs.iter().enumerate() {
        rsrc[data_offsets[i]..data_offsets[i] + d.len()].copy_from_slice(d);
    }
    rsrc[data_offsets[n_icons]..data_offsets[n_icons] + grp.len()].copy_from_slice(grp);

    let raw_data_ptr = 20u32 + 40u32;
    let reloc_ptr = raw_data_ptr + rsrc.len() as u32;
    let n_relocs = reloc_offsets.len() as u16;
    let sym_ptr = reloc_ptr + (n_relocs as u32) * 10;

    let mut coff = Vec::with_capacity(sym_ptr as usize + 22);
    coff.extend_from_slice(&0x8664u16.to_le_bytes());
    coff.extend_from_slice(&1u16.to_le_bytes());
    coff.extend_from_slice(&0u32.to_le_bytes());
    coff.extend_from_slice(&sym_ptr.to_le_bytes());
    coff.extend_from_slice(&1u32.to_le_bytes());
    coff.extend_from_slice(&0u16.to_le_bytes());
    coff.extend_from_slice(&0u16.to_le_bytes());

    coff.extend_from_slice(b".rsrc\0\0\0");
    coff.extend_from_slice(&0u32.to_le_bytes());
    coff.extend_from_slice(&0u32.to_le_bytes());
    coff.extend_from_slice(&(rsrc.len() as u32).to_le_bytes());
    coff.extend_from_slice(&raw_data_ptr.to_le_bytes());
    coff.extend_from_slice(&reloc_ptr.to_le_bytes());
    coff.extend_from_slice(&0u32.to_le_bytes());
    coff.extend_from_slice(&n_relocs.to_le_bytes());
    coff.extend_from_slice(&0u16.to_le_bytes());
    coff.extend_from_slice(&0x4000_0040u32.to_le_bytes());

    coff.extend_from_slice(&rsrc);

    for off in reloc_offsets {
        coff.extend_from_slice(&off.to_le_bytes());
        coff.extend_from_slice(&0u32.to_le_bytes());
        coff.extend_from_slice(&0x0003u16.to_le_bytes());
    }

    coff.extend_from_slice(b".rsrc\0\0\0");
    coff.extend_from_slice(&0u32.to_le_bytes());
    coff.extend_from_slice(&1i16.to_le_bytes());
    coff.extend_from_slice(&0u16.to_le_bytes());
    coff.push(3);
    coff.push(0);
    coff.extend_from_slice(&4u32.to_le_bytes());
    coff
}

fn build_ico_file(sizes: &[u32], dibs: &[Vec<u8>]) -> Vec<u8> {
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + sizes.len() * 16;
    for (&sz, dib) in sizes.iter().zip(dibs.iter()) {
        let b = if sz >= 256 { 0 } else { sz as u8 };
        ico.push(b);
        ico.push(b);
        ico.push(0);
        ico.push(0);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(dib.len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += dib.len();
    }
    for dib in dibs {
        ico.extend_from_slice(dib);
    }
    ico
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let sizes = [256u32, 64, 48, 32, 16];
    let dibs: Vec<Vec<u8>> = sizes.iter().map(|&s| render_dib(s)).collect();
    let ico = build_ico_file(&sizes, &dibs);
    let _ = fs::write("vn-browser.ico", &ico);
    let grp = build_grp_icon(&sizes, &dibs);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let coff_path = out_dir.join("vn_browser_icon.o");
    fs::write(&coff_path, build_coff_rsrc_object(&dibs, &grp)).expect("write icon");
    println!("cargo:rustc-link-arg-bins={}", coff_path.display());
}
