// imports
use derive_more::Display;

#[derive(Clone, Debug, PartialEq, Display)]
// Graph operation error
pub enum GraphErr {
    // There is no vertex with the given id in the graph
    NoSuchVertex,

    // Could not add an vertex to the graph
    CannotAddVertex,

    // Couldn't construct State from database
    CannotConstructState(String),
}
