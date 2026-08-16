use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Default)]
struct Node<Key, Value> {
    children: HashMap<Key, Node<Key, Value>>,
    value: Option<Value>,
    count: usize,
}

#[derive(Debug, Default)]
pub struct PrefixTrie<Key, Value> {
    root: Node<Key, Value>,
}

impl<Key, Value> From<Vec<Value>> for PrefixTrie<Key, Value>
where
    Key: Default + Eq + Hash + Clone,
    Value: Default + AsRef<[Key]>,
{
    fn from(value: Vec<Value>) -> Self {
        let mut trie = PrefixTrie::default();
        for v in value {
            trie.insert(v);
        }
        trie
    }
}

impl<Key, Value> PrefixTrie<Key, Value>
where
    Key: Default + Eq + Hash + Clone,
    Value: Default + AsRef<[Key]>,
{
    pub fn insert(&mut self, value: Value) {
        let keys = value.as_ref();

        let mut node = &mut self.root;
        node.count += 1;

        for key in keys {
            node = node.children.entry(key.clone()).or_default();
            node.count += 1;
        }

        node.value = Some(value);
    }

    pub fn get_by_prefix(&self, value: Value) -> Option<&Value> {
        let keys = value.as_ref();
        let mut node = &self.root;
        let mut keys = keys.iter();

        while let Some(key) = keys.next() {
            match node.children.get(key) {
                // If we found a path that only contains one leaf node,
                // we'll need to walk to the leaf node in order to retrieve that value.
                Some(mut child_node) if child_node.count == 1 => {
                    while let Some((new_key, new_child_node)) = child_node.children.iter().next()
                    // We also need to make sure the actually match.
                    // If we run out of keys, that's alright, just means we managed to identify a
                    // unique path with the given input.
                    //
                    // If however we still have keys after finding a unique path,
                    // (eg when the input is the same as the value we want to retrieve from the
                    // trie), we need to make sure those match, otherwise an input such as
                    // "qwerty" would match "qwertz", given the path is unique before reaching
                    // the z vs y point.
                    // (See the test_full test below, 2nd to last test case/function)
                        && keys.next().map(|item| item == new_key).unwrap_or(true)
                    {
                        child_node = new_child_node;
                    }
                    return child_node.value.as_ref();
                }
                Some(child_node) => {
                    node = child_node;
                }
                None => {
                    return node.value.as_ref();
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod trie_tests {
    use crate::trie::PrefixTrie;

    #[test]
    fn test_full() {
        let mut trie = PrefixTrie::default();
        trie.insert("abcdef");
        trie.insert("abhjkl");
        trie.insert("qwertz");

        assert_eq!(trie.get_by_prefix(""), None);
        assert_eq!(trie.get_by_prefix("a"), None);
        assert_eq!(trie.get_by_prefix("ab"), None);
        assert_eq!(trie.get_by_prefix("abb"), None);
        assert_eq!(trie.get_by_prefix("testing"), None);

        assert_eq!(trie.get_by_prefix("abc"), Some(&"abcdef"));
        assert_eq!(trie.get_by_prefix("abh"), Some(&"abhjkl"));

        assert_eq!(trie.get_by_prefix("q"), Some(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qw"), Some(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwe"), Some(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwer"), Some(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwert"), Some(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwertz"), Some(&"qwertz"));

        // This is important, as it ensures we correctly match the whole key we're given.
        assert_eq!(trie.get_by_prefix("qwerty"), None);
    }
}
