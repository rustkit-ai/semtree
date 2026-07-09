use std::collections::HashMap;

use semtree_core::Chunk;

// BM25 tuning constants (Robertson/Zaragoza defaults).
const K1: f32 = 1.2;
const B: f32 = 0.75;
// A chunk's name is repeated this many times so identifier matches rank high.
const NAME_BOOST: usize = 3;

/// In-memory BM25 lexical index over indexed chunks.
///
/// Built fresh from the registry at query time - no extra persistence. The
/// tokenizer is code-aware: it splits on non-alphanumeric characters (so
/// `rate_limit` → `rate`, `limit`) and on camelCase boundaries (so
/// `TokenBucket` → `token`, `bucket`), which is what lets a lexical query
/// match identifiers a pure substring grep would phrase differently.
pub struct LexicalIndex {
    ids: Vec<String>,
    doc_len: Vec<f32>,
    avgdl: f32,
    /// term -> list of (document index, term frequency)
    postings: HashMap<String, Vec<(usize, u32)>>,
}

impl LexicalIndex {
    /// Builds the index from an iterator of chunks (typically `registry.iter()`).
    pub fn from_chunks<'a>(chunks: impl Iterator<Item = &'a Chunk>) -> Self {
        let mut ids = Vec::new();
        let mut doc_len = Vec::new();
        let mut postings: HashMap<String, Vec<(usize, u32)>> = HashMap::new();

        for chunk in chunks {
            let doc_idx = ids.len();
            ids.push(chunk.id.clone());

            let mut text = String::new();
            if let Some(name) = &chunk.name {
                for _ in 0..NAME_BOOST {
                    text.push_str(name);
                    text.push(' ');
                }
            }
            text.push_str(&chunk.content);

            let tokens = tokenize(&text);
            doc_len.push(tokens.len() as f32);

            let mut tf: HashMap<String, u32> = HashMap::new();
            for t in tokens {
                *tf.entry(t).or_insert(0) += 1;
            }
            for (term, count) in tf {
                postings.entry(term).or_default().push((doc_idx, count));
            }
        }

        let n = ids.len().max(1) as f32;
        let avgdl = doc_len.iter().sum::<f32>() / n;
        let avgdl = if avgdl == 0.0 { 1.0 } else { avgdl };

        Self {
            ids,
            doc_len,
            avgdl,
            postings,
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Returns up to `limit` chunk ids ranked by BM25 score (descending).
    pub fn search(&self, query: &str, limit: usize) -> Vec<(String, f32)> {
        let n = self.ids.len() as f32;
        let mut q_terms = tokenize(query);
        q_terms.sort();
        q_terms.dedup();

        let mut scores: HashMap<usize, f32> = HashMap::new();
        for term in q_terms {
            let Some(postings) = self.postings.get(&term) else {
                continue;
            };
            let df = postings.len() as f32;
            let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();
            for &(doc, tf) in postings {
                let tf = tf as f32;
                let dl = self.doc_len[doc];
                let denom = tf + K1 * (1.0 - B + B * dl / self.avgdl);
                *scores.entry(doc).or_insert(0.0) += idf * (tf * (K1 + 1.0)) / denom;
            }
        }

        let mut ranked: Vec<(usize, f32)> = scores.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit);
        ranked
            .into_iter()
            .map(|(doc, score)| (self.ids[doc].clone(), score))
            .collect()
    }
}

/// Splits text into lowercase tokens, breaking on non-alphanumeric characters
/// and camelCase boundaries. Tokens shorter than 2 chars are dropped.
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            buf.push(ch);
        } else if !buf.is_empty() {
            split_camel(&buf, &mut tokens);
            buf.clear();
        }
    }
    if !buf.is_empty() {
        split_camel(&buf, &mut tokens);
    }
    tokens
}

fn split_camel(word: &str, out: &mut Vec<String>) {
    let chars: Vec<char> = word.chars().collect();
    let mut start = 0;
    for i in 1..chars.len() {
        // lower -> upper transition marks a camelCase boundary
        if chars[i - 1].is_lowercase() && chars[i].is_uppercase() {
            push_token(&chars[start..i], out);
            start = i;
        }
    }
    push_token(&chars[start..], out);
}

fn push_token(chars: &[char], out: &mut Vec<String>) {
    if chars.len() >= 2 {
        out.push(chars.iter().collect::<String>().to_lowercase());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semtree_core::{ChunkKind, Language, Span};
    use std::path::PathBuf;

    fn chunk(id: &str, name: &str, content: &str) -> Chunk {
        Chunk {
            id: id.to_string(),
            path: PathBuf::from(format!("{id}.rs")),
            language: Language::Rust,
            kind: ChunkKind::Function,
            name: Some(name.to_string()),
            content: content.to_string(),
            span: Span::new(0, content.len(), 0, 0),
            doc: None,
        }
    }

    #[test]
    fn tokenizer_splits_camel_and_snake() {
        let toks = tokenize("TokenBucket rate_limit HTTPServer");
        assert!(toks.contains(&"token".to_string()));
        assert!(toks.contains(&"bucket".to_string()));
        assert!(toks.contains(&"rate".to_string()));
        assert!(toks.contains(&"limit".to_string()));
    }

    #[test]
    fn bm25_ranks_identifier_match_first() {
        let chunks = [
            chunk(
                "a",
                "TokenBucket",
                "fn refill(&mut self) { self.tokens += 1; }",
            ),
            chunk("b", "parse_config", "fn parse_config() { read_file(); }"),
            chunk(
                "c",
                "log_message",
                "fn log_message(msg: &str) { println!(); }",
            ),
        ];
        let idx = LexicalIndex::from_chunks(chunks.iter());

        // Query phrased differently than the substring, but shares the token.
        let hits = idx.search("token bucket throttling", 3);
        assert_eq!(hits.first().map(|(id, _)| id.as_str()), Some("a"));
    }

    #[test]
    fn empty_query_returns_nothing() {
        let chunks = [chunk("a", "foo", "bar")];
        let idx = LexicalIndex::from_chunks(chunks.iter());
        assert!(idx.search("", 5).is_empty());
    }
}
