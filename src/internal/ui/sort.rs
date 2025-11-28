use crate::internal::models::Story;
use std::cmp::Ordering;

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

/// Compare two stories by score (returns Ordering).
fn cmp_score(a: &Story, b: &Story) -> Ordering {
    a.score.cmp(&b.score)
}

/// Compare two stories by number of comments/descendants.
fn cmp_comments(a: &Story, b: &Story) -> Ordering {
    a.descendants.cmp(&b.descendants)
}

/// Compare two stories by time.
fn cmp_time(a: &Story, b: &Story) -> Ordering {
    a.time.cmp(&b.time)
}

/// Apply the requested sort order (ascending/descending) to a base Ordering.
fn apply_ordering(ord: Ordering, sort_order: SortOrder) -> Ordering {
    match sort_order {
        SortOrder::Ascending => ord,
        SortOrder::Descending => ord.reverse(),
    }
}

/// Sort stories in-place based on the specified criteria and order
pub fn sort_stories(stories: &mut [Story], sort_by: SortBy, sort_order: SortOrder) {
    stories.sort_unstable_by(|a, b| {
        let base = match sort_by {
            SortBy::Score => cmp_score(a, b),
            SortBy::Comments => cmp_comments(a, b),
            SortBy::Time => cmp_time(a, b),
        };
        apply_ordering(base, sort_order)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_helpers_score() {
        let a = Story {
            id: 1,
            score: Some(100),
            ..Default::default()
        };
        let b = Story {
            id: 2,
            score: Some(200),
            ..Default::default()
        };

        assert_eq!(cmp_score(&a, &b), std::cmp::Ordering::Less);
        assert_eq!(
            apply_ordering(cmp_score(&a, &b), SortOrder::Ascending),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            apply_ordering(cmp_score(&a, &b), SortOrder::Descending),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn cmp_helpers_comments() {
        let a = Story {
            id: 1,
            descendants: Some(5),
            ..Default::default()
        };
        let b = Story {
            id: 2,
            descendants: Some(10),
            ..Default::default()
        };

        assert_eq!(cmp_comments(&a, &b), std::cmp::Ordering::Less);
        assert_eq!(
            apply_ordering(cmp_comments(&a, &b), SortOrder::Ascending),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            apply_ordering(cmp_comments(&a, &b), SortOrder::Descending),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn cmp_helpers_time() {
        let a = Story {
            id: 1,
            time: Some(1000),
            ..Default::default()
        };
        let b = Story {
            id: 2,
            time: Some(2000),
            ..Default::default()
        };

        assert_eq!(cmp_time(&a, &b), std::cmp::Ordering::Less);
        assert_eq!(
            apply_ordering(cmp_time(&a, &b), SortOrder::Ascending),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            apply_ordering(cmp_time(&a, &b), SortOrder::Descending),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn sort_stories_uses_helpers() {
        // Create sample stories
        let s1 = Story {
            id: 1,
            score: Some(100),
            time: Some(1000),
            descendants: Some(50),
            ..Default::default()
        };
        let s2 = Story {
            id: 2,
            score: Some(200),
            time: Some(2000),
            descendants: Some(10),
            ..Default::default()
        };
        let s3 = Story {
            id: 3,
            score: Some(50),
            time: Some(3000),
            descendants: Some(100),
            ..Default::default()
        };

        let mut stories = vec![s1.clone(), s2.clone(), s3.clone()];

        // Descending by score
        sort_stories(&mut stories, SortBy::Score, SortOrder::Descending);
        assert_eq!(stories[0].id, 2);
        assert_eq!(stories[1].id, 1);
        assert_eq!(stories[2].id, 3);

        // Ascending by score
        sort_stories(&mut stories, SortBy::Score, SortOrder::Ascending);
        assert_eq!(stories[0].id, 3);
        assert_eq!(stories[1].id, 1);
        assert_eq!(stories[2].id, 2);

        // Time descending
        sort_stories(&mut stories, SortBy::Time, SortOrder::Descending);
        assert_eq!(stories[0].id, 3);
        assert_eq!(stories[1].id, 2);
        assert_eq!(stories[2].id, 1);

        // Comments descending
        sort_stories(&mut stories, SortBy::Comments, SortOrder::Descending);
        assert_eq!(stories[0].id, 3);
        assert_eq!(stories[1].id, 1);
        assert_eq!(stories[2].id, 2);
    }

    #[test]
    fn sort_with_load_more_combines_and_sorts() {
        // Initial two stories
        let s1 = Story {
            id: 1,
            score: Some(100),
            ..Default::default()
        };
        let s2 = Story {
            id: 2,
            score: Some(200),
            ..Default::default()
        };

        let mut stories = vec![s1, s2];

        // Sort descending by score
        sort_stories(&mut stories, SortBy::Score, SortOrder::Descending);
        assert_eq!(stories[0].id, 2);
        assert_eq!(stories[1].id, 1);

        // New stories loaded
        let s3 = Story {
            id: 3,
            score: Some(300),
            ..Default::default()
        };
        let s4 = Story {
            id: 4,
            score: Some(50),
            ..Default::default()
        };

        stories.extend(vec![s3, s4]);

        // Re-sort after loading more
        sort_stories(&mut stories, SortBy::Score, SortOrder::Descending);

        // Expect fully sorted order: 3,2,1,4
        assert_eq!(stories.len(), 4);
        assert_eq!(stories[0].id, 3);
        assert_eq!(stories[1].id, 2);
        assert_eq!(stories[2].id, 1);
        assert_eq!(stories[3].id, 4);
    }
}
