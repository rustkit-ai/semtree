use crate::parser::ParsedTree;
use semtree_core::Chunk;

pub trait Extractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk>;
}
