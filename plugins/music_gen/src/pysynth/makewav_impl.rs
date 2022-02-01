use std::ops::{AddAssign, MulAssign};

use crate::pysynth::pysynth_b::{ATT_BASS, ATT_LEN, ATT_TREB, DECAY, HARMTAB};

use super::pysynth_b::{WaveRenderer, KEYNUM, PITCHHZ};
use anyhow::anyhow;
// use ndarray::parallel::prelude::*;
use ndarray::prelude::*;
use ndarray::Zip;
// use wav::{BitDepth, Header};
fn waves2(hz: f64, l: f64) -> (f64, f64) {
    (44100.0 / hz, (l / 44100.0 * hz).round())
}
impl WaveRenderer {
    pub fn make_wav(mut self, song: &[(String, f64)]) -> anyhow::Result<Vec<i16>> {
        let bpmfac = 120.0 / self.bpm as f64;
        let length = |v: f64| 88200.0 / v * bpmfac;
        let mut ex_pos = 0.00;
        let mut t_len = 0.0;
        for (y, x) in song.iter() {
            if y.is_empty() {
                return Err(anyhow!("Empty note encoutered."));
            }
            if *x < 0.0 {
                t_len += length(-2.0 * x / 3.0);
            } else {
                t_len += length(*x);
            }
            let next_y = if y.ends_with("*") {
                y[..y.len() - 1].to_string()
            } else if y.chars().last().unwrap().is_digit(10) {
                format!("{}4", y)
            } else {
                y.clone()
            };
            self.cache_this
                .insert(next_y, self.cache_this.get(y).unwrap_or(&0.0) + 1.0);
        }
        let mut data =
            Array1::<f64>::zeros(((self.repeat + 1) as f64 * t_len + 441000.0).floor() as usize);
        for _ in 0..self.repeat + 1 {
            for x in song.iter() {
                // debug!("Note {}", idx);
                let b = if x.1 < 0.0 {
                    length(-2.0 * x.1 / 3.0)
                } else {
                    length(x.1)
                };
                if x.0.as_str() != "r" {
                    let (vol, mut note) = if x.0.ends_with("*") {
                        (self.boost, x.0[..x.0.len() - 1].to_string())
                    } else {
                        (1.0, x.0.clone())
                    };
                    if !note.chars().last().unwrap().is_digit(10) {
                        note.push_str("4");
                    }
                    let mut a = *PITCHHZ.get(&note).ok_or(anyhow!("非法音符: {}", note))?;
                    let kn = *KEYNUM.get(&note).ok_or(anyhow!("非法音符: {}", note))?;
                    a *= (2.0f64).powi(self.transpose);
                    self.render2(a, b, vol, ex_pos as usize, kn, &note, &mut data);
                    ex_pos += b;
                } else {
                    ex_pos += b;
                }
            }
        }
        let maxval = *data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        data /= maxval * 2.0;
        let out_len = (2.0 * 44100.0 + ex_pos + 0.5).floor() as usize;
        let inter_data = data.slice(s![..out_len]).to_owned() * 32000.0;
        let music_data = inter_data
            .iter()
            .map(|v| v.round() as i16)
            .collect::<Vec<i16>>();
        return Ok(music_data);
    }
    fn render2(
        &mut self,
        a: f64,
        b: f64,
        vol: f64,
        pos: usize,
        knum: usize,
        note: &str,
        data: &mut Array1<f64>,
    ) {
        use std::f64::consts::PI;
        let l = waves2(a, b);
        let q = (l.0 * l.1) as i64;
        let lf = a.ln();
        let t = (lf - 3.) / (8.5 - 3.);
        let volfac = 1. + 0.8 * t * (PI / 5.3 * (lf - 3.)).cos();
        let schweb = waves2(lf * 100., b).0;
        let schweb_amp = 0.05 - (lf - 5.) / 100.;
        let att_fac = (knum as f64 / 87. * vol).min(1.0);
        let raw_note = 12 * 44100;
        let snd_len = (((3.1 * q as f64) as i64).max(44100)).min(raw_note);
        let mut fac = Array1::<f64>::ones(snd_len as usize);
        Zip::from(&mut fac.slice_mut(s![..ATT_LEN]))
            .and(&*ATT_TREB)
            .and(&*ATT_BASS)
            .par_for_each(|c, &a, &b| {
                *c = a * att_fac + b * (1.0 - att_fac);
            });
        let mut new = if !self.note_cache.contains_key(note) {
            let x2 = Array1::<f64>::range(0.0, raw_note as f64, 1.0);
            let sina = x2.mapv(|a| a * 2.0 * PI / l.0);
            let ov = x2.mapv(|a| (-a / 3.0 / DECAY[(lf * 100.0) as usize] / 44100.0).exp());
            let mut new = Array1::<f64>::zeros(sina.len());
            Zip::from(&mut new)
                .and(&ov)
                .and(&sina)
                .par_for_each(|c, &ov, &sina| {
                    *c = volfac
                        * (sina.sin()
                            + ov * HARMTAB[[knum, 2]] * (2.0 * sina).sin()
                            + ov * HARMTAB[[knum, 3]] * (3.0 * sina).sin()
                            + ov * HARMTAB[[knum, 4]] * (4.0 * sina).sin()
                            + ov * HARMTAB[[knum, 5]] * (8.0 * sina).sin());
                });
            // (sina.mapv(f64::sin)
            //     + ov.mapv(|a| a * HARMTAB[[knum, 2]]) * sina.mapv(|a| (2.0 * a).sin())
            //     + ov.mapv(|a| a * HARMTAB[[knum, 3]]) * sina.mapv(|a| (3.0 * a).sin())
            //     + ov.mapv(|a| a * HARMTAB[[knum, 4]]) * sina.mapv(|a| (4.0 * a).sin())
            //     + ov.mapv(|a| a * HARMTAB[[knum, 5]]) * sina.mapv(|a| (8.0 * a).sin()))
            // .mapv(|a| a * volfac);
            Zip::from(&mut new).and(&x2).par_for_each(|c, &a| {
                *c *= (-a / DECAY[(lf * 100.0) as usize] / 44100.0).exp();
            });
            // new *= &x2.mapv(|a| (-a / DECAY[(lf * 100.0) as usize] / 44100.0).exp());
            if *self.cache_this.get(note).unwrap_or(&0.0) > 1.0 {
                self.note_cache.insert(note.to_string(), new.clone());
            }
            new
        } else {
            self.note_cache.get(note).unwrap().clone()
        };
        let dec_ind = (self.leg_stac * q as f64) as usize;

        new.slice_mut(s![dec_ind..]).mul_assign(
            &Array1::<f64>::range(0.0, raw_note as f64 - dec_ind as f64, 1.0)
                .mapv(|a| (-a / 3000.0).exp()),
        );
        let snd_len = snd_len.min(raw_note) as usize;
        let right = (&new.slice(s![..snd_len]) * (&fac) * vol)
            * Array1::<f64>::range(0.0, snd_len as f64, 1.0)
                .mapv(|a| (2.0 * PI * a / schweb / 32.0).sin() * schweb_amp + 1.0);
        data.slice_mut(s![pos..pos + snd_len]).add_assign(&right);
    }
}
