use crate::{BodyAnalysis, MacroCall, RustAnalysis};
use std::collections::HashMap;

#[derive(Default)]
struct CallRanges {
    literals: Vec<usize>,
    diagnostics: Vec<usize>,
    class_tokens: Vec<usize>,
}

pub(crate) struct ReuseIndex<'a> {
    previous: &'a RustAnalysis,
    bodies: HashMap<(&'a str, &'a str), usize>,
    ranges: Vec<CallRanges>,
}

impl<'a> ReuseIndex<'a> {
    pub(crate) fn new(source: &'a str, previous: &'a RustAnalysis) -> Self {
        let mut order: Vec<_> = (0..previous.calls.len()).collect();
        order.sort_by_key(|&index| {
            let call = &previous.calls[index];
            (call.start, std::cmp::Reverse(call.end))
        });
        let mut parents = vec![None; previous.calls.len()];
        let mut stack: Vec<usize> = Vec::new();
        for &index in &order {
            let call = &previous.calls[index];
            while stack.last().is_some_and(|&parent| {
                let outer = &previous.calls[parent];
                call.start > outer.end || call.end > outer.end
            }) {
                stack.pop();
            }
            parents[index] = stack.last().copied();
            stack.push(index);
        }
        let owner = |start, end| {
            let position = order.partition_point(|&index| previous.calls[index].start <= start);
            let mut candidate = position.checked_sub(1).map(|position| order[position]);
            while let Some(index) = candidate {
                let call = &previous.calls[index];
                if start >= call.start && end <= call.end {
                    return Some(index);
                }
                candidate = parents[index];
            }
            None
        };
        let mut ranges: Vec<CallRanges> = (0..previous.calls.len())
            .map(|_| Default::default())
            .collect();
        for (index, &(start, end)) in previous.literals.iter().enumerate() {
            if let Some(call) = owner(start, end) {
                ranges[call].literals.push(index);
            }
        }
        for (index, diagnostic) in previous.diagnostics.iter().enumerate() {
            if let Some(call) = owner(diagnostic.start, diagnostic.end) {
                ranges[call].diagnostics.push(index);
            }
        }
        for (index, token) in previous.class_tokens.iter().enumerate() {
            if let Some(call) = owner(token.start, token.end) {
                ranges[call].class_tokens.push(index);
            }
        }
        let mut bodies = HashMap::new();
        for (index, call) in previous.calls.iter().enumerate() {
            if let Some(body) = source.get(call.start..call.end) {
                bodies.entry((call.name.as_str(), body)).or_insert(index);
            }
        }
        Self {
            previous,
            bodies,
            ranges,
        }
    }

    pub(crate) fn get(&self, call: &MacroCall, body: &str) -> Option<BodyAnalysis> {
        let &index = self.bodies.get(&(call.name.as_str(), body))?;
        let previous_start = self.previous.calls[index].start;
        let offset = |position| call.start + (position - previous_start);
        let ranges = &self.ranges[index];
        Some(BodyAnalysis {
            literals: ranges
                .literals
                .iter()
                .map(|&index| {
                    let (start, end) = self.previous.literals[index];
                    (offset(start), offset(end))
                })
                .collect(),
            diagnostics: ranges
                .diagnostics
                .iter()
                .map(|&index| {
                    let mut diagnostic = self.previous.diagnostics[index].clone();
                    diagnostic.start = offset(diagnostic.start);
                    diagnostic.end = offset(diagnostic.end);
                    diagnostic
                })
                .collect(),
            class_tokens: ranges
                .class_tokens
                .iter()
                .map(|&index| {
                    let mut token = self.previous.class_tokens[index].clone();
                    token.start = offset(token.start);
                    token.end = offset(token.end);
                    token
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests;
