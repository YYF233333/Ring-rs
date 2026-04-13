use super::*;

impl AppStateInner {
    pub(super) fn snapshot_for_trace(&self) -> HarnessSnapshot {
        HarnessSnapshot {
            render_state: self.render_state.clone(),
            waiting: self.waiting.clone(),
            script_finished: self.script_finished,
            playback_mode: self.playback_mode.clone(),
            host_screen: self.host_screen.clone(),
            history_count: self.history.len(),
        }
    }

    pub(super) fn clear_trace(&mut self) {
        self.trace_events.clear();
        self.trace_seq = 0;
        self.logical_time_ms = 0;
    }

    pub(super) fn record_trace(&mut self, kind: &str, data: serde_json::Value) {
        self.trace_events.push(HarnessTraceEvent {
            seq: self.trace_seq,
            logical_time_ms: self.logical_time_ms,
            kind: kind.to_string(),
            data,
        });
        self.trace_seq += 1;
    }

    pub(super) fn build_trace_bundle(
        &self,
        dt_seconds: f32,
        steps_run: usize,
        stop_reason: impl Into<String>,
    ) -> HarnessTraceBundle {
        HarnessTraceBundle {
            metadata: HarnessTraceMetadata {
                dt_seconds,
                steps_run,
                stop_reason: stop_reason.into(),
                owner_label: self.client_owner.as_ref().map(|owner| owner.label.clone()),
            },
            events: self.trace_events.clone(),
            final_snapshot: self.snapshot_for_trace(),
        }
    }

    pub fn debug_run_until(
        &mut self,
        dt: f32,
        max_steps: usize,
        stop_on_wait: bool,
        stop_on_script_finished: bool,
    ) -> HarnessTraceBundle {
        self.clear_trace();
        let mut steps = 0usize;

        let stop_reason = loop {
            if steps >= max_steps {
                break "max_steps";
            }
            if !self.host_screen.allows_progression() {
                break "host_screen_blocked";
            }

            self.process_tick(dt);
            steps += 1;

            if stop_on_script_finished && self.script_finished {
                break "script_finished";
            }
            if stop_on_wait && self.waiting != WaitingFor::Nothing {
                break "waiting";
            }
        };

        self.build_trace_bundle(dt, steps, stop_reason)
    }
}
