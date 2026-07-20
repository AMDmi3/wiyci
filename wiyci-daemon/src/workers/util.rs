// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use tracing::{Instrument as _, Span, debug, error};

const DEFAULT_TASK_WAIT: Duration = if cfg!(debug_assertions) {
    Duration::from_secs(5)
} else {
    Duration::from_secs(60)
};
const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(60);

pub struct PollingWorkerRunner<Task, GetTaskFn, ProcessTaskFn> {
    name: &'static str,
    get_task_fn: GetTaskFn,
    process_task_fn: ProcessTaskFn,
    span_from_task_fn: Option<fn(&Task) -> Span>,
    task_wait: Duration,
    retry_wait: Duration,
}

impl<Task, Error, GetTaskFn, ProcessTaskFn> PollingWorkerRunner<Task, GetTaskFn, ProcessTaskFn>
where
    Error: std::fmt::Display,
    GetTaskFn: AsyncFn() -> Result<Option<Task>, Error>,
    ProcessTaskFn: AsyncFn(&Task) -> Result<(), Error>,
{
    pub fn new(name: &'static str, get_task_fn: GetTaskFn, process_task_fn: ProcessTaskFn) -> Self {
        Self {
            name,
            get_task_fn,
            process_task_fn,
            span_from_task_fn: None,
            task_wait: DEFAULT_TASK_WAIT,
            retry_wait: DEFAULT_RETRY_WAIT,
        }
    }

    #[allow(unused)]
    pub fn with_task_wait(mut self, wait: Duration) -> Self {
        self.task_wait = wait;
        self
    }

    #[allow(unused)]
    pub fn with_retry_wait(mut self, wait: Duration) -> Self {
        self.retry_wait = wait;
        self
    }

    #[allow(unused)]
    pub fn with_span(mut self, span_from_task_fn: fn(&Task) -> Span) -> Self {
        self.span_from_task_fn = Some(span_from_task_fn);
        self
    }

    pub async fn run(&self) -> Result<(), Error>
    where
        Error: std::fmt::Display,
        GetTaskFn: AsyncFn() -> Result<Option<Task>, Error>,
        ProcessTaskFn: AsyncFn(&Task) -> Result<(), Error>,
    {
        loop {
            let task = match (self.get_task_fn)().await {
                Err(error) => {
                    counter!("wiyci_daemon_worker_runs_total", "worker" => self.name, "status" => "failure").increment(1);
                    error!(%error, "error while polling for task");
                    tokio::time::sleep(self.retry_wait).await;
                    continue;
                }
                Ok(None) => {
                    counter!("wiyci_daemon_worker_runs_total", "worker" => self.name, "status" => "no data").increment(1);
                    debug!("waiting for tasks");
                    tokio::time::sleep(self.task_wait).await;
                    continue;
                }
                Ok(Some(task)) => task,
            };

            let start = Instant::now();
            let span = self
                .span_from_task_fn
                .as_ref()
                .map(|f| f(&task))
                .unwrap_or_else(Span::none);
            let res = async {
                debug!("start processing");
                let res = (self.process_task_fn)(&task).await;

                let duration = Instant::now()
                    .saturating_duration_since(start)
                    .as_secs_f64();
                histogram!("wiyci_daemon_worker_run_duration_seconds", "worker" => self.name).record(duration);

                match &res {
                    Err(error) => {
                        counter!("wiyci_daemon_worker_runs_total", "worker" => self.name, "status" => "failure").increment(1);
                        error!(%error, "error while processing task");
                    }
                    Ok(..) => {
                        debug!("done processing");
                        counter!("wiyci_daemon_worker_runs_total", "worker" => self.name, "status" => "success").increment(1);
                    },
                }

                res
            }.instrument(span).await;

            if res.is_err() {
                tokio::time::sleep(self.retry_wait).await;
            }
        }
    }
}
