use semtree_core::Chunk;
use crate::parser::ParsedTree;

pub trait Extractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk>;
}
