use pareto_front::{Dominate, ParetoFront};
use std::collections::HashMap;

use crate::game::{
    state::InProgress,
    units::{Progress, Quality},
    Action, ComboAction, Effects, Settings,
};

use super::ActionSequence;

#[derive(Debug, Clone, Copy)]
pub struct SearchTrace<'a> {
    pub parent: &'a SearchNode<'a>,
    pub action: ActionSequence,
}

impl<'a> SearchTrace<'a> {
    pub fn actions(self) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();
        self.do_trace(&mut actions);
        actions.reverse();
        actions
    }

    fn do_trace(self, actions: &mut Vec<Action>) {
        for action in self.action.actions().iter().rev() {
            actions.push(*action);
        }
        if let Some(parent) = self.parent.trace {
            parent.do_trace(actions);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchNode<'a> {
    pub state: InProgress,
    pub trace: Option<SearchTrace<'a>>,
}

impl<'a> Dominate for SearchNode<'a> {
    fn dominate(&self, other: &Self) -> bool {
        self.state.progress >= other.state.progress && self.state.quality >= other.state.quality
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParetoKey {
    pub combo: Option<ComboAction>,
    pub durability: i32,
    pub effects: Effects,
}

impl From<&SearchNode<'_>> for ParetoKey {
    fn from(value: &SearchNode) -> Self {
        ParetoKey {
            combo: value.state.combo,
            durability: value.state.durability,
            effects: value.state.effects,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParetoValue {
    pub progress: Progress,
    pub quality: Quality,
}

impl<'a> From<&SearchNode<'a>> for ParetoValue {
    fn from(value: &SearchNode<'a>) -> Self {
        ParetoValue {
            progress: value.state.progress,
            quality: value.state.quality,
        }
    }
}

impl Dominate for ParetoValue {
    fn dominate(&self, other: &Self) -> bool {
        self.progress >= other.progress && self.quality >= other.quality
    }
}

type FrontHashMap<T> = HashMap<ParetoKey, ParetoFront<T>>;

pub struct SearchQueue<'a> {
    current: Vec<SearchNode<'a>>,
    buckets: Vec<FrontHashMap<SearchNode<'a>>>,
    pareto_front: FrontHashMap<ParetoValue>,
}

impl<'a> SearchQueue<'a> {
    pub fn new(settings: Settings) -> SearchQueue<'a> {
        SearchQueue {
            current: Vec::new(),
            buckets: vec![FrontHashMap::new(); (settings.max_cp + 1) as usize],
            pareto_front: FrontHashMap::new(),
        }
    }

    pub fn push(&mut self, value: SearchNode<'a>) {
        let key = ParetoKey::from(&value);
        self.buckets[value.state.cp as usize]
            .entry(key)
            .or_default()
            .push(value);
    }

    pub fn pop(&mut self) -> Option<SearchNode<'a>> {
        if let Some(node) = self.current.pop() {
            return Some(node);
        } else if self.pop_bucket() {
            return self.pop();
        } else {
            return None;
        }
    }

    fn pop_bucket(&mut self) -> bool {
        if let Some(bucket) = self.buckets.pop() {
            for (key, front) in bucket {
                let global_front = self.pareto_front.entry(key).or_default();
                for value in front {
                    if global_front.push(ParetoValue::from(&value)) {
                        self.current.push(value);
                    }
                }
            }
            true
        } else {
            false
        }
    }
}
