use crate::domain::Element;
use crate::functional_groups::FunctionalGroupKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PatternRole {
    Center,
    Attachment,
    CarbonylO,
    SingleO,
    SecondO,
    H,
    N,
    C1,
    C2,
    X,
}

#[derive(Clone, Copy, Debug)]
pub struct FunctionalGroupPattern {
    pub kind: FunctionalGroupKind,
    pub atoms: &'static [PatternAtom],
    pub bonds: &'static [PatternBond],
    pub constraints: &'static [PatternConstraint],
    pub attachment: Option<PatternAttachment>,
    pub reference: Option<PatternRole>,
}

#[derive(Clone, Copy, Debug)]
pub struct PatternAtom {
    pub role: PatternRole,
    pub element: PatternElement,
}

#[derive(Clone, Copy, Debug)]
pub struct PatternBond {
    pub left: PatternRole,
    pub right: PatternRole,
    pub order: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum PatternConstraint {
    HasHydrogenNeighbor {
        role: PatternRole,
    },
    HasCarbonNeighborExcept {
        role: PatternRole,
        except: PatternRole,
    },
    CarbonNeighborCountAtLeast {
        role: PatternRole,
        count: usize,
        except: PatternRole,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum PatternAttachment {
    Role(PatternRole),
    CarbonNeighborOf {
        role: PatternRole,
    },
    CarbonNeighborOfExcept {
        role: PatternRole,
        except: PatternRole,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum PatternElement {
    AnyHalogen,
    Exact(Element),
}

pub const FUNCTIONAL_GROUP_PATTERNS: &[FunctionalGroupPattern] = &[
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::CarboxylicAcid,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::CarbonylO, Element::O),
            atom(PatternRole::SingleO, Element::O),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::CarbonylO, 2),
            bond(PatternRole::Center, PatternRole::SingleO, 1),
        ],
        constraints: &[PatternConstraint::HasHydrogenNeighbor {
            role: PatternRole::SingleO,
        }],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::SingleO,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::SulfonicAcid,
        atoms: &[
            atom(PatternRole::Center, Element::S),
            atom(PatternRole::CarbonylO, Element::O),
            atom(PatternRole::SecondO, Element::O),
            atom(PatternRole::SingleO, Element::O),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::CarbonylO, 2),
            bond(PatternRole::Center, PatternRole::SecondO, 2),
            bond(PatternRole::Center, PatternRole::SingleO, 1),
        ],
        constraints: &[PatternConstraint::HasHydrogenNeighbor {
            role: PatternRole::SingleO,
        }],
        attachment: Some(PatternAttachment::CarbonNeighborOf {
            role: PatternRole::Center,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Amide,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::CarbonylO, Element::O),
            atom(PatternRole::N, Element::N),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::CarbonylO, 2),
            bond(PatternRole::Center, PatternRole::N, 1),
        ],
        constraints: &[],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::N,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Nitrile,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::N, Element::N),
        ],
        bonds: &[bond(PatternRole::Center, PatternRole::N, 3)],
        constraints: &[],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::N,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Ester,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::CarbonylO, Element::O),
            atom(PatternRole::SingleO, Element::O),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::CarbonylO, 2),
            bond(PatternRole::Center, PatternRole::SingleO, 1),
        ],
        constraints: &[PatternConstraint::HasCarbonNeighborExcept {
            role: PatternRole::SingleO,
            except: PatternRole::Center,
        }],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::SingleO,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Aldehyde,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::CarbonylO, Element::O),
        ],
        bonds: &[bond(PatternRole::Center, PatternRole::CarbonylO, 2)],
        constraints: &[PatternConstraint::HasHydrogenNeighbor {
            role: PatternRole::Center,
        }],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::CarbonylO,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Ketone,
        atoms: &[
            atom(PatternRole::Center, Element::C),
            atom(PatternRole::CarbonylO, Element::O),
        ],
        bonds: &[bond(PatternRole::Center, PatternRole::CarbonylO, 2)],
        constraints: &[PatternConstraint::CarbonNeighborCountAtLeast {
            role: PatternRole::Center,
            count: 2,
            except: PatternRole::CarbonylO,
        }],
        attachment: Some(PatternAttachment::CarbonNeighborOfExcept {
            role: PatternRole::Center,
            except: PatternRole::CarbonylO,
        }),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Alcohol,
        atoms: &[
            atom(PatternRole::Center, Element::O),
            atom(PatternRole::C1, Element::C),
        ],
        bonds: &[bond(PatternRole::Center, PatternRole::C1, 1)],
        constraints: &[PatternConstraint::HasHydrogenNeighbor {
            role: PatternRole::Center,
        }],
        attachment: Some(PatternAttachment::Role(PatternRole::C1)),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Amine,
        atoms: &[
            atom(PatternRole::Center, Element::N),
            atom(PatternRole::C1, Element::C),
        ],
        bonds: &[bond(PatternRole::Center, PatternRole::C1, 1)],
        constraints: &[],
        attachment: Some(PatternAttachment::Role(PatternRole::C1)),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Alkene,
        atoms: &[
            atom(PatternRole::C1, Element::C),
            atom(PatternRole::C2, Element::C),
        ],
        bonds: &[bond(PatternRole::C1, PatternRole::C2, 2)],
        constraints: &[],
        attachment: None,
        reference: Some(PatternRole::C1),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Alkyne,
        atoms: &[
            atom(PatternRole::C1, Element::C),
            atom(PatternRole::C2, Element::C),
        ],
        bonds: &[bond(PatternRole::C1, PatternRole::C2, 3)],
        constraints: &[],
        attachment: None,
        reference: Some(PatternRole::C1),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Ether,
        atoms: &[
            atom(PatternRole::Center, Element::O),
            atom(PatternRole::C1, Element::C),
            atom(PatternRole::C2, Element::C),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::C1, 1),
            bond(PatternRole::Center, PatternRole::C2, 1),
        ],
        constraints: &[],
        attachment: Some(PatternAttachment::Role(PatternRole::C1)),
        reference: Some(PatternRole::Center),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Halogen,
        atoms: &[
            PatternAtom {
                role: PatternRole::X,
                element: PatternElement::AnyHalogen,
            },
            atom(PatternRole::C1, Element::C),
        ],
        bonds: &[bond(PatternRole::X, PatternRole::C1, 1)],
        constraints: &[],
        attachment: Some(PatternAttachment::Role(PatternRole::C1)),
        reference: Some(PatternRole::X),
    },
    FunctionalGroupPattern {
        kind: FunctionalGroupKind::Nitro,
        atoms: &[
            atom(PatternRole::Center, Element::N),
            atom(PatternRole::CarbonylO, Element::O),
            atom(PatternRole::SingleO, Element::O),
            atom(PatternRole::C1, Element::C),
        ],
        bonds: &[
            bond(PatternRole::Center, PatternRole::CarbonylO, 2),
            bond(PatternRole::Center, PatternRole::SingleO, 1),
            bond(PatternRole::Center, PatternRole::C1, 1),
        ],
        constraints: &[],
        attachment: Some(PatternAttachment::Role(PatternRole::C1)),
        reference: Some(PatternRole::Center),
    },
];

const fn atom(role: PatternRole, element: Element) -> PatternAtom {
    PatternAtom {
        role,
        element: PatternElement::Exact(element),
    }
}

const fn bond(left: PatternRole, right: PatternRole, order: u8) -> PatternBond {
    PatternBond { left, right, order }
}
