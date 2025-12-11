/// Event processor implementations
use crate::{Event, pipeline::PipelineStage};
use anyhow::Result;

pub struct EventProcessor;

/// Filter stage - drops events based on criteria
pub struct FilterStage {
    predicate: Box<dyn Fn(&Event) -> bool + Send + Sync>,
}

impl FilterStage {
    pub fn new<F>(predicate: F) -> Self 
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
        }
    }
}

impl PipelineStage for FilterStage {
    fn process(&self, event: &Event) -> Result<Option<Event>> {
        if (self.predicate)(event) {
            Ok(Some(event.clone()))
        } else {
            Ok(None)
        }
    }
}

/// Transform stage - modifies events
pub struct TransformStage {
    transformer: Box<dyn Fn(Event) -> Event + Send + Sync>,
}

impl TransformStage {
    pub fn new<F>(transformer: F) -> Self 
    where
        F: Fn(Event) -> Event + Send + Sync + 'static,
    {
        Self {
            transformer: Box::new(transformer),
        }
    }
}

impl PipelineStage for TransformStage {
    fn process(&self, event: &Event) -> Result<Option<Event>> {
        Ok(Some((self.transformer)(event.clone())))
    }
}
