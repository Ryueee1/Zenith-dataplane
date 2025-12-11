/// Event processing pipeline
use crate::Event;
use anyhow::Result;

pub trait PipelineStage: Send + Sync {
    fn process(&self, event: &Event) -> Result<Option<Event>>;
}

pub struct Pipeline {
    stages: Vec<Box<dyn PipelineStage>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
        }
    }
    
    pub fn add_stage<S: PipelineStage + 'static>(&mut self, stage: S) {
        self.stages.push(Box::new(stage));
    }
    
    pub fn execute(&self, mut event: Event) -> Result<Option<Event>> {
        for stage in &self.stages {
            match stage.process(&event)? {
                Some(e) => event = e,
                None => return Ok(None), // Event filtered out
            }
        }
        Ok(Some(event))
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
