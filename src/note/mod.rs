//! # Note (convert Note Struct <-> String)
//! A note has an optional pitch, None = Rest.
//! 
//! ## Structure
//! 
//! **note duration**: Almost always required number for note length.  If it is
//! not provided, then next must be R for a whole measure rest.
//! 
//! - O: 128th note
//! - X: 64th note
//! - Y: 32nd note
//! - S: 16th note
//! - T: 8th note
//! - Q: quarter note
//! - U: half note
//! - W: whole note
//! - V: double whole note (breve)
//! - L: quadruple whole note (longa)
//! - .: augmentation dot
//! 
//! **note name**: Required name of the note.  A-G, or R for rest.
//! 
//! - `A`
//! - `B`
//! - `C`
//! - `D`
//! - `E`
//! - `F`
//! - `G`
//! - `R`
//! 
//! **accidental**: Optional accidental.  If not provided, from key signature.  Cannot be same as what is in the key signature.
//! 
//! - `bb`: Double Flat (Whole-Tone Flat)
//! - `db`: 3/4-Tone Flat
//! - `b`: Flat (1/2-Tone Flat)
//! - `d`: 1/4-Tone Flat
//! - `n`: Natural
//! - `t`: 1/4-Tone Sharp
//! - `#`: Sharp (1/2-Tone Sharp)
//! - `t#`: 3/4-Tone Sharp
//! - `x`: Double Sharp (Whole-Tone Sharp)
//! 
//! **octave**: Required octave.  `-`=-1,`0`,`1`,`2`,`3`,`4`,`5`,`6`,`7`,`8`,`9`
//! 
//! **articulation**: Optional articulation.
//! 
//! - `^`: Marcato (separated sharp attack)
//! - `>`: Accent (sharp attack)
//! - `.`: Staccato (separated)
//! - `'`: Staccatissimo (very separated)
//! - `_`: Tenuto (connected)
//! - `_.`: Tenuto Staccato
//! - `^.`: Marcato Staccato
//! - `^_`: Marcato Tenuto
//! - `>.`: Accent Staccato
//! - `>_`: Accent Tenuto
//! - `+`: closed mute (or palm mute rendered as _ on guitar)
//! - `o`: open mute
//! - `@`: harmonic (smaller o)
//! - `|`: pedal

use std::{fmt, str::FromStr};
use crate::Fraction;

mod articulation;
mod pitch;
mod duration;

pub use self::articulation::*;
pub use self::pitch::*;
pub use self::duration::*;

/// Number of steps above middle C
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Steps(pub i32);

impl std::ops::Add for Steps {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Steps(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Steps {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Steps(self.0 - rhs.0)
    }
}

impl std::ops::Mul<i32> for Steps {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Steps(self.0 * rhs)
    }
}

impl std::ops::Div<i32> for Steps {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Steps(self.0 / rhs)
    }
}

/// A note.
#[derive(Clone)]
pub struct Note {
    /// Pitch & Octave
    pub pitch: Option<(PitchClass, PitchOctave)>,
    /// Duration of the note as a fraction.
    pub duration: Vec<Duration>,
    /// Articulation.
    pub articulation: Vec<Articulation>,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write duration.
        for duration in &self.duration {
            write!(f, "{}", duration)?;
        }

        // Write pitch
        match self.pitch {
            // Write note name & octave.
            Some(ref pitch) => write!(f, "{}{}", pitch.0, pitch.1)?,
            // Write R for rest.
            None => write!(f, "R")?,
        }

        // Write articulation symbols.
        for articulation in &self.articulation {
            write!(f, "{}", articulation)?;
        }

        Ok(())
    }
}

impl FromStr for Note {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Read duration (until pitch).
        let mut end_index = Err(());
        for (i, c) in s.char_indices() {
            match c {
                'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'R' => {
                    end_index = Ok(i);
                    break;
                }
                _ => {}
            }
        }
        let mut end_index = end_index?;
        let duration = s[..end_index].parse::<Duration2>().or(Err(()))?.0;

        // Read pitch
        let begin_index = end_index;
        let pitch = match s.get(begin_index..).ok_or(())? {
            "R" => {
                None
            }
            a => {
                // Get Pitch Class
                let mut end_index2 = Err(());
                for (i, c) in s.char_indices().skip(begin_index) {
                    match c {
                        '-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7'
                            | '8' | '9' =>
                        {
                            end_index2 = Ok(i);
                            break;
                        }
                        _ => {}
                    }
                }
                end_index = end_index2?;

                let pitch_class = s[begin_index..end_index].parse::<PitchClass>()?;

                // Get Pitch Octave
                let pitch_octave = s[end_index..end_index+1].parse::<PitchOctave>()?;

                Some((pitch_class, pitch_octave))
            }
        };
        end_index += 1;

        // Read articulation symbols.
        let mut articulation = vec![];
        let mut articulation_str = "".to_string();
        for articulation_char in s[end_index..].chars() {
            articulation_str.clear();
            articulation_str.push(articulation_char);
            articulation.push(articulation_str.parse::<Articulation>().or(Err(()))?);
        }

        Ok(Note {
            pitch,
            duration,
            articulation,
        })
    }
}

