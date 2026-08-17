use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Default)]
struct Node<Key, Value> {
    children: HashMap<Key, Node<Key, Value>>,
    value: Option<Value>,
    count: usize,
}

impl<Key, Value> Node<Key, Value> {
    pub fn all_values(&self) -> Option<Vec<&Value>> {
        if self.value.is_some() {
            return self.value.as_ref().map(|item| vec![item]);
        }

        Some(
            self.children
                .values()
                .flat_map(|item| item.all_values())
                .flatten()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum TrieLookupResult<'a, T> {
    #[default]
    None,
    Ambiguous,
    Unique(&'a T),
}

impl<'a, T> TrieLookupResult<'a, T> {
    pub fn into_option(self) -> Option<&'a T> {
        match self {
            Self::None | Self::Ambiguous => None,
            Self::Unique(value) => Some(value),
        }
    }
}

impl<'a, T> From<TrieLookupResult<'a, T>> for Option<&'a T> {
    fn from(value: TrieLookupResult<'a, T>) -> Self {
        value.into_option()
    }
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

    pub fn get_by_prefix(&self, value: Value) -> TrieLookupResult<'_, Value> {
        let keys = value.as_ref();
        let mut node = &self.root;
        let mut keys = keys.iter().peekable();

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
                    // (See the test_full_prefix_single test below, 2nd to last test case/function)
                        && keys.next().map(|item| item == new_key).unwrap_or(true)
                    {
                        child_node = new_child_node;
                    }
                    if let Some(child_node_value) = child_node.value.as_ref() {
                        return TrieLookupResult::Unique(child_node_value);
                    }

                    return TrieLookupResult::None;
                }
                Some(child_node) => {
                    node = child_node;
                    if node.count != 1 && keys.peek().is_none() {
                        return TrieLookupResult::Ambiguous;
                    }
                }
                None => {
                    return TrieLookupResult::None;
                }
            }
        }

        TrieLookupResult::None
    }

    pub fn get_by_prefix_all(&self, value: Value) -> Option<Vec<&Value>> {
        let keys = value.as_ref();
        let mut node = &self.root;

        for key in keys {
            match node.children.get(key) {
                Some(child_node) => node = child_node,
                None => return node.value.as_ref().map(|item| vec![item]),
            }
        }

        if node.value.is_some() {
            return node.value.as_ref().map(|item| vec![item]);
        }

        Some(
            node.children
                .values()
                .flat_map(Node::all_values)
                .flatten()
                .collect::<Vec<_>>(),
        )
    }
}

#[cfg(test)]
mod trie_tests {
    use crate::trie::{PrefixTrie, TrieLookupResult as TLR};

    #[test]
    fn test_full_prefix_single() {
        let mut trie = PrefixTrie::default();
        trie.insert("abcdef");
        trie.insert("abhjkl");
        trie.insert("qwertz");

        assert_eq!(trie.get_by_prefix(""), TLR::None);
        assert_eq!(trie.get_by_prefix("a"), TLR::Ambiguous);
        assert_eq!(trie.get_by_prefix("ab"), TLR::Ambiguous);
        assert_eq!(trie.get_by_prefix("abb"), TLR::None);
        assert_eq!(trie.get_by_prefix("testing"), TLR::None);

        assert_eq!(trie.get_by_prefix("abc"), TLR::Unique(&"abcdef"));
        assert_eq!(trie.get_by_prefix("abh"), TLR::Unique(&"abhjkl"));

        assert_eq!(trie.get_by_prefix("q"), TLR::Unique(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qw"), TLR::Unique(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwe"), TLR::Unique(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwer"), TLR::Unique(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwert"), TLR::Unique(&"qwertz"));
        assert_eq!(trie.get_by_prefix("qwertz"), TLR::Unique(&"qwertz"));

        // This is important, as it ensures we correctly match the whole key we're given.
        assert_eq!(trie.get_by_prefix("qwerty"), TLR::None);
    }

    #[test]
    fn test_full_prefix_all() {
        let mut trie = PrefixTrie::default();
        trie.insert("abcdef");
        trie.insert("abhjkl");
        trie.insert("qwertz");

        assert!(trie.get_by_prefix_all("").is_some_and(|item| {
            item.into_iter()
                .all(|item| ["abcdef", "abhjkl", "qwertz"].contains(item))
        }));

        assert!(trie.get_by_prefix_all("a").is_some_and(|item| {
            item.into_iter()
                .all(|item| ["abcdef", "abhjkl"].contains(item))
        }));

        assert!(trie.get_by_prefix_all("ab").is_some_and(|item| {
            item.into_iter()
                .all(|item| ["abcdef", "abhjkl"].contains(item))
        }));

        assert_eq!(trie.get_by_prefix_all("abb"), None);
        assert_eq!(trie.get_by_prefix_all("testing"), None);

        assert_eq!(trie.get_by_prefix_all("abc"), Some(vec![&"abcdef"]));

        assert_eq!(trie.get_by_prefix_all("abh"), Some(vec![&"abhjkl"]));

        assert_eq!(trie.get_by_prefix_all("q"), Some(vec![&"qwertz"]));
        assert_eq!(trie.get_by_prefix_all("qw"), Some(vec![&"qwertz"]));
        assert_eq!(trie.get_by_prefix_all("qwe"), Some(vec![&"qwertz"]));
        assert_eq!(trie.get_by_prefix_all("qwer"), Some(vec![&"qwertz"]));
        assert_eq!(trie.get_by_prefix_all("qwert"), Some(vec![&"qwertz"]));
        assert_eq!(trie.get_by_prefix_all("qwertz"), Some(vec![&"qwertz"]));

        // This is important, as it ensures we correctly match the whole key we're given.
        assert_eq!(trie.get_by_prefix_all("qwerty"), None);
    }
}
