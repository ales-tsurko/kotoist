use std::convert::TryFrom;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Scale {
    Acoustic,
    Minor,
    Aeolian,
    Altered,
    Augmented,
    BebopDominant,
    Blues,
    Chromatic,
    Dorian,
    DoubleHarmonic,
    Enigmatic,
    Flamenco,
    Gypsy,
    HalfDiminished,
    HarmonicMajor,
    HarmonicMinor,
    Hirajoshi,
    HungarianGypsy,
    HungarianMinor,
    In,
    Insen,
    Major,
    Ionian,
    Iwato,
    Locrian,
    LydianAugmented,
    Lydian,
    MajorBebop,
    MajorLocrian,
    MajorPentatonic,
    MelodicMinor,
    MinorPentatonic,
    Mixolydian,
    AdonaiMalakh,
    NeapolitanMajor,
    NeapolitanMinor,
    Persian,
    PhrygianDominant,
    Phrygian,
    Prometheus,
    Spectral,
    Tritone,
    UkrainianDorian,
    WholeTone,
    Yo,
}

impl Scale {
    /// Returns a String for printing all available scales.
    pub(crate) fn list() -> String {
        "
            acoustic          => 0, 2, 4, 6, 7, 9, 10
            adonai-malakh     => 0, 2, 4, 5, 7, 9, 10
            aeolian           => 0, 2, 3, 5, 7, 8, 10
            altered           => 0, 1, 3, 4, 6, 8, 10
            augmented         => 0, 3, 4, 7, 8, 11
            bebop-dominant    => 0, 2, 4, 5, 7, 9, 10, 11
            blues             => 0, 3, 5, 6, 7, 10
            chromatic         => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
            dorian            => 0, 2, 3, 5, 7, 9, 10
            double-harmonic   => 0, 1, 4, 5, 7, 8, 11
            enigmatic         => 0, 1, 4, 6, 8, 10, 11
            flamenco          => 0, 1, 4, 5, 7, 8, 11
            gypsy             => 0, 2, 3, 6, 7, 8, 10
            half-diminished   => 0, 2, 3, 5, 6, 8, 10
            harmonic-major    => 0, 2, 4, 5, 7, 8, 11
            harmonic-minor    => 0, 2, 3, 5, 7, 8, 11
            hirajoshi         => 0, 4, 6, 7, 11
            hungarian-gypsy   => 0, 2, 3, 6, 7, 8, 11
            hungarian-minor   => 0, 2, 3, 6, 7, 8, 11
            in                => 0, 1, 5, 7, 8
            insen             => 0, 1, 5, 7, 10
            ionian            => 0, 2, 4, 5, 7, 9, 11
            iwato             => 0, 1, 5, 6, 10
            locrian           => 0, 1, 3, 5, 6, 8, 10
            lydian-augmented  => 0, 2, 4, 6, 8, 9, 11
            lydian            => 0, 2, 4, 6, 7, 9, 11
            major             => 0, 2, 4, 5, 7, 9, 11
            major-bebop       => 0, 2, 4, 5, 7, 8, 9, 11
            major-locrian     => 0, 2, 4, 5, 6, 8, 10
            major-pentatonic  => 0, 2, 4, 7, 9
            melodic-minor     => 0, 2, 3, 5, 7, 9, 11
            minor             => 0, 2, 3, 5, 7, 8, 10
            minor-pentatonic  => 0, 3, 5, 7, 10
            mixolydian        => 0, 2, 4, 5, 7, 9, 10
            neapolitan-major  => 0, 1, 3, 5, 7, 9, 11
            neapolitan-minor  => 0, 1, 3, 5, 7, 8, 11
            persian           => 0, 1, 4, 5, 6, 8, 11
            phrygian-dominant => 0, 1, 4, 5, 7, 8, 10
            phrygian          => 0, 1, 3, 5, 7, 8, 10
            prometheus        => 0, 2, 4, 6, 9, 10
            spectral          => 0, 3, 4, 5, 7, 9
            tritone           => 0, 1, 4, 6, 7, 10
            ukrainian-dorian  => 0, 2, 3, 6, 7, 9, 10
            whole-tone        => 0, 2, 4, 6, 8, 10
            yo                => 0, 3, 5, 7, 10
            "
        .to_string()
    }
}