impl Note {
    /// Get the note's visual distance above middle C (C4).
    pub fn visual_distance(&self) -> Option<Steps> {
        if let Some(ref pitch) = self.pitch {
            // Calculate number of octaves from middle C (C4).
            let octaves = pitch.1 as i32 - 4;
            // Calculate number of steps from C within key.
            let steps = pitch.0.name as i32;
            // Calculate total number of steps from middle C.
            Some(Steps { 0: steps + octaves * 7 })
        } else {
            None
        }
    }

    /// Set pitch class and octave.
    pub fn set_pitch(&mut self, pitch: (PitchClass, PitchOctave)) {
        self.pitch = Some(pitch);
    }

    /// Set duration of note.
    pub fn set_duration(&mut self, duration: Vec<Duration>) {
        self.duration = duration;
    }

    /// Set duration of note (indexed).
    pub fn set_duration_indexed(&mut self, duration: Duration, index: usize) {
        self.duration[index] = duration;
    }

    /// Get the fraction of the note.
    pub fn fraction(&self, index: usize) -> Option<Fraction> {
        Some(self.duration.get(index)?.fraction())
    }

    fn move_step(&self, create: (PitchClass, PitchOctave), run: &dyn Fn(&(PitchClass, PitchOctave)) -> Option<(PitchClass, PitchOctave)>) -> Note {
        let pitch = if let Some(ref pitch) = self.pitch {
            (run)(pitch)
        } else {
            Some(create)
        };

        Note {
            pitch,
            duration: self.duration.clone(),
            articulation: self.articulation.clone(),
        }
    }

    /// Calculate note one quarter step up.
    pub fn quarter_step_up(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.step_up(create) // FIXME
    }

    /// Calculate note one quarter step down.
    pub fn quarter_step_down(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.step_down(create) // FIXME
    }

    /// Calculate note one half step up.
    pub fn half_step_up(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.step_up(create) // FIXME
    }

    /// Calculate note one half step down.
    pub fn half_step_down(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.step_down(create) // FIXME
    }

    /// Calculate note one step up within the key.
    /// - `create`: Note that is generated from a rest.
    pub fn step_up(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.move_step(create, &|pitch| {
            let (pitch_class, offset) = match pitch.0.name {
                PitchName::A => (PitchName::B, false),
                PitchName::B => (PitchName::C, true),
                PitchName::C => (PitchName::D, false),
                PitchName::D => (PitchName::E, false),
                PitchName::E => (PitchName::F, false),
                PitchName::F => (PitchName::G, false),
                PitchName::G => (PitchName::A, false),
            };
            let pitch_octave = if offset {
                pitch.1.raise()
            } else {
                Some(pitch.1)
            };

            if let Some(pitch_octave) = pitch_octave {
                Some((PitchClass {
                    name: pitch_class,
                    accidental: pitch.0.accidental,
                }, pitch_octave))
            } else {
                Some((pitch.0, pitch.1))
            }
        })
    }

    /// Calculate note one step down within the key.
    /// - `create`: Note that is generated from a rest.
    pub fn step_down(&self, create: (PitchClass, PitchOctave)) -> Note {
        self.move_step(create, &|pitch| {
            let (pitch_class, offset) = match pitch.0.name {
                PitchName::A => (PitchName::G, false),
                PitchName::B => (PitchName::A, false),
                PitchName::C => (PitchName::B, true),
                PitchName::D => (PitchName::C, false),
                PitchName::E => (PitchName::D, false),
                PitchName::F => (PitchName::E, false),
                PitchName::G => (PitchName::F, false),
            };
            let pitch_octave = if offset {
                pitch.1.lower()
            } else {
                Some(pitch.1)
            };

            if let Some(pitch_octave) = pitch_octave {
                Some((PitchClass {
                    name: pitch_class,
                    accidental: pitch.0.accidental,
                }, pitch_octave))
            } else {
                Some((pitch.0, pitch.1))
            }
        })
    }
}
