use crate::domain::Element;
use crate::functional_groups::FunctionalGroupKind;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternRole {
    Center,
    Other(String),
}

#[derive(Clone, Debug)]
pub struct FunctionalGroupPattern {
    pub kind: FunctionalGroupKind,
    pub atoms: Vec<PatternAtom>,
    pub bonds: Vec<PatternBond>,
    pub constraints: Vec<NeighborQuery>,
    pub attachment: Option<PatternAttachment>,
    pub reference: Option<PatternRole>,
}

#[derive(Clone, Debug)]
pub struct PatternAtom {
    pub role: PatternRole,
    pub element: PatternElement,
}

#[derive(Clone, Debug)]
pub struct PatternBond {
    pub left: PatternRole,
    pub right: PatternRole,
    pub order: u8,
}

#[derive(Clone, Debug)]
pub struct NeighborQuery {
    pub from: PatternRole,
    pub element: PatternElement,
    pub bond_order: Option<u8>,
    pub exclude: Vec<PatternRole>,
    pub min_count: usize,
}

#[derive(Clone, Debug)]
pub enum PatternAttachment {
    Role(PatternRole),
    Neighbor(NeighborQuery),
}

#[derive(Clone, Copy, Debug)]
pub enum PatternElement {
    AnyHalogen,
    Exact(Element),
}

