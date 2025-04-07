use super::*;
use crate::type_::{FieldMap, HasType};

pub type TypedConstant = Constant<Arc<Type>, EcoString>;
pub type UntypedConstant = Constant<(), ()>;

// TODO: remove RecordTag paramter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant<T, RecordTag> {
    Int {
        location: SrcSpan,
        value: EcoString,
        int_value: BigInt,
    },

    Float {
        location: SrcSpan,
        value: EcoString,
    },

    String {
        location: SrcSpan,
        value: EcoString,
    },

    Tuple {
        location: SrcSpan,
        elements: Vec<Self>,
    },

    List {
        location: SrcSpan,
        elements: Vec<Self>,
        type_: T,
    },

    Record {
        location: SrcSpan,
        module: Option<(EcoString, SrcSpan)>,
        name: EcoString,
        args: Vec<CallArg<Self>>,
        tag: RecordTag,
        type_: T,
        field_map: Option<FieldMap>,
    },

    BitArray {
        location: SrcSpan,
        segments: Vec<BitArraySegment<Self, T>>,
    },

    Var {
        location: SrcSpan,
        module: Option<(EcoString, SrcSpan)>,
        name: EcoString,
        constructor: Option<Box<ValueConstructor>>,
        type_: T,
    },

    StringConcatenation {
        location: SrcSpan,
        left: Box<Self>,
        right: Box<Self>,
    },

    /// A placeholder constant used to allow module analysis to continue
    /// even when there are type errors. Should never end up in generated code.
    Invalid {
        location: SrcSpan,
        type_: T,
    },
}

impl TypedConstant {
    pub fn type_(&self) -> Arc<Type> {
        match self {
            Constant::Int { .. } => type_::int(),
            Constant::Float { .. } => type_::float(),
            Constant::String { .. } | Constant::StringConcatenation { .. } => type_::string(),
            Constant::BitArray { .. } => type_::bits(),
            Constant::Tuple { elements, .. } => {
                type_::tuple(elements.iter().map(|element| element.type_()).collect())
            }
            Constant::List { type_, .. }
            | Constant::Record { type_, .. }
            | Constant::Var { type_, .. }
            | Constant::Invalid { type_, .. } => type_.clone(),
        }
    }

    pub fn find_node(&self, byte_index: u32) -> Option<Located<'_>> {
        if !self.location().contains(byte_index) {
            return None;
        }
        Some(match self {
            Constant::Int { .. }
            | Constant::Float { .. }
            | Constant::String { .. }
            | Constant::Var { .. }
            | Constant::Invalid { .. } => Located::Constant(self),
            Constant::Tuple { elements, .. } | Constant::List { elements, .. } => elements
                .iter()
                .find_map(|element| element.find_node(byte_index))
                .unwrap_or(Located::Constant(self)),
            Constant::Record { args, .. } => args
                .iter()
                .find_map(|argument| argument.find_node(byte_index))
                .unwrap_or(Located::Constant(self)),
            Constant::BitArray { segments, .. } => segments
                .iter()
                .find_map(|segment| segment.find_node(byte_index))
                .unwrap_or(Located::Constant(self)),
            Constant::StringConcatenation { left, right, .. } => left
                .find_node(byte_index)
                .or_else(|| right.find_node(byte_index))
                .unwrap_or(Located::Constant(self)),
        })
    }
}

impl HasType for TypedConstant {
    fn type_(&self) -> Arc<Type> {
        self.type_()
    }
}

impl<A, B> Constant<A, B> {
    pub fn location(&self) -> SrcSpan {
        match self {
            Constant::Int { location, .. }
            | Constant::List { location, .. }
            | Constant::Float { location, .. }
            | Constant::Tuple { location, .. }
            | Constant::String { location, .. }
            | Constant::Record { location, .. }
            | Constant::BitArray { location, .. }
            | Constant::Var { location, .. }
            | Constant::Invalid { location, .. }
            | Constant::StringConcatenation { location, .. } => *location,
        }
    }

    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            Self::Int { .. } | Self::Float { .. } | Self::String { .. }
        )
    }
}

impl<A, B> HasLocation for Constant<A, B> {
    fn location(&self) -> SrcSpan {
        self.location()
    }
}

impl<A, B> crate::bit_array::GetLiteralValue for Constant<A, B> {
    fn as_int_literal(&self) -> Option<i64> {
        if let Constant::Int { value, .. } = self {
            if let Ok(val) = value.parse::<i64>() {
                return Some(val);
            }
        }
        None
    }
}
