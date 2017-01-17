use ::serde::Serialize;

#[derive(Copy,Clone)]
pub enum PenaltyType {
    BackBlock,
    LowBlock,
    HighBlock,
    Forearms,
    Elbows,
    BlockWithHead,
    MultiPlayer,
    Directional,
    CutTrack,
    IllegalProcedure,
    OutOfPlay,
    OutOfBounds,
    SkatingOutOfBounds,
    Insubordination,
    Misconduct,
    DelayOfGame,
    Unknown,
}

impl PenaltyType {
    pub fn from_char(code: char) -> PenaltyType {
        match code {
            'A' => PenaltyType::HighBlock,
            'B' => PenaltyType::BackBlock,
            'C' => PenaltyType::Directional,
            'E' => PenaltyType::Elbows,
            'F' => PenaltyType::Forearms,
            'G' => PenaltyType::Misconduct,
            'H' => PenaltyType::BlockWithHead,
            'I' => PenaltyType::IllegalProcedure,
            'L' => PenaltyType::LowBlock,
            'M' => PenaltyType::MultiPlayer,
            'N' => PenaltyType::Insubordination,
            'O' => PenaltyType::OutOfBounds,
            'P' => PenaltyType::OutOfPlay,
            'S' => PenaltyType::SkatingOutOfBounds,
            'X' => PenaltyType::CutTrack,
            'Z' => PenaltyType::DelayOfGame,
            _ => PenaltyType::Unknown,
        }
    }
    pub fn as_char(self) -> char {
        match self {
            PenaltyType::BackBlock => 'B',
            PenaltyType::LowBlock => 'L',
            PenaltyType::HighBlock => 'A',
            PenaltyType::Forearms => 'F',
            PenaltyType::Elbows => 'E',
            PenaltyType::BlockWithHead => 'H',
            PenaltyType::MultiPlayer => 'M',
            PenaltyType::Directional => 'C',
            PenaltyType::CutTrack => 'X',
            PenaltyType::IllegalProcedure => 'I',
            PenaltyType::OutOfPlay => 'P',
            PenaltyType::OutOfBounds => 'B',
            PenaltyType::SkatingOutOfBounds => 'S',
            PenaltyType::Insubordination => 'N',
            PenaltyType::Misconduct => 'G',
            PenaltyType::DelayOfGame => 'Z',
            PenaltyType::Unknown => 'U',
        }
    }
}

impl Serialize for PenaltyType {
    fn serialize<S: ::serde::Serializer>(&self, serializer: &mut S)
                                       -> Result<(), S::Error> {
        serializer.serialize_char(self.as_char())
    }
}
