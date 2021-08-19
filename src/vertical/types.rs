use factorio_blueprint::objects::SignalIDType;



pub struct Statement {
    pub wires: Option<Vec<Wire>>,
    pub expr: Expr,
    pub out: Option<Vec<Name>>
}

pub enum Expr {
    Arithmetic(Arithmetic),
    Decider(Decider),
    Constant(Vec<ConstantItem>)
}

pub struct Arithmetic {
    pub left: SignalOrConstant,
    pub op: String,
    pub right: SignalOrConstant,
    pub out: Signal,
}

pub struct Decider {
    pub left: Signal,
    pub op: String,
    pub right: SignalOrConstant,
    pub out: Signal,
    pub copy_count: bool,
}

pub struct ConstantItem {
    pub signal: Signal,
    pub count: i32
}

pub struct Wire {
    pub loc: Location,
    pub color: WireColor
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum WireColor {
    Red,
    Green
}

/// [Name] or [Line]
pub type Location = String;

pub type Name = String;
/// eg. L123 or U123
pub type Line = String;

pub enum SignalOrConstant {
    Signal(Signal),
    Constant(i32)
}

pub struct Signal {
    pub type_: String,
    pub name: String
}

impl Signal {
    pub fn type_as_enum(&self) -> SignalIDType {
        match self.type_.as_str() {
            "item" => SignalIDType::Item,
            "fluid" => SignalIDType::Fluid,
            "virtual" => SignalIDType::Virtual,
            _ => unimplemented!("Error handling for invalid signalID")
        }
    }
}

pub type ConstantNumber = i32;