impl From<Scale> for &[f64] {
    fn from(scale: Scale) -> Self {
        match scale {
            Scale::Acoustic => &[0.0, 2.0, 4.0, 6.0, 7.0, 9.0, 10.0],
            Scale::Minor => &[0.0, 2.0, 3.0, 5.0, 7.0, 8.0, 10.0],
            Scale::Aeolian => &[0.0, 2.0, 3.0, 5.0, 7.0, 8.0, 10.0],
            Scale::Altered => &[0.0, 1.0, 3.0, 4.0, 6.0, 8.0, 10.0],
            Scale::Augmented => &[0.0, 3.0, 4.0, 7.0, 8.0, 11.0],
            Scale::BebopDominant => &[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 10.0, 11.0],
            Scale::Blues => &[0.0, 3.0, 5.0, 6.0, 7.0, 10.0],
            Scale::Chromatic => &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0],
            Scale::Dorian => &[0.0, 2.0, 3.0, 5.0, 7.0, 9.0, 10.0],
            Scale::DoubleHarmonic => &[0.0, 1.0, 4.0, 5.0, 7.0, 8.0, 11.0],
            Scale::Enigmatic => &[0.0, 1.0, 4.0, 6.0, 8.0, 10.0, 11.0],
            Scale::Flamenco => &[0.0, 1.0, 4.0, 5.0, 7.0, 8.0, 11.0],
            Scale::Gypsy => &[0.0, 2.0, 3.0, 6.0, 7.0, 8.0, 10.0],
            Scale::HalfDiminished => &[0.0, 2.0, 3.0, 5.0, 6.0, 8.0, 10.0],
            Scale::HarmonicMajor => &[0.0, 2.0, 4.0, 5.0, 7.0, 8.0, 11.0],
            Scale::HarmonicMinor => &[0.0, 2.0, 3.0, 5.0, 7.0, 8.0, 11.0],
            Scale::Hirajoshi => &[0.0, 4.0, 6.0, 7.0, 11.0],
            Scale::HungarianGypsy => &[0.0, 2.0, 3.0, 6.0, 7.0, 8.0, 11.0],
            Scale::HungarianMinor => &[0.0, 2.0, 3.0, 6.0, 7.0, 8.0, 11.0],
            Scale::In => &[0.0, 1.0, 5.0, 7.0, 8.0],
            Scale::Insen => &[0.0, 1.0, 5.0, 7.0, 10.0],
            Scale::Major => &[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0],
            Scale::Ionian => &[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0],
            Scale::Iwato => &[0.0, 1.0, 5.0, 6.0, 10.0],
            Scale::Locrian => &[0.0, 1.0, 3.0, 5.0, 6.0, 8.0, 10.0],
            Scale::LydianAugmented => &[0.0, 2.0, 4.0, 6.0, 8.0, 9.0, 11.0],
            Scale::Lydian => &[0.0, 2.0, 4.0, 6.0, 7.0, 9.0, 11.0],
            Scale::MajorBebop => &[0.0, 2.0, 4.0, 5.0, 7.0, 8.0, 9.0, 11.0],
            Scale::MajorLocrian => &[0.0, 2.0, 4.0, 5.0, 6.0, 8.0, 10.0],
            Scale::MajorPentatonic => &[0.0, 2.0, 4.0, 7.0, 9.0],
            Scale::MelodicMinor => &[0.0, 2.0, 3.0, 5.0, 7.0, 9.0, 11.0],
            Scale::MinorPentatonic => &[0.0, 3.0, 5.0, 7.0, 10.0],
            Scale::Mixolydian => &[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 10.0],
            Scale::AdonaiMalakh => &[0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 10.0],
            Scale::NeapolitanMajor => &[0.0, 1.0, 3.0, 5.0, 7.0, 9.0, 11.0],
            Scale::NeapolitanMinor => &[0.0, 1.0, 3.0, 5.0, 7.0, 8.0, 11.0],
            Scale::Persian => &[0.0, 1.0, 4.0, 5.0, 6.0, 8.0, 11.0],
            Scale::PhrygianDominant => &[0.0, 1.0, 4.0, 5.0, 7.0, 8.0, 10.0],
            Scale::Phrygian => &[0.0, 1.0, 3.0, 5.0, 7.0, 8.0, 10.0],
            Scale::Prometheus => &[0.0, 2.0, 4.0, 6.0, 9.0, 10.0],
            Scale::Spectral => &[0.0, 3.0, 4.0, 5.0, 7.0, 9.0],
            Scale::Tritone => &[0.0, 1.0, 4.0, 6.0, 7.0, 10.0],
            Scale::UkrainianDorian => &[0.0, 2.0, 3.0, 6.0, 7.0, 9.0, 10.0],
            Scale::WholeTone => &[0.0, 2.0, 4.0, 6.0, 8.0, 10.0],
            Scale::Yo => &[0.0, 3.0, 5.0, 7.0, 10.0],
        }
    }
}

