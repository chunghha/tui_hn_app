#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SortBy {
    Score,
    Comments,
    Time,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}
