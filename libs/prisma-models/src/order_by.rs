use crate::{RelationFieldRef, ScalarFieldRef};
use quaint::prelude::Order;
use std::string::ToString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrderBy {
    Scalar(OrderByScalar),
    Aggregation(OrderByAggregation),
    Relevance(OrderByRelevance),
}

impl OrderBy {
    pub fn path(&self) -> Vec<RelationFieldRef> {
        match self {
            OrderBy::Scalar(o) => o.path.clone(),
            OrderBy::Aggregation(o) => o.path.clone(),
            OrderBy::Relevance(_) => vec![],
        }
    }

    pub fn sort_order(&self) -> SortOrder {
        match self {
            OrderBy::Scalar(o) => o.sort_order,
            OrderBy::Aggregation(o) => o.sort_order,
            OrderBy::Relevance(o) => o.sort_order,
        }
    }

    pub fn field(&self) -> ScalarFieldRef {
        match self {
            OrderBy::Scalar(o) => o.field.clone(),
            OrderBy::Aggregation(o) => o.field(),
            OrderBy::Relevance(o) => o.field.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderByScalar {
    pub field: ScalarFieldRef,
    pub path: Vec<RelationFieldRef>,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderByAggregation {
    pub path: Vec<RelationFieldRef>,
    pub sort_order: SortOrder,
    pub sort_aggregation: SortAggregation,
}

impl OrderByAggregation {
    pub fn field(&self) -> ScalarFieldRef {
        let ids: Vec<_> = self
            .path
            .last()
            .unwrap()
            .related_model()
            .primary_identifier()
            .scalar_fields()
            .collect();
        let id = ids.first().unwrap();

        id.clone()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderByRelevance {
    pub field: ScalarFieldRef,
    pub sort_order: SortOrder,
}

impl OrderBy {
    pub fn scalar(field: ScalarFieldRef, path: Vec<RelationFieldRef>, sort_order: SortOrder) -> Self {
        Self::Scalar(OrderByScalar {
            field,
            path,
            sort_order,
        })
    }

    pub fn aggregation(path: Vec<RelationFieldRef>, sort_order: SortOrder, sort_aggregation: SortAggregation) -> Self {
        Self::Aggregation(OrderByAggregation {
            path,
            sort_order,
            sort_aggregation,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn into_order(self, reverse: bool) -> Order {
        match (self, reverse) {
            (SortOrder::Ascending, false) => Order::Asc,
            (SortOrder::Descending, false) => Order::Desc,
            (SortOrder::Ascending, true) => Order::Desc,
            (SortOrder::Descending, true) => Order::Asc,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub enum SortAggregation {
    Count,
    Avg,
    Sum,
    Min,
    Max,
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::Ascending => String::from("ASC"),
            SortOrder::Descending => String::from("DESC"),
        }
    }
}

impl From<ScalarFieldRef> for OrderBy {
    fn from(field: ScalarFieldRef) -> Self {
        Self::Scalar(OrderByScalar {
            field,
            path: vec![],
            sort_order: SortOrder::Ascending,
        })
    }
}