impl TryFrom<&str> for Scale {
    type Error = ScaleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ACOUSTIC" => Ok(Scale::Acoustic),
            "MINOR" => Ok(Scale::Minor),
            "AEOLIAN" => Ok(Scale::Aeolian),
            "ALTERED" => Ok(Scale::Altered),
            "AUGMENTED" => Ok(Scale::Augmented),
            "BEBOP-DOMINANT" => Ok(Scale::BebopDominant),
            "BLUES" => Ok(Scale::Blues),
            "CHROMATIC" => Ok(Scale::Chromatic),
            "DORIAN" => Ok(Scale::Dorian),
            "DOUBLE-HARMONIC" => Ok(Scale::DoubleHarmonic),
            "ENIGMATIC" => Ok(Scale::Enigmatic),
            "FLAMENCO" => Ok(Scale::Flamenco),
            "GYPSY" => Ok(Scale::Gypsy),
            "HALF-DIMINISHED" => Ok(Scale::HalfDiminished),
            "HARMONIC-MAJOR" => Ok(Scale::HarmonicMajor),
            "HARMONIC-MINOR" => Ok(Scale::HarmonicMinor),
            "HIRAJOSHI" => Ok(Scale::Hirajoshi),
            "HUNGARIAN-GYPSY" => Ok(Scale::HungarianGypsy),
            "HUNGARIAN-MINOR" => Ok(Scale::HungarianMinor),
            "IN" => Ok(Scale::In),
            "INSEN" => Ok(Scale::Insen),
            "MAJOR" => Ok(Scale::Major),
            "IONIAN" => Ok(Scale::Ionian),
            "IWATO" => Ok(Scale::Iwato),
            "LOCRIAN" => Ok(Scale::Locrian),
            "LYDIAN-AUGMENTED" => Ok(Scale::LydianAugmented),
            "LYDIAN" => Ok(Scale::Lydian),
            "MAJOR-BEBOP" => Ok(Scale::MajorBebop),
            "MAJOR-LOCRIAN" => Ok(Scale::MajorLocrian),
            "MAJOR-PENTATONIC" => Ok(Scale::MajorPentatonic),
            "MELODIC-MINOR" => Ok(Scale::MelodicMinor),
            "MINOR-PENTATONIC" => Ok(Scale::MinorPentatonic),
            "MIXOLYDIAN" => Ok(Scale::Mixolydian),
            "ADONAI-MALAKH" => Ok(Scale::AdonaiMalakh),
            "NEAPOLITAN-MAJOR" => Ok(Scale::NeapolitanMajor),
            "NEAPOLITAN-MINOR" => Ok(Scale::NeapolitanMinor),
            "PERSIAN" => Ok(Scale::Persian),
            "PHRYGIAN-DOMINANT" => Ok(Scale::PhrygianDominant),
            "PHRYGIAN" => Ok(Scale::Phrygian),
            "PROMETHEUS" => Ok(Scale::Prometheus),
            "SPECTRAL" => Ok(Scale::Spectral),
            "TRITONE" => Ok(Scale::Tritone),
            "UKRAINIAN-DORIAN" => Ok(Scale::UkrainianDorian),
            "WHOLE-TONE" => Ok(Scale::WholeTone),
            "YO" => Ok(Scale::Yo),
            _ => Err(ScaleError::UnknownScale(value.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum ScaleError {
    #[error("The scale '{0}' is unknown.")]
    UnknownScale(String),
}
