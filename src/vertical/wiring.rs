use std::error::Error;
use std::collections::HashMap;
use std::num::NonZeroUsize;

use super::types::*;

#[derive(Debug)]
pub struct CircuitConnection {
    pub idx: usize,
    /// 1 for U side, 2 for L side
    pub conn_point: NonZeroUsize
}

#[derive(Debug)]
pub struct Circuit {
    pub color: WireColor,
    pub connections: Vec<CircuitConnection>
}

pub trait WiringCreator {
    fn create_wiring(&self) -> Result<Vec<Circuit>, Box<dyn Error>>;
}

impl WiringCreator for Vec<Statement> {
    fn create_wiring(&self) -> Result<Vec<Circuit>, Box<dyn Error>> {
        // First pass, map the connections for the wires

        // Source -> last_reference
        let reference_colors: HashMap<(String, WireColor), String> = {
            let mut reference_colors: HashMap<(String, WireColor), String> = HashMap::new();

            for (idx, stmt) in self.iter().enumerate() {
                // Line numbers
                let l_number = format!("L{}", idx);
                let u_number = format!("U{}", idx);

                let (l_red_reference, l_green_reference) = {
                    let find_l_reference_color = |color: WireColor| {
                        if let Some(reference) = reference_colors.get(&(l_number.clone(), color)) {
                            return reference.clone();
                        }

                        for out in stmt.out.iter().flatten() {
                            if let Some(reference) = reference_colors.get(&(out.clone(), color)) {
                                return reference.clone();
                            }
                        }

                        return l_number.clone();
                    };

                    (find_l_reference_color(WireColor::Red), find_l_reference_color(WireColor::Green))
                };

                let (u_red_reference, u_green_reference) = {
                    let find_u_reference_color = |color: WireColor| {
                        if let Some(reference) = reference_colors.get(&(u_number.clone(), color)) {
                            return reference.clone();
                        }

                        for wire in stmt.wires.iter().flatten().filter(|w| w.color == color) {
                            if let Some(reference) = reference_colors.get(&(wire.loc.clone(), color)) {
                                return reference.clone();
                            }
                        }

                        return u_number.clone();
                    };

                    (find_u_reference_color(WireColor::Red), find_u_reference_color(WireColor::Green))
                };

                let mut set_reference_color = |name: &String, color: WireColor, target: &String| {
                    if let Some(old_reference) = reference_colors.insert((name.clone(), color), target.clone()) {
                        // For all members referencing the old root, change it to the new root
                        for (_, connection) in reference_colors.iter_mut().filter(|((_, clr), conn)| **conn == old_reference && *clr == color) {
                            *connection = target.clone();
                        }
                    }
                };

                let mut set_l_reference_color = |color: WireColor, target: &String| {
                    set_reference_color(&l_number, color, target);

                    for out in stmt.out.iter().flatten() {
                        set_reference_color(out, color, target);
                    }
                };

                set_l_reference_color(WireColor::Red, &l_red_reference);
                set_l_reference_color(WireColor::Green, &l_green_reference);

                let mut set_u_reference_color = |color: WireColor, target: &String| {
                    set_reference_color(&u_number, color, target);

                    for wire in stmt.wires.iter().flatten().filter(|wire| wire.color == color) {
                        set_reference_color(&wire.loc, color, target);
                    }
                };

                set_u_reference_color(WireColor::Red, &u_red_reference);
                set_u_reference_color(WireColor::Green, &u_green_reference);
            }

            reference_colors
        };

        // Perform check whether the wires can reach
        {
            let mut last_used: HashMap<(String, WireColor), usize> = HashMap::new();
            // .1 Wire at .0 (line .3) cannot reach long enough - closest connection point is on network .2 (line .4)
            // FIXME: Refactor
            let mut errors: Vec<(String, WireColor, String, usize, usize)> = Vec::new();

            let mut perform_check = |idx: usize, name: &String, color: WireColor| {
                let resolved_name = reference_colors.get(&(name.clone(), color)).expect("ref colors doesn't resolve a name");
                if let Some(previous_idx) = last_used.insert((resolved_name.clone(), color), idx) {

                    if idx - previous_idx > 5 {
                        errors.push((name.clone(), color, resolved_name.clone(), idx, previous_idx));
                        return;
                    }
                }
            };

            for (idx, stmt) in self.iter().enumerate() {
                // Line numbers
                let l_number = format!("L{}", idx);
                let u_number = format!("U{}", idx);

                perform_check(idx, &l_number, WireColor::Red);
                perform_check(idx, &l_number, WireColor::Green);

                for out in stmt.out.iter().flatten() {
                    perform_check(idx, out, WireColor::Red);
                    perform_check(idx, out, WireColor::Green);
                }

                perform_check(idx, &u_number, WireColor::Red);
                perform_check(idx, &u_number, WireColor::Green);

                for wire in stmt.wires.iter().flatten() {
                    perform_check(idx, &wire.loc, wire.color);
                }
            }

            if errors.first().is_some() {
                return Err(Box::from("wires cannot reach long enough"));
            }
        }

        // Finally, assemble the connections
        let mut connections: HashMap<(String, WireColor), Vec<CircuitConnection>> = {
            let mut connections: HashMap<(String, WireColor), Vec<CircuitConnection>> = HashMap::new();
            
            let mut add_to_connections = |idx: usize, color: WireColor, mut conn_point: NonZeroUsize, stmt: &Statement| {
                let name = match conn_point.get() {
                    1 => format!("U{}", idx),
                    2 => format!("L{}", idx),
                    _ => unreachable!("invalid conn point")
                };

                // Workaround for constant combinators
                if matches!(stmt.expr, Expr::Constant(_)) {
                    conn_point = NonZeroUsize::new(1).unwrap();
                }

                let resolved_name = reference_colors.get(&(name, color)).expect("ref colors doesn't resolve to a name");

                let idx_tuple = (resolved_name.clone(), color);
                if let Some(value) = connections.get_mut(&idx_tuple) {
                    value.push(CircuitConnection { idx, conn_point });
                } else {
                    connections.insert(idx_tuple, vec![CircuitConnection { idx, conn_point }]);
                }
            };

            for (idx, stmt) in self.iter().enumerate() {
                add_to_connections(idx, WireColor::Red, NonZeroUsize::new(1).unwrap(), &stmt);
                add_to_connections(idx, WireColor::Red, NonZeroUsize::new(2).unwrap(), &stmt);
                add_to_connections(idx, WireColor::Green, NonZeroUsize::new(1).unwrap(), &stmt);
                add_to_connections(idx, WireColor::Green, NonZeroUsize::new(2).unwrap(), &stmt);
            }

            connections
        };

        // Prepare the returned array, and return it
        let circuits = {
            let mut circuits = Vec::new();
            
            let conns = connections.drain()
                .filter(|(_, conn)| conn.len() > 1)
                .map(|((_, color), connections)| Circuit { color, connections });

            circuits.extend(conns);

            circuits
        };

        Ok(circuits)
    } 
}