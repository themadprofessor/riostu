use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware};
use iron::typemap;
use slog::Logger;
use slog_perf::TimeReporter;

use logging::LoggerMiddleware;

pub struct MonitoringMiddleware {
}

impl typemap::Key for MonitoringMiddleware {
    type Value = TimeReporter;
}

impl BeforeMiddleware for MonitoringMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let rep_opt = req.extensions.get::<LoggerMiddleware>()
            .map(|ref log|
                TimeReporter::new(req.url.path().into_iter().collect::<String>(), log.new(o!())));

        if let Some(mut reporter) = rep_opt {
            reporter.start("time");
            req.extensions.insert::<MonitoringMiddleware>(reporter);
        }
        Ok(())
    }
}

impl AfterMiddleware for MonitoringMiddleware {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        if let Some(mut reporter) = req.extensions.remove::<MonitoringMiddleware>() {
            reporter.stop();
        }
        Ok(res)
    }
}
