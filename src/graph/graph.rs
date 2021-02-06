// imports
use super::errors::GraphErr;
use super::models::{FIELD, LOCATION};
use dashmap::DashMap;
use priority_queue::PriorityQueue;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::Mutex;

// Figures are simplified based on denomination Rules
pub type Figure = u8;
// State containing Positions of all figures (5 figures per player, 5 gray stoppers, 5 black stoppers)
// LOCATION: ([i16; 3], u8)

#[derive(Debug, Clone, Copy)]
pub struct GraphState([LOCATION; 35]);

// Serializable variant (due to some serde constraints the array seems to not be fertilizable directly)
#[derive(Debug, Clone, Serialize)]
pub struct ResizableGraphState {
    locations: Vec<LOCATION>,
}

impl From<GraphState> for ResizableGraphState {
    fn from(base: GraphState) -> ResizableGraphState {
        ResizableGraphState {
            locations: base.0.to_vec(),
        }
    }
}

impl From<&GraphState> for ResizableGraphState {
    fn from(base: &GraphState) -> ResizableGraphState {
        ResizableGraphState {
            locations: base.0.to_vec(),
        }
    }
}

// vertexmap
pub const BASE_VERTEX_MAP: [i16; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; // in case the naming changes these are statically mapped

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Field {
    pub occupied: bool,
    pub owner: Option<Figure>,
}

#[derive(Clone, Debug)]
pub struct Graph {
    /// Mapping of vertex ids and vertex values
    pub vertices: DashMap<FIELD, Field>,
    // This doesn't need to hold data about figures
}

impl Graph {
    pub fn new() -> Graph {
        return Graph {
            vertices: DashMap::with_capacity(100_usize),
        };
    }

    pub fn shrink_to_fit(&mut self) {
        self.vertices.shrink_to_fit();
    }

    pub fn fetch(&self, id: FIELD) -> Result<Field, GraphErr> {
        match self.vertices.get(&id) {
            Some(vertex) => Ok(*vertex),
            None => Err(GraphErr::NoSuchVertex {}),
        }
    }

    pub fn add_vertex(&mut self, id: FIELD, field: Field) -> Result<FIELD, GraphErr> {
        match self.vertices.insert(id, field) {
            Some(value) => {
                // insert the old vertex again to prevent UB
                self.vertices.insert(id, value);
                Err(GraphErr::CannotAddVertex {})
            }
            None => Ok(id),
        }
    }

    pub fn validate<'a>(
        &'a self,
        src: &'a FIELD,
        dest: &'a FIELD,
        state: &'a GraphState,
    ) -> Result<(bool, Figure), GraphErr> {
        // check if specified vertices exists
        self.fetch(*src)?;

        // test with a* if there's a possible path
        return Ok(self.a_star(src, dest, state));
    }

    fn a_star<'a>(&'a self, src: &'a FIELD, dest: &'a FIELD, state: &'a GraphState) -> (bool, u8) {
        // a star with heuristics
        let visited = VISITED.clone(); // RAM goes brrrrrrrrrrr

        // prepare visited. (Ah that sweet parallel overkill)
        let destination_owner_mutex = Mutex::new(u8::MAX);
        state.0.into_par_iter().for_each(|(field, figure)| {
            if field[0] != -1 {
                visited.insert(*field, true);
            } else if field == dest {
                *destination_owner_mutex.lock().unwrap() = *figure;
            }
        });
        let destination_owner = destination_owner_mutex.lock().unwrap().clone();

        let mut priority_queue: PriorityQueue<FIELD, i16> = PriorityQueue::new();
        let mut item = Some((*src, 0));

        // find initial neighbors for src
        Graph::add_neighbors(src, *dest, &mut priority_queue);

        // search until found
        while item.is_some() {
            // fetch visited state and vertex value
            let vertex = item.unwrap().0;
            let state = visited.get(&vertex).unwrap();

            // check if destination is reached
            if vertex == *dest {
                return (true, destination_owner);
            } else if !*state {
                // Evaluate edges on the fly. Didn't know why I even bothered with static edges
                // this actually is faster than having an ege map since calculating hashes
                // is more expensive than simple comparisons
                Graph::add_neighbors(&vertex, *dest, &mut priority_queue);
            }

            item = priority_queue.pop();
        }

        return (false, u8::MAX);
    }

    fn add_neighbors(src: &FIELD, dest: FIELD, queue: &mut PriorityQueue<FIELD, i16>) {
        if src[1] == 0 {
            // src is junction or corner
            // edge cases may
            let (first, second, third, fourth) = match src[0] {
                0 => ([1, 1, 0], [9, 6, 0], [4, 3, 0], [5, 6, 0]),
                5 => ([5, 1, 0], [9, 3, 5], [5, 1, 1], [6, 2, 5]),
                9 => ([9, 1, 8], [9, 1, 5], [9, 1, 0], [9, 1, 4]), // there's no need to calculate anything if the number is known
                _ => {
                    if src[0] < 4 {
                        (
                            [src[0], 1, src[0] - 1],
                            [src[0] + 1, 1, src[0]],
                            [src[0] + 5, 1, src[0]],
                            [src[0] + 4, 1, src[0]],
                        )
                    } else {
                        (
                            [src[0], 1, src[0] - 1],
                            [src[0] + 1, 1, src[0]],
                            [src[0], 1, src[0] - 5],
                            [src[0], 1, src[0] - 4],
                        )
                    }
                }
            };

            queue.push(first, Graph::heuristic(first, dest));
            queue.push(second, Graph::heuristic(second, dest));
            queue.push(third, Graph::heuristic(third, dest));
            queue.push(fourth, Graph::heuristic(fourth, dest));
        } else {
            // src is stop
            let positive_counter = src[1] + 1;
            let negative_counter = src[1] - 1;

            if (positive_counter == 6 && (src[0] > 4 && src[2] < 5))
                || (positive_counter == 3
                    && ((src[0] < 4 && src[2] < 4) || (src[0] > 4 && src[2] > 4)))
            {
                queue.push([src[1], 0, 0], Graph::heuristic([src[1], 0, 0], dest));
            } else {
                queue.push(
                    [src[0], positive_counter, src[1]],
                    Graph::heuristic([src[0], positive_counter, src[1]], dest),
                );
            }

            queue.push(
                [src[0], negative_counter, src[1]],
                Graph::heuristic([src[0], negative_counter, src[1]], dest),
            );
        }
    }

    fn heuristic(src: FIELD, dest: FIELD) -> i16 {
        if src[0] > 4 {
            // k1 ∊ J
            if dest[0] > 4 {
                // k2 ∊ J
                return (src[0] - dest[0]) * 4 + src[1] + dest[1];
            } else {
                // k2 ∊ C
                return dest[0] * 4 + src[1] + dest[1] + 6;
            }
        } else {
            // k1 ∊ C
            return (src[0] - dest[0]) * 4 + src[1] + dest[1];
        }
    }

    pub fn construct_graph() -> Graph {
        // this function doesn't rely on Result as it should have a static result
        let mut graph: Graph = Graph::new();
        let mut base_map: [FIELD; 10] = [[0, 0, 0]; 10];

        // the base nodes (junction, corners) need to be preinserted to do effective EDGE and stop mapping
        for i in 0..BASE_VERTEX_MAP.len() {
            base_map[i] = graph
                .add_vertex(
                    [BASE_VERTEX_MAP[i], 0, 0],
                    Field {
                        occupied: false,
                        owner: None,
                    },
                )
                .expect("Unable to add base vertex on graph creation");
        }

        // construct edges from edgemap. See pentagraph (python)

        // ensure only required space is used
        graph.shrink_to_fit();

        return graph;
    }

    // construct empty figures locations
    pub fn construct_figure_location(&self) -> DashMap<Figure, FIELD> {
        let figure_locations = DashMap::with_capacity(35);
        EMPTY_STATE.clone().0.iter().for_each(|figure| {
            figure_locations.insert(figure.1, figure.0);
        });

        return figure_locations;
    }

    // construct empty visited map for a*
    pub fn construct_visited(&self) -> DashMap<FIELD, bool> {
        let visited = DashMap::with_capacity(self.vertices.len());
        self.vertices.iter().for_each(|multi_ref| {
            visited.insert(*multi_ref.key(), multi_ref.value().occupied);
        });

        return visited;
    }
}

