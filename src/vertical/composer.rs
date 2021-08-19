use std::{collections::HashMap, error::Error, num::NonZeroUsize};

use factorio_blueprint::{
    Container,
    objects::*
};
use noisy_float::prelude::*;

use crate::vertical::wiring::CircuitConnection;

use super::{types::*, wiring::WiringCreator};

pub trait BlueprintComposer {
    fn create_blueprint(&self, name: &str, icons: Vec<SignalID>) -> Result<Container, Box<dyn Error>>;
}

impl BlueprintComposer for Vec<Statement> {
    fn create_blueprint(&self, name: &str, mut icons: Vec<SignalID>) -> Result<Container, Box<dyn Error>> {
        let icons = icons.drain(..).enumerate()
            .map(|(index, signal)| Icon {
                index: NonZeroUsize::new(index + 1).unwrap(),
                signal
            }).collect();

        macro_rules! some_wrap {
            {if let $variant:pat = $var:expr => $result:expr} => {
                if let $variant = $var { Some($result) } else { None }
            }
        }

        // Create the combinator entities
        let mut entities: Vec<Entity> = self.iter().enumerate()
            .map(|(idx, stmt)| {
                let entity_number = NonZeroUsize::new(idx + 1).unwrap();

                let name = match stmt.expr {
                    Expr::Arithmetic(_) => "arithmetic-combinator",
                    Expr::Decider(_) => "decider-combinator",
                    Expr::Constant(_) => "constant-combinator",
                }.to_string();

                let position = Position {x: r64(0.5), y: r64(2. * idx as f64 + 1.)};

                let control_behavior = match &stmt.expr {
                    Expr::Arithmetic(arith) => {
                        let conditions = ArithmeticConditions {
                            first_constant: some_wrap!{ if let SignalOrConstant::Constant(x) = arith.left => x },
                            first_signal: some_wrap!{ if let SignalOrConstant::Signal(x) = &arith.left =>
                                SignalID { type_: x.type_as_enum(), name: x.name.clone() }
                            },
                            second_constant: some_wrap!{ if let SignalOrConstant::Constant(x) = arith.right => x },
                            second_signal: some_wrap!{ if let SignalOrConstant::Signal(x) = &arith.right =>
                                SignalID { type_: x.type_as_enum(), name: x.name.clone() }
                            },
                            operation: arith.op.clone(),
                            output_signal: Some(SignalID { type_: arith.out.type_as_enum(), name: arith.out.name.clone() })
                        };

                        Some(ControlBehavior {
                            arithmetic_conditions: Some(conditions),
                            decider_conditions: None,
                            filters: None,
                            is_on: None,
                        })
                    },
                    Expr::Decider(decider) => {
                        let comparator = match decider.op.as_str() {
                            "==" => "=",
                            "<=" => "≤",
                            ">=" => "≥",
                            "/=" | "!=" | "~=" => "≠",
                            x => x
                        }.to_string();
                        let conditions = DeciderConditions {
                            first_signal: Some(SignalID { type_: decider.left.type_as_enum(), name: decider.left.name.clone() }),
                            constant: some_wrap!{ if let SignalOrConstant::Constant(x) = decider.right => x },
                            second_signal: some_wrap!{ if let SignalOrConstant::Signal(x) = &decider.right =>
                                SignalID { type_: x.type_as_enum(), name: x.name.clone() }
                            },
                            comparator,
                            output_signal: Some(SignalID { type_: decider.out.type_as_enum(), name: decider.out.name.clone() }),
                            copy_count_from_input: Some(decider.copy_count)
                        };

                        Some(ControlBehavior {
                            arithmetic_conditions: None,
                            decider_conditions: Some(conditions),
                            filters: None,
                            is_on: None,
                        })
                    },
                    Expr::Constant(constant) => {
                        let filters = constant.iter().enumerate()
                            .map(|(idx, item)| {
                                ConstantCombinatorFilter {
                                    signal: SignalID {
                                        type_: item.signal.type_as_enum(),
                                        name: item.signal.name.clone()
                                    },
                                    index: NonZeroUsize::new(idx + 1).unwrap(),
                                    count: item.count
                                }
                            }).collect();

                        Some(ControlBehavior {
                            arithmetic_conditions: None,
                            decider_conditions: None,
                            filters: Some(filters),
                            is_on: Some(true)
                        })
                    }
                };

                // FIXME: #[derive(Default)]
                Entity {
                    entity_number,
                    name,
                    position,
                    control_behavior,
                    direction: None,
                    orientation: None,
                    connections: None,
                    items: None,
                    recipe: None,
                    bar: None,
                    inventory: None,
                    infinity_settings: None,
                    type_: None,
                    input_priority: None,
                    output_priority: None,
                    filter: None,
                    filters: None,
                    filter_mode: None,
                    override_stack_size: None,
                    drop_position: None,
                    pickup_position: None,
                    request_filters: None,
                    request_from_buffers: None,
                    parameters: None,
                    alert_parameters: None,
                    auto_launch: None,
                    variation: None,
                    color: None,
                    station: None
                }
            }).collect();
        
        // And then add wiring onto it
        for circuit in self.create_wiring()?.drain(..) {
            let mut add_entry = |source: &CircuitConnection, target: &CircuitConnection| {
                let source_object = ConnectionData {
                    entity_id: NonZeroUsize::new(target.idx + 1).unwrap(),
                    circuit_id: Some(target.conn_point.get() as i32),
                };

                if let Some(EntityConnections::NumberIdx(start_conn)) = &mut entities[source.idx].connections {

                    if let Some(start_point) = start_conn.get_mut(&source.conn_point) {
                        if let Some(modifiable) = match circuit.color {
                            WireColor::Red => start_point.red.as_mut(),
                            WireColor::Green => start_point.green.as_mut(),
                        } {
                            modifiable.push(source_object);
                        } else {
                            match circuit.color {
                                WireColor::Red => start_point.red = Some(vec![source_object]),
                                WireColor::Green => start_point.green = Some(vec![source_object]),
                            }
                        }
                    } else {
                        start_conn.insert(source.conn_point, ConnectionPoint {
                            red: some_wrap!{ if let WireColor::Red = circuit.color => vec![source_object.clone()] },
                            green: some_wrap!{ if let WireColor::Green = circuit.color => vec![source_object] },
                        });
                    }
                } else {
                    let mut start_conn = HashMap::new();
                    
                    start_conn.insert(source.conn_point, ConnectionPoint {
                        red: some_wrap!{ if let WireColor::Red = circuit.color => vec![source_object.clone()] },
                        green: some_wrap!{ if let WireColor::Green = circuit.color => vec![source_object] },
                    });

                    entities[source.idx].connections = Some(EntityConnections::NumberIdx(start_conn));
                }
            };

            for (start, end) in circuit.connections.iter().zip(circuit.connections.iter().skip(1)) {
                add_entry(start, end);
                add_entry(end, start);
            }
        }

        let blueprint = Blueprint {
            label: name.to_string(),
            icons,
            entities,
            ..Default::default()
        };

        Ok(Container::Blueprint(blueprint))
    }
}
