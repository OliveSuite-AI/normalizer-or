//! Reviewed address labels and aliases, admitted only within address context.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Alias {
    Neighborhood,
    Avenue,
    Street,
}

impl Alias {
    pub(super) fn spoken(self) -> &'static str {
        match self {
            Self::Neighborhood => "mahalle",
            Self::Avenue => "cadde",
            Self::Street => "sokak",
        }
    }
}

pub(super) fn alias(word: &str) -> Option<Alias> {
    match word {
        "mah." | "Mah." | "MAH." | "mh." | "Mh." | "MH." => Some(Alias::Neighborhood),
        "cad." | "Cad." | "CAD." | "cd." | "Cd." | "CD." => Some(Alias::Avenue),
        "sk." | "Sk." | "SK." | "sok." | "Sok." | "SOK." => Some(Alias::Street),
        _ => None,
    }
}

pub(super) fn cue(word: &str) -> bool {
    matches!(
        word.strip_suffix(':').unwrap_or(word),
        "adres"
            | "Adres"
            | "ADRES"
            | "adresi"
            | "Adresi"
            | "ADRESİ"
            | "adresiniz"
            | "Adresiniz"
            | "ADRESİNİZ"
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Label {
    Number,
    Floor,
    Apartment,
    Postcode,
}

impl Label {
    pub(super) fn spoken(self) -> &'static str {
        match self {
            Self::Number => "numara ",
            Self::Floor => "kat ",
            Self::Apartment => "daire ",
            Self::Postcode => "posta kodu ",
        }
    }
}

pub(super) fn label(word: &str) -> Option<Label> {
    match word {
        "no" | "No" | "NO" => Some(Label::Number),
        "kat" | "Kat" | "KAT" => Some(Label::Floor),
        "d" | "D" | "daire" | "Daire" | "DAİRE" => Some(Label::Apartment),
        _ => None,
    }
}