impl GraphState {
    // Creates an empty, as in no changes to the board but all player figures on board, state
    pub fn empty() -> GraphState {
        /*
        This 'construction' is not especially optimized to allow for better readability
        It doesn't really matter anyway since it's saved in a lazy constant
        */
        let mut figures: [LOCATION; 35] = [([0_i16; 3], 0_u8); 35];

        // adding players
        (0..5).into_iter().for_each(|figure| {
            (0..5).into_iter().for_each(|index| {
                if index > 5 {
                    figures[figure * 5 + index] = (
                        [(index - (figure - 1) * 5).try_into().unwrap(), 0, 0],
                        (index + 1).try_into().unwrap(),
                    )
                } else {
                    figures[figure * 5 + index] = (
                        [(index + 5).try_into().unwrap(), 0, 0],
                        (index + 1).try_into().unwrap(),
                    )
                };
            });
        });

        // adding black stoppers
        for figure in 25..30 {
            figures[figure] = (
                [(figure - 25).try_into().unwrap(), 0, 0],
                (figure + 1).try_into().unwrap(),
            );
        }

        // adding gray stoppers
        for figure in 30..35 {
            figures[figure] = ([-1, -1, -1], (figure + 1).try_into().unwrap());
        }

        GraphState(figures)
    }
}

// There's no need to construct the graph multiple times because it loads itself from a state
lazy_static! {
    pub static ref EMPTY_STATE: GraphState = GraphState::empty();
    pub static ref EMPTY_FIGURE_LOCATIONS: DashMap<Figure, FIELD> =
        GRAPH.construct_figure_location();
    pub static ref GRAPH: Graph = Graph::construct_graph();
    static ref VISITED: DashMap<FIELD, bool> = GRAPH.construct_visited();
}
