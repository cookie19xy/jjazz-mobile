/// Chord degrees (root, 3rd, 5th, 7th, extensions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Degree {
    Root,
    NinthFlat, Ninth, NinthSharp,
    ThirdFlat, Third,
    FourthOrEleventh, EleventhSharp,
    FifthFlat, Fifth, FifthSharp,
    ThirteenthFlat, SixthOrThirteenth,
    SeventhFlat, Seventh,
}

/// Natural degree categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Natural { Root, Ninth, Third, Eleventh, Fifth, Sixth, Seventh }

impl Natural {
    pub fn pitch(self) -> u8 {
        match self { Natural::Root=>0, Natural::Ninth=>2, Natural::Third=>4,
            Natural::Eleventh=>5, Natural::Fifth=>7, Natural::Sixth=>9, Natural::Seventh=>11 }
    }
    pub fn int_value(self) -> u8 {
        match self { Natural::Root=>1, Natural::Ninth=>9, Natural::Third=>3,
            Natural::Eleventh=>11, Natural::Fifth=>5, Natural::Sixth=>13, Natural::Seventh=>7 }
    }
    pub fn from_rel_pitch(rp: u8) -> Option<Natural> {
        match rp { 0=>Some(Natural::Root), 2=>Some(Natural::Ninth), 4=>Some(Natural::Third),
            5=>Some(Natural::Eleventh), 7=>Some(Natural::Fifth), 9=>Some(Natural::Sixth), 11=>Some(Natural::Seventh), _=>None }
    }
}

impl Degree {
    pub fn pitch(self) -> u8 { (self.natural().pitch() as i16 + self.accidental() as i16) as u8 % 12 }
    pub fn natural(self) -> Natural {
        match self { Degree::Root=>Natural::Root, Degree::NinthFlat|Degree::Ninth|Degree::NinthSharp=>Natural::Ninth,
            Degree::ThirdFlat|Degree::Third=>Natural::Third, Degree::FourthOrEleventh|Degree::EleventhSharp=>Natural::Eleventh,
            Degree::FifthFlat|Degree::Fifth|Degree::FifthSharp=>Natural::Fifth,
            Degree::ThirteenthFlat|Degree::SixthOrThirteenth=>Natural::Sixth,
            Degree::SeventhFlat|Degree::Seventh=>Natural::Seventh }
    }
    pub fn accidental(self) -> i8 {
        match self { Degree::Root|Degree::Ninth|Degree::Third|Degree::FourthOrEleventh|Degree::Fifth|Degree::SixthOrThirteenth|Degree::Seventh=>0,
            Degree::NinthFlat|Degree::ThirdFlat|Degree::FifthFlat|Degree::ThirteenthFlat|Degree::SeventhFlat=>-1,
            Degree::NinthSharp|Degree::EleventhSharp|Degree::FifthSharp=>1 }
    }
    pub fn short_name(self) -> String {
        let n = self.natural().int_value();
        match self.accidental() { -1=>format!("b{}",n), 1=>format!("#{}",n), _=>format!("{}",n) }
    }
    pub fn from_natural_alt(n: Natural, alt: i8) -> Option<Degree> {
        match (n, alt) {
            (Natural::Root,0)=>Some(Degree::Root),
            (Natural::Ninth,-1)=>Some(Degree::NinthFlat),(Natural::Ninth,0)=>Some(Degree::Ninth),(Natural::Ninth,1)=>Some(Degree::NinthSharp),
            (Natural::Third,-1)=>Some(Degree::ThirdFlat),(Natural::Third,0)=>Some(Degree::Third),
            (Natural::Eleventh,0)=>Some(Degree::FourthOrEleventh),(Natural::Eleventh,1)=>Some(Degree::EleventhSharp),
            (Natural::Fifth,-1)=>Some(Degree::FifthFlat),(Natural::Fifth,0)=>Some(Degree::Fifth),(Natural::Fifth,1)=>Some(Degree::FifthSharp),
            (Natural::Sixth,-1)=>Some(Degree::ThirteenthFlat),(Natural::Sixth,0)=>Some(Degree::SixthOrThirteenth),
            (Natural::Seventh,-1)=>Some(Degree::SeventhFlat),(Natural::Seventh,0)=>Some(Degree::Seventh),
            _=>None
        }
    }
    pub fn most_probable(rel_pitch: u8) -> Degree {
        match rel_pitch { 0=>Degree::Root,1=>Degree::NinthFlat,2=>Degree::Ninth,3=>Degree::ThirdFlat,
            4=>Degree::Third,5=>Degree::FourthOrEleventh,6=>Degree::FifthFlat,7=>Degree::Fifth,
            8=>Degree::FifthSharp,9=>Degree::SixthOrThirteenth,10=>Degree::SeventhFlat,11=>Degree::Seventh,_=>panic!() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_degree_pitch() {
        assert_eq!(Degree::Root.pitch(), 0);
        assert_eq!(Degree::Third.pitch(), 4);
        assert_eq!(Degree::ThirdFlat.pitch(), 3);
        assert_eq!(Degree::Fifth.pitch(), 7);
        assert_eq!(Degree::Seventh.pitch(), 11);
        assert_eq!(Degree::SeventhFlat.pitch(), 10);
    }
}
