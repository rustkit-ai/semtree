use semtree_core::Chunk;
use semtree_tree::ParsedTree;

pub trait Extractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk>;
}
