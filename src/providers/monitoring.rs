use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware};
use iron::typemap;
use slog_perf::TimeReporter;

use providers::LogProvider;

pub struct MonitoringProvider {
}

impl typemap::Key for MonitoringProvider {
    type Value = TimeReporter;
}

impl BeforeMiddleware for MonitoringProvider {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let rep_opt = req.extensions.get::<LogProvider>()
            .map(|log|
                TimeReporter::new(req.url.path().into_iter().collect::<String>(), log.new(o!())));

        if let Some(mut reporter) = rep_opt {
            reporter.start("time");
            req.extensions.insert::<MonitoringProvider>(reporter);
        }
        Ok(())
    }
}

impl AfterMiddleware for MonitoringProvider {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        if let Some(mut reporter) = req.extensions.remove::<MonitoringProvider>() {
            reporter.stop();
        }
        Ok(res)
    }
}
