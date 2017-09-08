use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware};
use iron::typemap;
use slog_perf::TimeReporter;

use providers::Log;

pub struct Monitoring {
}

impl typemap::Key for Monitoring {
    type Value = TimeReporter;
}

impl BeforeMiddleware for Monitoring {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let rep_opt = req.extensions.get::<Log>()
            .map(|log|
                TimeReporter::new(req.url.path().into_iter().collect::<String>(), log.new(o!())));

        if let Some(mut reporter) = rep_opt {
            reporter.start("time");
            req.extensions.insert::<Monitoring>(reporter);
        }
        Ok(())
    }
}

impl AfterMiddleware for Monitoring {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        if let Some(mut reporter) = req.extensions.remove::<Monitoring>() {
            reporter.stop();
        }
        Ok(res)
    }
}
