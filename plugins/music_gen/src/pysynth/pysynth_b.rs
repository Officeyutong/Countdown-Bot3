/*
##########################################################################
#                       * * *  PySynth  * * *
#       A very basic audio synthesizer in Python (www.python.org)
#
#          Martin C. Doege, 2009-06-08 (mdoege@compuserve.com)
##########################################################################
# Based on a program by Tyler Eaves (tyler at tylereaves.com) found at
#   http://mail.python.org/pipermail/python-list/2000-August/041308.html
##########################################################################

# 'song' is a Python list (or tuple) in which the song is defined,
#   the format is [['note', value]]

# Notes are 'a' through 'g' of course,
# optionally with '#' or 'b' appended for sharps or flats.
# Finally the octave number (defaults to octave 4 if not given).
# An asterisk at the end makes the note a little louder (useful for the beat).
# 'r' is a rest.

# Note value is a number:
# 1=Whole Note; 2=Half Note; 4=Quarter Note, etc.
# Dotted notes can be written in two ways:
# 1.33 = -2 = dotted half
# 2.66 = -4 = dotted quarter
# 5.33 = -8 = dotted eighth
*/

use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use ndarray::prelude::*;
pub(crate) static HARMO: [[f64; 6]; 8] = [
    [1., -15.8, -3., -15.3, -22.8, -40.7],
    [16., -15.8, -3., -15.3, -22.8, -40.7],
    [28., -5.7, -4.4, -17.7, -16., -38.7],
    [40., -6.8, -17.2, -22.4, -16.8, -75.6],
    [52., -8.4, -19.7, -23.5, -21.6, -76.8],
    [64., -9.3, -20.8, -37.2, -36.3, -76.4],
    [76., -18., -64.5, -74.4, -77.3, -80.8],
    [88., -24.8, -53.8, -77.2, -80.8, -90.],
];
use anyhow::anyhow;

use crate::pysynth::mkfreq::getfreq;
fn linint(arr: &[(f64, f64)], x: f64) -> anyhow::Result<f64> {
    if arr.is_empty() {
        return Err(anyhow!("Empty array!"));
    }
    for (x0, y) in arr {
        if *x0 == x {
            return Ok(*y);
        }
    }
    let mut ux = arr[0].0;
    let mut lx = arr[0].0;
    let mut uy = 0 as f64;
    let mut ly = 0 as f64;
    for (x, _) in arr.iter() {
        ux = ux.max(*x);
        lx = lx.min(*x);
    }
    for (x0, y0) in arr.iter() {
        if *x0 > x && x0 - x <= ux - x {
            ux = *x0;
            uy = *y0;
        }
        if *x0 < x && x - x0 >= lx - x {
            lx = *x0;
            ly = *y0;
        }
    }
    return Ok((x - lx) / (ux - lx) * (uy - ly) + ly);
}
pub(crate) const ATT_LEN: usize = 3000;
lazy_static! {
    pub(crate) static ref PITCHHZ: BTreeMap<String, f64> = getfreq().0;
    pub(crate) static ref KEYNUM: BTreeMap<String, usize> = getfreq().1;
    pub(crate) static ref HARMTAB: Array2<f64> = {
        let mut arr = Array2::<f64>::zeros((88, 20));

        for h in 1..6 {
            let mut data: Vec<(f64, f64)> = vec![];
            for n in 0..8 {
                data.push((HARMO[n][0], HARMO[n][h]));
            }
            for h2 in 0..88 {
                arr[[h2, h]] = linint(&data[..], (h2 + 1) as f64).unwrap();
            }
        }
        for h2 in 0..88 {
            for n in 0..20 {
                let val = arr[[h2, 1]];
                arr[[h2, n]] = (10.0f64).powf((arr[[h2, n]] - val) / 20.0);
            }
        }
        arr
    };
    pub(crate) static ref ATT_BASS: Array1::<f64> = {
        let mut att_bass_local = Array1::<f64>::zeros(ATT_LEN);
        for n in 0..ATT_LEN {
            att_bass_local[n] = linint(
                &[
                    (0., 0.),
                    (100., 0.1),
                    (300., 0.2),
                    (400., 0.15),
                    (600., 0.1),
                    (800., 0.9),
                    (1000., 1.25),
                    (2000., 1.15),
                    (3000., 1.),
                ],
                n as f64,
            )
            .unwrap();
        }

        att_bass_local
    };
    pub(crate) static ref ATT_TREB: Array1::<f64> = {
        let mut att_treb_local = Array1::<f64>::zeros(ATT_LEN);
        for n in 0..ATT_LEN {
            att_treb_local[n] = linint(
                &[
                    (0.0, 0.),
                    (100., 0.2),
                    (300., 0.7),
                    (400., 0.6),
                    (600., 0.25),
                    (800., 0.9),
                    (1000., 1.25),
                    (2000., 1.15),
                    (3000., 1.),
                ],
                n as f64,
            )
            .unwrap();
        }
        att_treb_local
    };
    pub(crate) static ref DECAY: Array1<f64> = {
        let mut decay_local = Array1::<f64>::zeros(1000);
        for n in 0..900 {
            decay_local[n] = linint(
                &[
                    (0.0, (3.0f64).ln()),
                    (3.0, (5.0f64).ln()),
                    (5.0, (1.0f64).ln()),
                    (6.0, (8.0f64).ln()),
                    (9.0, (0.1f64).ln()),
                ],
                n as f64 / 100.0,
            )
            .unwrap()
            .exp();
        }
        decay_local
    };
}

pub struct WaveRenderer {
    pub(crate) bpm: i32,
    pub(crate) transpose: i32,
    pub(crate) leg_stac: f64,
    pub(crate) boost: f64,
    pub(crate) repeat: i32,
    pub(crate) cache_this: HashMap<String, f64>,
    pub(crate) note_cache: HashMap<String, Array1<f64>>,
}
impl Default for WaveRenderer {
    fn default() -> Self {
        Self {
            bpm: 120,
            transpose: 0,
            leg_stac: 0.9,
            boost: 1.1,
            repeat: 0,
            cache_this: HashMap::new(),
            note_cache: HashMap::new(),
        }
    }
}
impl WaveRenderer {
    pub fn set_bpm(self, v: i32) -> Self {
        Self { bpm: v, ..self }
    }
    // pub fn set_transpose(self, v: i32) -> Self {
    //     Self {
    //         transpose: v,
    //         ..self
    //     }
    // }
    // pub fn set_leg_stac(self, v: f64) -> Self {
    //     Self {
    //         leg_stac: v,
    //         ..self
    //     }
    // }
    // pub fn set_boost(self, v: f64) -> Self {
    //     Self { boost: v, ..self }
    // }
    // pub fn set_repeat(self, v: i32) -> Self {
    //     Self { repeat: v, ..self }
    // }
}