pub static FUNCTIONAL_GROUP_PATTERNS: LazyLock<Vec<FunctionalGroupPattern>> = LazyLock::new(|| {
    vec![
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::CarboxylicAcid,
            atoms: vec![
                atom(PatternRole::Center, Element::C),
                atom(o_double(), Element::O),
                atom(o_single(), Element::O),
            ],
            bonds: vec![
                bond(PatternRole::Center, o_double(), 2),
                bond(PatternRole::Center, o_single(), 1),
            ],
            constraints: vec![neighbor(o_single(), Element::H, Some(1), [], 1)],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_single(), o_double()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::SulfonicAcid,
            atoms: vec![
                atom(PatternRole::Center, Element::S),
                atom(o_double(), Element::O),
                atom(o_second(), Element::O),
                atom(o_single(), Element::O),
            ],
            bonds: vec![
                bond(PatternRole::Center, o_double(), 2),
                bond(PatternRole::Center, o_second(), 2),
                bond(PatternRole::Center, o_single(), 1),
            ],
            constraints: vec![neighbor(o_single(), Element::H, Some(1), [], 1)],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_single(), o_double(), o_second()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Amide,
            atoms: vec![
                atom(PatternRole::Center, Element::C),
                atom(o_double(), Element::O),
                atom(n(), Element::N),
            ],
            bonds: vec![
                bond(PatternRole::Center, o_double(), 2),
                bond(PatternRole::Center, n(), 1),
            ],
            constraints: vec![],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [n(), o_double()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Nitrile,
            atoms: vec![atom(PatternRole::Center, Element::C), atom(n(), Element::N)],
            bonds: vec![bond(PatternRole::Center, n(), 3)],
            constraints: vec![],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [n()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Ester,
            atoms: vec![
                atom(PatternRole::Center, Element::C),
                atom(o_double(), Element::O),
                atom(o_single(), Element::O),
            ],
            bonds: vec![
                bond(PatternRole::Center, o_double(), 2),
                bond(PatternRole::Center, o_single(), 1),
            ],
            constraints: vec![neighbor(
                o_single(),
                Element::C,
                Some(1),
                [PatternRole::Center],
                1,
            )],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_single(), o_double()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Aldehyde,
            atoms: vec![
                atom(PatternRole::Center, Element::C),
                atom(o_double(), Element::O),
            ],
            bonds: vec![bond(PatternRole::Center, o_double(), 2)],
            constraints: vec![neighbor(PatternRole::Center, Element::H, Some(1), [], 1)],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_double()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Ketone,
            atoms: vec![
                atom(PatternRole::Center, Element::C),
                atom(o_double(), Element::O),
            ],
            bonds: vec![bond(PatternRole::Center, o_double(), 2)],
            constraints: vec![neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_double()],
                2,
            )],
            attachment: Some(PatternAttachment::Neighbor(neighbor(
                PatternRole::Center,
                Element::C,
                Some(1),
                [o_double()],
                1,
            ))),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Alcohol,
            atoms: vec![
                atom(PatternRole::Center, Element::O),
                atom(c1(), Element::C),
            ],
            bonds: vec![bond(PatternRole::Center, c1(), 1)],
            constraints: vec![neighbor(PatternRole::Center, Element::H, Some(1), [], 1)],
            attachment: Some(PatternAttachment::Role(c1())),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Amine,
            atoms: vec![
                atom(PatternRole::Center, Element::N),
                atom(c1(), Element::C),
            ],
            bonds: vec![bond(PatternRole::Center, c1(), 1)],
            constraints: vec![],
            attachment: Some(PatternAttachment::Role(c1())),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Alkene,
            atoms: vec![atom(c1(), Element::C), atom(c2(), Element::C)],
            bonds: vec![bond(c1(), c2(), 2)],
            constraints: vec![],
            attachment: None,
            reference: Some(c1()),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Alkyne,
            atoms: vec![atom(c1(), Element::C), atom(c2(), Element::C)],
            bonds: vec![bond(c1(), c2(), 3)],
            constraints: vec![],
            attachment: None,
            reference: Some(c1()),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Ether,
            atoms: vec![
                atom(PatternRole::Center, Element::O),
                atom(c1(), Element::C),
                atom(c2(), Element::C),
            ],
            bonds: vec![
                bond(PatternRole::Center, c1(), 1),
                bond(PatternRole::Center, c2(), 1),
            ],
            constraints: vec![],
            attachment: Some(PatternAttachment::Role(c1())),
            reference: Some(PatternRole::Center),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Halogen,
            atoms: vec![
                PatternAtom {
                    role: x(),
                    element: PatternElement::AnyHalogen,
                },
                atom(c1(), Element::C),
            ],
            bonds: vec![bond(x(), c1(), 1)],
            constraints: vec![],
            attachment: Some(PatternAttachment::Role(c1())),
            reference: Some(x()),
        },
        FunctionalGroupPattern {
            kind: FunctionalGroupKind::Nitro,
            atoms: vec![
                atom(PatternRole::Center, Element::N),
                atom(o_double(), Element::O),
                atom(o_single(), Element::O),
                atom(c1(), Element::C),
            ],
            bonds: vec![
                bond(PatternRole::Center, o_double(), 2),
                bond(PatternRole::Center, o_single(), 1),
                bond(PatternRole::Center, c1(), 1),
            ],
            constraints: vec![],
            attachment: Some(PatternAttachment::Role(c1())),
            reference: Some(PatternRole::Center),
        },
    ]
});

fn role(name: &str) -> PatternRole {
    PatternRole::Other(name.to_string())
}

fn o_double() -> PatternRole {
    role("carbonyl_o")
}

fn o_single() -> PatternRole {
    role("single_o")
}

fn o_second() -> PatternRole {
    role("second_o")
}

fn n() -> PatternRole {
    role("n")
}

fn c1() -> PatternRole {
    role("c1")
}

fn c2() -> PatternRole {
    role("c2")
}

fn x() -> PatternRole {
    role("x")
}

fn atom(role: PatternRole, element: Element) -> PatternAtom {
    PatternAtom {
        role,
        element: PatternElement::Exact(element),
    }
}

fn bond(left: PatternRole, right: PatternRole, order: u8) -> PatternBond {
    PatternBond { left, right, order }
}

fn neighbor<const N: usize>(
    from: PatternRole,
    element: Element,
    bond_order: Option<u8>,
    exclude: [PatternRole; N],
    min_count: usize,
) -> NeighborQuery {
    NeighborQuery {
        from,
        element: PatternElement::Exact(element),
        bond_order,
        exclude: exclude.into_iter().collect(),
        min_count,
    }
}
