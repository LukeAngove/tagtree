# RDBFS

This is an attempt at a fast filesystem that uses non-tree based lookups.

The idea is to use a tree of bits and a list of tags.

The idea is to have unique leaves, where each leaf contains the full list
of all files with that specific set of tags.

The nodes of the tree are a key value pairs, with the value being a
binary map, with each bit indicating if the child associated with that
bit also contains the tag being searched.

When the data base is given a set of tags, it will return all files that
match the given tag set, including files that also have additional tags.
