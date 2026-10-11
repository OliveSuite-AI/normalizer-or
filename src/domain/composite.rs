use crate::{
    SegmentKind, SourceRange,
    domain::{numeric::Numeric, quantities::Quantity},
};

/// Components are validated together; their ranges partition one source owner.
#[derive(Clone, Debug)]
pub(crate) struct Composite {
    pub(crate) components: Vec<Component>,
}

#[derive(Clone, Debug)]
pub(crate) struct Component {
    pub(crate) range: SourceRange,
    pub(crate) value: ComponentValue,
}

#[derive(Clone, Debug)]
pub(crate) enum ComponentValue {
    Numeric(Numeric),
    Quantity(Quantity),
    Text {
        kind: SegmentKind,
        rule_id: &'static str,
        text: String,
    },
}
