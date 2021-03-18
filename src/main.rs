use std::fs::File;
use std::path::Path;
use wav;
use std::convert::TryInto;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "morse_rs", about = "Your Morse code command line buddy.")]
struct Opt {
    #[structopt(short = "w", long = "wpm", default_value = "20")]
    wpm: u32,

    #[structopt(short = "o", long = "output", default_value = "output.wav")]
    output_file: String,
}

#[derive(Debug, PartialEq, Eq)]
enum Sound {
    On,
    Off,
}

// This is one period of a 600 Hz wave sampled at 44,100
const WAV: [u8; 75] = [
    128, 138, 149, 160, 170, 181, 190, 200, 208, 217, 224, 231, 237, 242, 247,
    250, 253, 255, 255, 255, 254, 252, 249, 246, 241, 236, 229, 222, 215, 206,
    197, 188, 178, 168, 157, 147, 136, 125, 114, 103,  92,  82,  72,  62,  53,
     45,  37,  29,  23,  17,  12,   7,   4,   2,   0,   0,   0,   1,   3,   6,
     10,  15,  21,  27,  35,  42,  51,  60,  70,  79,  90, 100, 111, 122, 128,
];

const TABLE: [(char, &str); 40]= [
    ('A', ".-"    ), ('B', "-..."  ), ('C', "-.-."  ), ('D', "-.."  ),
    ('E', "."     ), ('F', "..-."  ), ('G', "--."   ), ('H', "...." ),
    ('I', ".."    ), ('J', ".---"  ), ('K', "-.-"   ), ('L', ".-.." ),
    ('M', "--"    ), ('N', "-."    ), ('O', "---"   ), ('P', ".--." ),
    ('Q', "--.-"  ), ('R', ".-."   ), ('S', "..."   ), ('T', "-"    ),
    ('U', "..-"   ), ('V', "...-"  ), ('W', ".--"   ), ('X', "-..-" ),
    ('Y', "-.--"  ), ('Z', "--.."  ), ('0', "-----" ), ('1', ".----"),
    ('2', "..---" ), ('3', "...--" ), ('4', "....-" ), ('5', "....."),
    ('6', "-...." ), ('7', "--..." ), ('8', "---.." ), ('9', "----."),
    ('?', "..--.."), (',', "--..--"), ('.', ".-.-.-"), ('/', "_..-."),
];

fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();

    let mut out_file = File::create(Path::new(&opt.output_file))?;
    let sample_rate = 44_100;
    let bit_depth = 8;
    let wpm = opt.wpm;

    let elt_per_word = 50; // "PARIS" - standard word for WPM calculation
    let secs_per_min = 60;
    let samples_per_element: u32 = (
        sample_rate as f64 /
        ((wpm as f64 * elt_per_word as f64) / secs_per_min as f64)
    ) as u32;

    if wpm == 20 {
        // in this case the answer is exact
        assert_eq!(2646, samples_per_element);
    }

    let header = wav::Header::new(1, 1, sample_rate, bit_depth);
    let mut raw_data: Vec<u8> = Vec::new();

    for (event, duration) in schedule_word(samples_per_element, "KD9KJV").iter() {
        match event {
            Sound::Off => {
                raw_data.append(&mut vec![0; (*duration as u32).try_into().unwrap()]);
            },
            Sound::On => {
                for i in 0..*duration {
		    if i > (*duration - 220) {
			// damp out the sound near the end, 220 ~ 5ms
			let scale_factor : f64 = 1.0 - (i as f64 - 220.0)/(*duration as f64 - 220.0);
			let sample = WAV[i as usize % WAV.len()];
			let attenuated = scale_factor * (sample as f64 - 128.0) + 128.0;
			raw_data.push(attenuated as u8);
		    }
                    raw_data.push(WAV[i as usize % WAV.len()]);
                }
            }
        }
    }

    wav::write(header, &wav::BitDepth::Eight(raw_data), &mut out_file)?;
    Ok(())
}

// Character spacing:
// "R" -> dit - dah - dit |
//         1  1  3  1  1  3
fn schedule_character(unit: u32, s: &str) -> Vec<(Sound, u32)> {
    let mut out = Vec::new();
    for c in s.chars() {
        if c == '.' { out.push((Sound::On, unit)); }
        if c == '-' { out.push((Sound::On, 3 * unit)); }
        out.push((Sound::Off, unit));
    }
    if let Some(last) = out.last_mut() {
        *last = (Sound::Off, 3 * unit);
    }
    out
}

fn schedule_word(unit: u32, s: &str) -> Vec<(Sound, u32)> {
    let mut out = Vec::new();
    for c in s.chars() {
        let letter = lookup(c.to_ascii_uppercase());
        out.append(&mut schedule_character(unit, letter));
    }
    if let Some(last) = out.last_mut() {
        *last = (Sound::Off, 7 * unit);
    }
    out
}

fn lookup(c: char) -> &'static str {
    for (key, val) in &TABLE {
        if *key == c {
            return val;
        }
    }
    panic!("Looked up an unknown character, '{}'", c);
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_schedule_character() {
        let actual = schedule_character(2, ".-.");
        let expected: Vec<(Sound, u32)> = vec!(
            (Sound::On, 2), (Sound::Off, 2),
            (Sound::On, 6), (Sound::Off, 2),
            (Sound::On, 2), (Sound::Off, 6),
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_schedule_word() {
        let actual = schedule_word(2, "YEET");
        let expected: Vec<(Sound, u32)> = vec!(
            (Sound::On, 6), (Sound::Off, 2),
            (Sound::On, 2), (Sound::Off, 2),
            (Sound::On, 6), (Sound::Off, 2),
            (Sound::On, 6), (Sound::Off, 6),
            (Sound::On, 2), (Sound::Off, 6),
            (Sound::On, 2), (Sound::Off, 6),
            (Sound::On, 6), (Sound::Off, 14),
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_callsign() {
        let actual = schedule_word(2, "KD9KJV");
        let expected: Vec<(Sound, u32)> = vec!(
            (Sound::On, 6), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 6), // K
            (Sound::On, 6), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 6), // D
            (Sound::On, 6), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 6), // 9
            (Sound::On, 6), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 6), // K
            (Sound::On, 2), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 6), // J
            (Sound::On, 2), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 2), (Sound::On, 2), (Sound::Off, 2), (Sound::On, 6), (Sound::Off, 14), // V
        );
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_lookup() {
        let actual = lookup('A');
        assert_eq!(actual, ".-");
    }
}
